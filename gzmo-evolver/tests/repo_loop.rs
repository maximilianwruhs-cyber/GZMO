//! Real-Git hermetic proofs for independent candidate workspaces.
//!
//! Uses temporary bare origins and subprocess Git only — no network.
//! Checkout stores a GitHub-shaped origin URL; a test-only ProcessRunner injects
//! command-local `url.<file>.insteadOf=<github>` + file protocol so product
//! argv/identity stay production-shaped.

use chrono::{TimeZone, Utc};
use evolution_contracts::{
    AuthorityTier, CandidateId, CandidateKind, CandidateManifest, CandidateState, CandidateTarget,
    ResourceBudget, CANDIDATE_SCHEMA,
};
use fs2::FileExt;
use gzmo_evolver::{
    cleanup_workspace, prepare_candidate, refresh_baseline_before_mission,
    validate_remote_identity, Clock, CoordinatorLock, FakeProcessRunner, GitError, GitRepository,
    ManualClock, MissionAdapter, ProcessError, ProcessOutput, ProcessRunner, ProcessSpec,
    RepoEvolverConfig, StateStore, SystemProcessRunner, TransitionMetadata, GIT_FETCH_TIMEOUT_SECS,
    GIT_OUTPUT_CAP_BYTES, GIT_TIMEOUT_SECS, MIRROR_LOCK_NAME, MISSION_STAGING_DIR, NO_FETCH_URL,
    NO_PUSH_URL, WORKSPACES_DIR,
};
use std::fs::{self, OpenOptions};
use std::os::unix::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::TempDir;

const POLICY_TOML: &str = r#"
schema = "gzmo.repo_evolver.policy/v1"
owner = "maximilianwruhs-cyber"
repository = "GZMO"
candidate_kind = "code"
max_active_candidates = 1
max_repair_attempts = 2
allowed_branch_prefix = "evolve/"

[budget]
wall_seconds = 2700
max_attempts = 1
max_changed_files = 20
max_added_lines = 1500
max_tool_calls = 80
max_input_tokens = 250000
max_output_tokens = 50000
allow_missing_energy_meter = true

[protected_paths]
protected_paths = [
  ".github/workflows/",
  "docs/superpowers/specs/",
  "docs/ADR-",
  "AGENTS.md",
  "Cargo.toml",
  "Cargo.lock",
  "crates/evolution-contracts/",
  "gzmo-evolver/",
  "config/repo-evolver.policy.toml",
]

[[gates]]
name = "format"
class = "hard_floor"
argv = ["cargo", "fmt", "--all", "--", "--check"]
timeout_seconds = 300

[[gates]]
name = "clippy"
class = "hard_floor"
argv = ["cargo", "clippy", "--all-targets", "--", "-D", "warnings"]
timeout_seconds = 900

[[gates]]
name = "tests"
class = "hard_floor"
argv = ["cargo", "test", "--all"]
timeout_seconds = 1800

[[gates]]
name = "opportunity-contract"
class = "hard_floor"
argv = ["bash", "scripts/opportunity-discovery-check.sh"]
timeout_seconds = 300
"#;

const FIXTURE_MARKDOWN: &str = r#"# Mission card

## Mission

Body of the mission.

## Constraints

- stay hermetic

## Verify

```bash
true
```
"#;

/// Exact GitHub-shaped origin stored in the trusted checkout (product identity).
const GITHUB_ORIGIN: &str = "https://github.com/maximilianwruhs-cyber/GZMO.git";

/// Test-only runner: keeps production argv, injects insteadOf + file allow for hermetic bare origin.
#[derive(Debug, Clone)]
struct HermeticGitRunner {
    inner: SystemProcessRunner,
    local_file_url: String,
    github_url: String,
}

impl HermeticGitRunner {
    fn new(origin_path: &Path) -> Self {
        let abs = fs::canonicalize(origin_path).unwrap();
        let local_file_url = format!("file://{}", abs.display());
        Self {
            inner: SystemProcessRunner,
            local_file_url,
            github_url: GITHUB_ORIGIN.to_owned(),
        }
    }

    fn inject(&self, spec: &ProcessSpec) -> Result<ProcessSpec, ProcessError> {
        let prog = spec.program.to_string_lossy();
        if !(prog.ends_with("git") || prog == "git") {
            return Ok(spec.clone());
        }
        let mut args = Vec::with_capacity(spec.args.len() + 4);
        args.push("-c".to_owned());
        args.push(format!(
            "url.{}.insteadOf={}",
            self.local_file_url, self.github_url
        ));
        args.push("-c".to_owned());
        args.push("protocol.file.allow=always".to_owned());
        args.extend(spec.args.iter().cloned());
        ProcessSpec::new(
            &spec.program,
            args,
            &spec.cwd,
            spec.env.clone(),
            spec.output_cap,
            spec.timeout,
        )
    }
}

impl ProcessRunner for HermeticGitRunner {
    fn run(&self, spec: &ProcessSpec) -> Result<ProcessOutput, ProcessError> {
        let injected = self.inject(spec)?;
        self.inner.run(&injected)
    }
}

/// Routes git through HermeticGitRunner and bash mission producer through FakeProcessRunner.
struct HybridRunner {
    git: HermeticGitRunner,
    fake_mission: FakeProcessRunner,
}

impl ProcessRunner for HybridRunner {
    fn run(&self, spec: &ProcessSpec) -> Result<ProcessOutput, ProcessError> {
        let prog = spec.program.to_string_lossy();
        if prog.ends_with("git") || prog == "git" {
            return self.git.run(spec);
        }
        if prog.ends_with("bash") || prog == "bash" || prog == "/bin/bash" {
            return self.fake_mission.run(spec);
        }
        self.git.run(spec)
    }
}

/// Strips `--no-local` from candidate clone to force shared objects (production must reject).
#[derive(Debug)]
struct StripNoLocalRunner {
    inner: HermeticGitRunner,
}

impl ProcessRunner for StripNoLocalRunner {
    fn run(&self, spec: &ProcessSpec) -> Result<ProcessOutput, ProcessError> {
        let mut injected = self.inner.inject(spec)?;
        if injected.args.iter().any(|a| a == "clone") {
            injected.args.retain(|a| a != "--no-local");
        }
        self.inner.inner.run(&injected)
    }
}

struct Fixture {
    _root: TempDir,
    origin: PathBuf,
    checkout: PathBuf,
    state_dir: PathBuf,
    config: RepoEvolverConfig,
    config_path: PathBuf,
    baseline_before: String,
    hermetic: HermeticGitRunner,
}

impl Fixture {
    fn new() -> Self {
        let root = TempDir::new().unwrap();
        let origin = root.path().join("origin.git");
        let checkout = root.path().join("checkout");
        let state_dir = root.path().join("state");
        let worker = root.path().join("omp");

        run_git(
            &root.path().to_path_buf(),
            &["init", "--bare", origin.to_str().unwrap()],
        );

        let seed = root.path().join("seed");
        run_git(
            root.path(),
            &["clone", origin.to_str().unwrap(), seed.to_str().unwrap()],
        );
        // Deterministic tree from committed fixture-repo/ (no ambient scripts).
        install_committed_fixture_tree(&seed);
        git_config_identity(&seed);
        run_git(&seed, &["add", "."]);
        run_git(&seed, &["commit", "-m", "initial"]);
        run_git(&seed, &["branch", "-M", "main"]);
        run_git(&seed, &["push", "origin", "main"]);

        run_git(
            root.path(),
            &[
                "clone",
                "--branch",
                "main",
                origin.to_str().unwrap(),
                checkout.to_str().unwrap(),
            ],
        );
        git_config_identity(&checkout);
        // Product identity: store GitHub-shaped origin (not the local bare path).
        run_git(&checkout, &["remote", "set-url", "origin", GITHUB_ORIGIN]);

        let baseline_before = rev_parse(&checkout, "HEAD");

        OpenOptions::new()
            .write(true)
            .create(true)
            .mode(0o755)
            .open(&worker)
            .unwrap();

        let config_path = root.path().join("repo-evolver.toml");
        let body = format!(
            r#"
state_dir = "{state}"
[repo]
path = "{repo}"
remote = "origin"
base_branch = "main"
owner = "maximilianwruhs-cyber"
repository = "GZMO"
[mission]
json_rel = "opportunity-discovery/next-mission.json"
markdown_rel = "opportunity-discovery/next-mission.md"
refresh_argv = ["bash", "scripts/opportunity-next-mission.sh"]
[worker]
executable = "{worker}"
profile = "gzmo-repo-evolver-worker"
[policy]
repo_path = "config/repo-evolver.policy.toml"
"#,
            state = state_dir.display(),
            repo = checkout.display(),
            worker = worker.display(),
        );
        fs::write(&config_path, body).unwrap();
        let config = RepoEvolverConfig::load(&config_path).unwrap();
        let hermetic = HermeticGitRunner::new(&origin);

        Self {
            _root: root,
            origin,
            checkout,
            state_dir,
            config,
            config_path,
            baseline_before,
            hermetic,
        }
    }

    fn runner(&self) -> HermeticGitRunner {
        self.hermetic.clone()
    }

    fn trusted_main_oid(&self) -> String {
        rev_parse(&self.checkout, "refs/heads/main")
    }

    fn trusted_git_dir(&self) -> PathBuf {
        let out = Command::new("git")
            .args([
                "-C",
                self.checkout.to_str().unwrap(),
                "rev-parse",
                "--absolute-git-dir",
            ])
            .output()
            .unwrap();
        PathBuf::from(String::from_utf8_lossy(&out.stdout).trim())
    }
}

fn run_git(cwd: &Path, args: &[&str]) {
    let status = Command::new("git")
        .args(args)
        .current_dir(cwd)
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .env("HOME", cwd)
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("LC_ALL", "C")
        .status()
        .unwrap();
    assert!(status.success(), "git {args:?} failed in {}", cwd.display());
}

fn git_config_identity(repo: &Path) {
    run_git(repo, &["config", "user.name", "Test"]);
    run_git(repo, &["config", "user.email", "test@gzmo.invalid"]);
}

fn rev_parse(repo: &Path, rev: &str) -> String {
    let out = Command::new("git")
        .args(["-C", repo.to_str().unwrap(), "rev-parse", rev])
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .env("HOME", "/tmp")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .output()
        .unwrap();
    assert!(out.status.success());
    String::from_utf8_lossy(&out.stdout).trim().to_owned()
}

fn fixed_now() -> chrono::DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 9, 1, 12, 0, 0).unwrap()
}

/// Copy committed `tests/fixtures/fixture-repo/**` into a seed working tree.
fn install_committed_fixture_tree(seed: &Path) {
    let src = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/fixture-repo");
    assert!(
        src.is_dir(),
        "missing committed fixture-repo at {}",
        src.display()
    );
    copy_tree(&src, seed);
    // Ensure mission script is executable for direct invocation; config still uses bash.
    let script = seed.join("scripts/opportunity-next-mission.sh");
    let mut perms = fs::metadata(&script).unwrap().permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&script, perms).unwrap();
}

fn copy_tree(from: &Path, to: &Path) {
    for entry in fs::read_dir(from).unwrap() {
        let entry = entry.unwrap();
        let name = entry.file_name();
        let src = entry.path();
        let dst = to.join(&name);
        let ft = entry.file_type().unwrap();
        if ft.is_dir() {
            fs::create_dir_all(&dst).unwrap();
            copy_tree(&src, &dst);
        } else if ft.is_file() {
            if let Some(parent) = dst.parent() {
                fs::create_dir_all(parent).unwrap();
            }
            fs::copy(&src, &dst).unwrap();
        }
    }
}

fn manifest_for(baseline: &str, id: &str) -> CandidateManifest {
    CandidateManifest {
        schema: CANDIDATE_SCHEMA.to_owned(),
        id: CandidateId::parse(id).unwrap(),
        mission_id: "felt-use-mass-growth".to_owned(),
        kind: CandidateKind::Code,
        authority: AuthorityTier::Candidate,
        target: CandidateTarget::Repository {
            owner: "maximilianwruhs-cyber".to_owned(),
            repository: "GZMO".to_owned(),
            base_branch: "main".to_owned(),
            candidate_branch: format!("evolve/{id}"),
        },
        baseline_digest: format!("git-sha1:{baseline}"),
        required_gates: vec![
            "format".to_owned(),
            "clippy".to_owned(),
            "tests".to_owned(),
            "opportunity-contract".to_owned(),
        ],
        protected_paths: vec![
            ".github/workflows/".to_owned(),
            "docs/superpowers/specs/".to_owned(),
            "docs/ADR-".to_owned(),
            "AGENTS.md".to_owned(),
            "Cargo.toml".to_owned(),
            "Cargo.lock".to_owned(),
            "crates/evolution-contracts/".to_owned(),
            "gzmo-evolver/".to_owned(),
            "config/repo-evolver.policy.toml".to_owned(),
        ],
        budget: ResourceBudget {
            wall_seconds: 2700,
            max_attempts: 1,
            max_changed_files: 20,
            max_added_lines: 1500,
            max_tool_calls: 80,
            max_input_tokens: 250_000,
            max_output_tokens: 50_000,
            max_energy_joules: None,
            allow_missing_energy_meter: true,
        },
        created_at: fixed_now(),
    }
}

fn install_mission_fake(fake: &FakeProcessRunner, clock: &ManualClock) {
    let clock_now = clock.now();
    fake.set_handler(move |spec| {
        let data_next = PathBuf::from(spec.env.get("GZMO_DATA_NEXT").expect("GZMO_DATA_NEXT"));
        let md = data_next.join("opportunity-discovery/next-mission.md");
        let json = data_next.join("opportunity-discovery/next-mission.json");
        fs::create_dir_all(md.parent().unwrap()).unwrap();
        fs::write(&md, FIXTURE_MARKDOWN).unwrap();
        let body = serde_json::json!({
            "schema": "gzmo.opportunity.next_mission/v1",
            "generated_at": clock_now.to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
            "ok": true,
            "bet_id": "felt-use-mass-growth",
            "title": "Felt Use mass growth",
            "score": 23,
            "ship_bar": true,
            "mission_md": md.to_str().unwrap(),
            "advice": "ok-advice-pad",
            "automation_note": "ok-note-pad"
        });
        fs::write(&json, body.to_string()).unwrap();
        Ok(ProcessOutput {
            status: 0,
            stdout: b"{}\n".to_vec(),
            stderr: Vec::new(),
        })
    });
}

fn install_mission_fake_with_marker(
    fake: &FakeProcessRunner,
    clock: &ManualClock,
    marker: PathBuf,
) {
    let clock_now = clock.now();
    fake.set_handler(move |spec| {
        fs::write(&marker, b"ran\n").unwrap();
        let data_next = PathBuf::from(spec.env.get("GZMO_DATA_NEXT").expect("GZMO_DATA_NEXT"));
        let md = data_next.join("opportunity-discovery/next-mission.md");
        let json = data_next.join("opportunity-discovery/next-mission.json");
        fs::create_dir_all(md.parent().unwrap()).unwrap();
        fs::write(&md, FIXTURE_MARKDOWN).unwrap();
        let body = serde_json::json!({
            "schema": "gzmo.opportunity.next_mission/v1",
            "generated_at": clock_now.to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
            "ok": true,
            "bet_id": "felt-use-mass-growth",
            "title": "Felt Use mass growth",
            "score": 23,
            "ship_bar": true,
            "mission_md": md.to_str().unwrap(),
            "advice": "ok-advice-pad",
            "automation_note": "ok-note-pad"
        });
        fs::write(&json, body.to_string()).unwrap();
        Ok(ProcessOutput {
            status: 0,
            stdout: b"{}\n".to_vec(),
            stderr: Vec::new(),
        })
    });
}

#[test]
fn independent_workspace_from_bare_remote() {
    let fx = Fixture::new();
    let runner = fx.runner();
    let git = GitRepository::open(&fx.config, &runner).unwrap();
    let baseline = git.refresh_and_resolve_baseline().unwrap();
    let policy = git
        .read_file_at(&baseline, "config/repo-evolver.policy.toml")
        .unwrap();
    let id = "cand-20260901t120000z-bet-01234567";
    let ws = git.prepare(&manifest_for(&baseline, id)).unwrap();
    assert_eq!(ws.current_branch().unwrap(), format!("evolve/{id}"));
    assert_eq!(ws.merge_base("HEAD", &baseline).unwrap(), baseline);
    assert_eq!(ws.fetch_url("origin").unwrap(), NO_FETCH_URL);
    assert_eq!(ws.push_url("origin").unwrap(), NO_PUSH_URL);
    assert_ne!(ws.git_dir().unwrap(), fx.trusted_git_dir());
    assert!(!ws.uses_alternates_or_shared_objects().unwrap());
    assert_eq!(fx.trusted_main_oid(), fx.baseline_before);
    assert!(!policy.is_empty());
    // Stored origin remains GitHub-shaped
    let origin = Command::new("git")
        .args([
            "-C",
            fx.checkout.to_str().unwrap(),
            "config",
            "--local",
            "--get",
            "remote.origin.url",
        ])
        .output()
        .unwrap();
    assert_eq!(
        String::from_utf8_lossy(&origin.stdout).trim(),
        GITHUB_ORIGIN
    );
}

#[test]
fn rejects_dirty_trusted_checkout() {
    let fx = Fixture::new();
    fs::write(fx.checkout.join("dirty.txt"), "x").unwrap();
    let runner = fx.runner();
    let git = GitRepository::open(&fx.config, &runner).unwrap();
    let err = git.refresh_and_resolve_baseline().unwrap_err();
    assert!(
        matches!(err, GitError::Trust(ref m) if m.contains("dirty")),
        "{err:?}"
    );
}

#[test]
fn rejects_non_baseline_checkout_head() {
    let fx = Fixture::new();
    fs::write(fx.checkout.join("extra.txt"), "y").unwrap();
    git_config_identity(&fx.checkout);
    run_git(&fx.checkout, &["add", "extra.txt"]);
    run_git(&fx.checkout, &["commit", "-m", "local only"]);
    let runner = fx.runner();
    let git = GitRepository::open(&fx.config, &runner).unwrap();
    let err = git.refresh_and_resolve_baseline().unwrap_err();
    assert!(matches!(err, GitError::Trust(_)), "{err:?}");
}

#[test]
fn rejects_embedded_credentials_wrong_host_and_local() {
    let err = validate_remote_identity(
        "https://user:sekrit@github.com/maximilianwruhs-cyber/GZMO.git",
        "maximilianwruhs-cyber",
        "GZMO",
    )
    .unwrap_err();
    let msg = err.to_string();
    assert!(!msg.contains("sekrit"), "{msg}");
    assert!(!msg.contains("user:"), "{msg}");

    let err = validate_remote_identity(
        "https://evil.example/maximilianwruhs-cyber/GZMO.git",
        "maximilianwruhs-cyber",
        "GZMO",
    )
    .unwrap_err();
    assert!(err.to_string().contains("github.com"), "{err}");

    let err =
        validate_remote_identity("/tmp/origin.git", "maximilianwruhs-cyber", "GZMO").unwrap_err();
    assert!(
        err.to_string().contains("local") || err.to_string().contains("not allowed"),
        "{err}"
    );
    let err = validate_remote_identity("file:///tmp/origin.git", "maximilianwruhs-cyber", "GZMO")
        .unwrap_err();
    assert!(
        err.to_string().contains("file") || err.to_string().contains("not allowed"),
        "{err}"
    );
}

#[test]
fn rejects_symlink_and_gitlink_in_baseline_tree() {
    let fx = Fixture::new();
    let seed = fx._root.path().join("seed2");
    run_git(
        fx._root.path(),
        &["clone", fx.origin.to_str().unwrap(), seed.to_str().unwrap()],
    );
    git_config_identity(&seed);
    run_git(&seed, &["checkout", "main"]);
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink("README.md", seed.join("link-readme")).unwrap();
        run_git(&seed, &["add", "link-readme"]);
        run_git(&seed, &["commit", "-m", "add symlink"]);
        run_git(&seed, &["push", "origin", "HEAD:main"]);
    }
    // Update checkout via hermetic rewrite path: fetch using insteadOf
    let runner = fx.runner();
    // reset checkout to new origin tip through raw git with file allow
    let status = Command::new("git")
        .args([
            "-C",
            fx.checkout.to_str().unwrap(),
            "-c",
            &format!(
                "url.file://{}.insteadOf={}",
                fx.origin.display(),
                GITHUB_ORIGIN
            ),
            "-c",
            "protocol.file.allow=always",
            "fetch",
            "origin",
        ])
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .env("HOME", "/tmp")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .status()
        .unwrap();
    assert!(status.success());
    run_git(&fx.checkout, &["reset", "--hard", "FETCH_HEAD"]);
    let git = GitRepository::open(&fx.config, &runner).unwrap();
    let err = git.refresh().unwrap_err();
    assert!(
        matches!(err, GitError::Trust(ref m) if m.contains("symlink") || m.contains("120000") || m.contains("special")),
        "{err:?}"
    );
}

#[test]
fn rejects_executable_hook_in_local_config() {
    let fx = Fixture::new();
    let hook = fx.checkout.join(".git/hooks/pre-commit");
    fs::write(&hook, "#!/bin/sh\nexit 0\n").unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut p = fs::metadata(&hook).unwrap().permissions();
        p.set_mode(0o755);
        fs::set_permissions(&hook, p).unwrap();
    }
    let runner = fx.runner();
    let git = GitRepository::open(&fx.config, &runner).unwrap();
    let err = git.refresh_and_resolve_baseline().unwrap_err();
    assert!(
        matches!(err, GitError::Trust(ref m) if m.contains("hook") || m.contains("executable")),
        "{err:?}"
    );
}

#[test]
fn rejects_existing_workspace_path_collision() {
    let fx = Fixture::new();
    let runner = fx.runner();
    let git = GitRepository::open(&fx.config, &runner).unwrap();
    let baseline = git.refresh_and_resolve_baseline().unwrap();
    let id = "cand-20260901t120000z-bet-aaaaaaa1";
    fs::create_dir_all(fx.state_dir.join(WORKSPACES_DIR).join(id)).unwrap();
    let err = git.prepare(&manifest_for(&baseline, id)).unwrap_err();
    assert!(
        matches!(err, GitError::Workspace(ref m) if m.contains("exists")),
        "{err:?}"
    );
}

#[test]
fn squash_diff_and_cleanup_happy_path() {
    let fx = Fixture::new();
    let runner = fx.runner();
    let git = GitRepository::open(&fx.config, &runner).unwrap();
    let baseline = git.refresh_and_resolve_baseline().unwrap();
    let id = "cand-20260901t120000z-bet-bbbbbbb2";
    let ws = git.prepare(&manifest_for(&baseline, id)).unwrap();

    fs::write(ws.path().join("worker.txt"), "change\n").unwrap();
    run_git(ws.path(), &["add", "worker.txt"]);
    run_git(
        ws.path(),
        &[
            "-c",
            "user.name=Worker",
            "-c",
            "user.email=w@x",
            "commit",
            "-m",
            "worker",
        ],
    );

    let new_oid = ws
        .squash_candidate(&baseline, "felt-use-mass-growth", fixed_now())
        .unwrap();
    assert_ne!(new_oid, baseline);
    assert_eq!(ws.candidate_commit().unwrap(), new_oid);

    let stats = ws
        .diff_stats(&baseline, &new_oid, &manifest_for(&baseline, id))
        .unwrap();
    assert!(!stats.files.is_empty());
    assert!(stats.added_lines >= 1);

    let store = StateStore::open(&fx.state_dir).unwrap();
    let rec = store
        .create_candidate(
            &manifest_for(&baseline, id),
            fx.config.working_policy_digest(),
            fixed_now(),
        )
        .unwrap();
    let rec = store
        .transition(
            rec.id(),
            CandidateState::Prepared,
            TransitionMetadata::empty().with_workspace(ws.path()),
            fixed_now(),
        )
        .unwrap();
    let rec = store
        .transition(
            rec.id(),
            CandidateState::Failed,
            TransitionMetadata::terminal("test-done")
                .with_candidate_digest(format!("git-sha1:{new_oid}")),
            fixed_now(),
        )
        .unwrap();
    cleanup_workspace(&fx.state_dir, &rec, None).unwrap();
    assert!(!ws.path().exists());
    assert_eq!(fx.trusted_main_oid(), fx.baseline_before);
}

#[test]
fn rejects_merge_commit_and_empty_candidate() {
    let fx = Fixture::new();
    let runner = fx.runner();
    let git = GitRepository::open(&fx.config, &runner).unwrap();
    let baseline = git.refresh_and_resolve_baseline().unwrap();
    let id = "cand-20260901t120000z-bet-cccccccc";
    let ws = git.prepare(&manifest_for(&baseline, id)).unwrap();

    let err = ws
        .squash_candidate(&baseline, "felt-use-mass-growth", fixed_now())
        .unwrap_err();
    assert!(
        matches!(err, GitError::Workspace(ref m) if m.contains("no changes")),
        "{err:?}"
    );

    fs::write(ws.path().join("a.txt"), "a\n").unwrap();
    run_git(ws.path(), &["add", "a.txt"]);
    run_git(
        ws.path(),
        &[
            "-c",
            "user.name=W",
            "-c",
            "user.email=w@x",
            "commit",
            "-m",
            "a",
        ],
    );
    run_git(ws.path(), &["checkout", "-b", "side"]);
    fs::write(ws.path().join("b.txt"), "b\n").unwrap();
    run_git(ws.path(), &["add", "b.txt"]);
    run_git(
        ws.path(),
        &[
            "-c",
            "user.name=W",
            "-c",
            "user.email=w@x",
            "commit",
            "-m",
            "b",
        ],
    );
    run_git(ws.path(), &["checkout", &format!("evolve/{id}")]);
    let status = Command::new("git")
        .args([
            "-C",
            ws.path().to_str().unwrap(),
            "-c",
            "user.name=W",
            "-c",
            "user.email=w@x",
            "merge",
            "--no-ff",
            "-m",
            "merge",
            "side",
        ])
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .env("HOME", "/tmp")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .status()
        .unwrap();
    assert!(status.success());
    let err = ws
        .squash_candidate(&baseline, "felt-use-mass-growth", fixed_now())
        .unwrap_err();
    assert!(
        matches!(err, GitError::Workspace(ref m) if m.contains("merge")),
        "{err:?}"
    );
}

#[test]
fn rejects_dirty_workspace_on_squash() {
    let fx = Fixture::new();
    let runner = fx.runner();
    let git = GitRepository::open(&fx.config, &runner).unwrap();
    let baseline = git.refresh_and_resolve_baseline().unwrap();
    let id = "cand-20260901t120000z-bet-dddddddd";
    let ws = git.prepare(&manifest_for(&baseline, id)).unwrap();
    fs::write(ws.path().join("untracked.txt"), "u").unwrap();
    let err = ws
        .squash_candidate(&baseline, "felt-use-mass-growth", fixed_now())
        .unwrap_err();
    assert!(
        matches!(err, GitError::Workspace(ref m) if m.contains("dirty")),
        "{err:?}"
    );
}

#[test]
fn rejects_cleanup_mismatch_and_nonterminal() {
    let fx = Fixture::new();
    let runner = fx.runner();
    let git = GitRepository::open(&fx.config, &runner).unwrap();
    let baseline = git.refresh_and_resolve_baseline().unwrap();
    let id = "cand-20260901t120000z-bet-eeeeeeee";
    let ws = git.prepare(&manifest_for(&baseline, id)).unwrap();
    let store = StateStore::open(&fx.state_dir).unwrap();
    let rec = store
        .create_candidate(
            &manifest_for(&baseline, id),
            fx.config.working_policy_digest(),
            fixed_now(),
        )
        .unwrap();
    let rec = store
        .transition(
            rec.id(),
            CandidateState::Prepared,
            TransitionMetadata::empty().with_workspace(ws.path()),
            fixed_now(),
        )
        .unwrap();
    let err = cleanup_workspace(&fx.state_dir, &rec, None).unwrap_err();
    assert!(
        matches!(err, GitError::Invalid(ref m) if m.contains("terminal")),
        "{err:?}"
    );
    let rec = store
        .transition(
            rec.id(),
            CandidateState::Failed,
            TransitionMetadata::terminal("x"),
            fixed_now(),
        )
        .unwrap();
    let err =
        cleanup_workspace(&fx.state_dir, &rec, Some(Path::new("/tmp/not-the-ws"))).unwrap_err();
    assert!(matches!(err, GitError::Invalid(_)), "{err:?}");
}

#[test]
fn prepare_active_first_and_failure_to_failed() {
    let fx = Fixture::new();
    let clock = ManualClock::new(fixed_now());
    let fake = FakeProcessRunner::new();
    install_mission_fake(&fake, &clock);
    let hybrid = HybridRunner {
        git: fx.runner(),
        fake_mission: fake,
    };
    let _lock = CoordinatorLock::try_acquire(&fx.state_dir).unwrap();
    let store = StateStore::open(&fx.state_dir).unwrap();

    let outcome = prepare_candidate(&fx.config, &hybrid, &clock, &store).unwrap();
    assert!(!outcome.reused_active);
    assert_eq!(outcome.record.state(), CandidateState::Prepared);
    assert!(outcome.record.workspace().is_some());
    let first_id = outcome.record.id().as_str().to_owned();

    let before: Vec<_> = fs::read_dir(fx.state_dir.join(WORKSPACES_DIR))
        .unwrap()
        .map(|e| e.unwrap().file_name())
        .collect();
    let outcome2 = prepare_candidate(&fx.config, &hybrid, &clock, &store).unwrap();
    assert!(outcome2.reused_active);
    assert_eq!(outcome2.record.id().as_str(), first_id);
    let after: Vec<_> = fs::read_dir(fx.state_dir.join(WORKSPACES_DIR))
        .unwrap()
        .map(|e| e.unwrap().file_name())
        .collect();
    assert_eq!(before, after);

    store
        .transition(
            outcome2.record.id(),
            CandidateState::Failed,
            TransitionMetadata::terminal("done"),
            fixed_now(),
        )
        .unwrap();
    cleanup_workspace(
        &fx.state_dir,
        &store.load(outcome2.record.id()).unwrap(),
        None,
    )
    .ok();

    clock.advance_secs(60);
    let fake2 = FakeProcessRunner::new();
    fake2.set_handler(|_| {
        Err(ProcessError::NonZeroExit {
            code: 9,
            stdout: Vec::new(),
            stderr: b"boom".to_vec(),
        })
    });
    let hybrid_fail = HybridRunner {
        git: fx.runner(),
        fake_mission: fake2,
    };
    let err = prepare_candidate(&fx.config, &hybrid_fail, &clock, &store).unwrap_err();
    assert!(
        err.to_string().contains("mission") || err.to_string().contains("prepare"),
        "{err}"
    );
}

#[test]
fn standalone_refresh_baseline_then_adapter_without_state() {
    let fx = Fixture::new();
    assert!(!fx.state_dir.join("state.db").exists());
    let runner = fx.runner();
    let baseline = refresh_baseline_before_mission(&fx.config, &runner).unwrap();
    assert_eq!(baseline, fx.baseline_before);
    assert!(!fx.state_dir.join("state.db").exists());
    assert!(!fx.state_dir.join("runner.lock").exists());

    // Positive sequence: baseline then mission adapter with marker.
    let clock = ManualClock::new(fixed_now());
    let fake = FakeProcessRunner::new();
    let marker = fx.state_dir.join("producer-ran.marker");
    install_mission_fake_with_marker(&fake, &clock, marker.clone());
    let hybrid = HybridRunner {
        git: fx.runner(),
        fake_mission: fake,
    };
    let _ = refresh_baseline_before_mission(&fx.config, &hybrid).unwrap();
    let adapter = MissionAdapter::new(&fx.config, &hybrid, &clock);
    let _ = adapter.refresh_and_load().unwrap();
    assert!(
        marker.exists(),
        "producer marker must appear when adapter runs"
    );
    assert!(!fx.state_dir.join("state.db").exists());
}

#[test]
fn standalone_refresh_rejects_local_only_commit_without_running_producer() {
    let fx = Fixture::new();
    fs::write(fx.checkout.join("local-only.txt"), "x\n").unwrap();
    git_config_identity(&fx.checkout);
    run_git(&fx.checkout, &["add", "local-only.txt"]);
    run_git(&fx.checkout, &["commit", "-m", "local only"]);

    let marker = fx.state_dir.join("producer-ran.marker");
    let runner = fx.runner();
    let err = refresh_baseline_before_mission(&fx.config, &runner).unwrap_err();
    assert!(matches!(err, GitError::Trust(_)), "{err:?}");
    assert!(!marker.exists());
    assert!(!fx.state_dir.join("state.db").exists());
    let staging = fx.state_dir.join(MISSION_STAGING_DIR);
    assert!(
        !staging.exists()
            || fs::read_dir(&staging)
                .map(|d| d.count() == 0)
                .unwrap_or(true)
    );
}

#[test]
fn squash_dates_are_utc_and_deterministic() {
    let fx = Fixture::new();
    let runner = fx.runner();
    let git = GitRepository::open(&fx.config, &runner).unwrap();
    let baseline = git.refresh_and_resolve_baseline().unwrap();
    let id = "cand-20260901t120000z-bet-tzcheck01";
    let ws = git.prepare(&manifest_for(&baseline, id)).unwrap();
    fs::write(ws.path().join("tz.txt"), "tz\n").unwrap();
    run_git(ws.path(), &["add", "tz.txt"]);
    run_git(
        ws.path(),
        &[
            "-c",
            "user.name=W",
            "-c",
            "user.email=w@x",
            "commit",
            "-m",
            "w",
        ],
    );
    let now = fixed_now();
    let oid1 = ws
        .squash_candidate(&baseline, "felt-use-mass-growth", now)
        .unwrap();
    let body = Command::new("git")
        .args([
            "-C",
            ws.path().to_str().unwrap(),
            "cat-file",
            "commit",
            &oid1,
        ])
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .env("HOME", "/tmp")
        .output()
        .unwrap();
    let text = String::from_utf8_lossy(&body.stdout);
    for line in text.lines() {
        if line.starts_with("author ") || line.starts_with("committer ") {
            assert!(line.ends_with("+0000"), "{line}");
        }
    }

    let id2 = "cand-20260901t120000z-bet-tzcheck02";
    let ws2 = git.prepare(&manifest_for(&baseline, id2)).unwrap();
    fs::write(ws2.path().join("tz.txt"), "tz\n").unwrap();
    run_git(ws2.path(), &["add", "tz.txt"]);
    run_git(
        ws2.path(),
        &[
            "-c",
            "user.name=W",
            "-c",
            "user.email=w@x",
            "commit",
            "-m",
            "w",
        ],
    );
    let oid2 = ws2
        .squash_candidate(&baseline, "felt-use-mass-growth", now)
        .unwrap();
    assert_eq!(oid1, oid2);
}

#[test]
fn rejects_shared_objects_when_clone_omits_no_local() {
    let fx = Fixture::new();
    let base = fx.runner();
    let git = GitRepository::open(&fx.config, &base).unwrap();
    let baseline = git.refresh_and_resolve_baseline().unwrap();
    let status = Command::new("git")
        .args([
            "--git-dir",
            git.mirror_path().to_str().unwrap(),
            "repack",
            "-ad",
        ])
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .env("HOME", "/tmp")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .status()
        .unwrap();
    assert!(status.success());

    let bad = StripNoLocalRunner { inner: fx.runner() };
    let git_bad = GitRepository::open(&fx.config, &bad).unwrap();
    let id_bad = "cand-20260901t120000z-bet-sharebad1";
    let err = git_bad
        .prepare(&manifest_for(&baseline, id_bad))
        .unwrap_err();
    assert!(
        matches!(err, GitError::Workspace(ref m) if m.contains("share") || m.contains("hardlink") || m.contains("inode")),
        "{err:?}"
    );
    assert!(!fx.state_dir.join(WORKSPACES_DIR).join(id_bad).exists());

    let id_ok = "cand-20260901t120000z-bet-sharenok1";
    let mut ws = None;
    for attempt in 0..8 {
        match git.prepare(&manifest_for(&baseline, id_ok)) {
            Ok(w) => {
                ws = Some(w);
                break;
            }
            Err(GitError::MirrorLockBusy) => {
                std::thread::sleep(std::time::Duration::from_millis(20 + attempt * 10));
            }
            Err(e) => panic!("unexpected prepare error: {e:?}"),
        }
    }
    let ws = ws.expect("prepare ok after MirrorLockBusy retries exhausted");
    assert!(!ws.uses_alternates_or_shared_objects().unwrap());
}

#[test]
fn prepare_reports_mirror_lock_busy_when_lease_held() {
    let fx = Fixture::new();
    let runner = fx.runner();
    let git = GitRepository::open(&fx.config, &runner).unwrap();
    let baseline = git.refresh_and_resolve_baseline().unwrap();
    let lock_path = fx.state_dir.join(MIRROR_LOCK_NAME);
    let file = OpenOptions::new()
        .create(true)
        .read(true)
        .write(true)
        .open(&lock_path)
        .unwrap();
    file.try_lock_exclusive().unwrap();
    let id = "cand-20260901t120000z-bet-lockbusy1";
    let err = git.prepare(&manifest_for(&baseline, id)).unwrap_err();
    assert!(matches!(err, GitError::MirrorLockBusy), "{err:?}");
    drop(file);
}

#[test]
fn git_timeout_and_cap_constants_are_plan_exact() {
    assert_eq!(GIT_FETCH_TIMEOUT_SECS, 900);
    assert_eq!(GIT_TIMEOUT_SECS, 120);
    assert_eq!(GIT_OUTPUT_CAP_BYTES, 8 * 1024 * 1024);
}

#[test]
fn product_rejects_local_origin_without_hermetic_rewrite() {
    let fx = Fixture::new();
    // Point origin at local bare path — product must reject.
    run_git(
        &fx.checkout,
        &["remote", "set-url", "origin", fx.origin.to_str().unwrap()],
    );
    let runner = SystemProcessRunner; // no rewrite
    let git = GitRepository::open(&fx.config, &runner).unwrap();
    let err = git.refresh_and_resolve_baseline().unwrap_err();
    assert!(
        matches!(err, GitError::Trust(ref m) if m.contains("local") || m.contains("not allowed") || m.contains("parsed")),
        "{err:?}"
    );
}

#[test]
fn policy_mismatch_is_rejected() {
    let fx = Fixture::new();
    let mut policy = POLICY_TOML.to_owned();
    policy = policy.replace("max_added_lines = 1500", "max_added_lines = 1400");
    fs::write(fx.checkout.join("config/repo-evolver.policy.toml"), &policy).unwrap();
    let config = RepoEvolverConfig::load(&fx.config_path).unwrap();
    let runner = fx.runner();
    let git = GitRepository::open(&config, &runner).unwrap();
    let err = git.refresh_and_resolve_baseline().unwrap_err();
    assert!(matches!(err, GitError::Trust(_)), "{err:?}");
}

#[test]
fn diff_rejects_protected_path_change() {
    let fx = Fixture::new();
    let runner = fx.runner();
    let git = GitRepository::open(&fx.config, &runner).unwrap();
    let baseline = git.refresh_and_resolve_baseline().unwrap();
    let id = "cand-20260901t120000z-bet-gggggggg";
    let ws = git.prepare(&manifest_for(&baseline, id)).unwrap();
    fs::create_dir_all(ws.path().join("gzmo-evolver")).unwrap();
    fs::write(ws.path().join("gzmo-evolver/hack.rs"), "x\n").unwrap();
    run_git(ws.path(), &["add", "gzmo-evolver/hack.rs"]);
    run_git(
        ws.path(),
        &[
            "-c",
            "user.name=W",
            "-c",
            "user.email=w@x",
            "commit",
            "-m",
            "hack",
        ],
    );
    let head = ws.candidate_commit().unwrap();
    let err = ws
        .diff_stats(&baseline, &head, &manifest_for(&baseline, id))
        .unwrap_err();
    assert!(
        matches!(err, GitError::Invalid(ref m) if m.contains("path policy") || m.contains("protected")),
        "{err:?}"
    );
}

#[test]
fn public_commands_require_config_hidden_worker_does_not() {
    // Build the binary path from the test exe location.
    let mut bin = std::env::current_exe().unwrap();
    bin.pop(); // deps
    if bin.ends_with("deps") {
        bin.pop();
    }
    bin.push("gzmo-evolver");
    assert!(bin.is_file(), "missing binary {}", bin.display());

    // Public command without --config must fail.
    let out = std::process::Command::new(&bin)
        .args(["status"])
        .output()
        .unwrap();
    assert!(!out.status.success(), "status without --config must fail");
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(
        err.contains("--config") || err.contains("required"),
        "stderr={err}"
    );

    // Hidden worker without --config should fail on request path validation,
    // not on missing config.
    let out = std::process::Command::new(&bin)
        .args(["worker", "--request", "/tmp/no-such-request.json"])
        .output()
        .unwrap();
    assert!(!out.status.success());
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(
        !err.contains("--config is required"),
        "worker must not require --config; stderr={err}"
    );

    // Stronger separation: supplying --config on the hidden worker hard-fails.
    let out = std::process::Command::new(&bin)
        .args([
            "--config",
            "/tmp/no-such-config.toml",
            "worker",
            "--request",
            "/tmp/no-such-request.json",
        ])
        .output()
        .unwrap();
    assert!(!out.status.success());
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(
        err.contains("must not be invoked with --config"),
        "worker with --config must hard-fail; stderr={err}"
    );
}

// --- Task 6 runner vertical / crash-resume harness ----------------------------

use async_trait::async_trait;
use evolution_contracts::{sha256_hex, ResourceUsage};
use gzmo_evolver::{
    worker_runtime_dirs, EffectiveIdentity, PrepareOutcome, RepoEvolver, RunOutcome, WorkerError,
    WorkerIdentity, WorkerLauncher, WorkerReceipt, WorkerRequest, WorkerRoots,
    WorkerRuntimeProvisioner, WorkerUnitState,
};
use std::os::unix::fs::PermissionsExt;
use std::sync::{Arc, Mutex};

/// Fake fixed worker identity = real euid (hermetic, no chown).
struct TestWorkerIdentity {
    uid: u32,
    gid: u32,
}

impl WorkerIdentity for TestWorkerIdentity {
    fn identity(&self) -> Result<EffectiveIdentity, gzmo_evolver::RunnerError> {
        Ok(EffectiveIdentity {
            uid: self.uid,
            gid: self.gid,
        })
    }
}

/// Creates the six runtime dirs as the current user (0700).
struct FakeRuntimeProvisioner {
    roots: WorkerRoots,
    profile: String,
    calls: Arc<Mutex<u32>>,
}

#[async_trait]
impl WorkerRuntimeProvisioner for FakeRuntimeProvisioner {
    async fn provision(&self, candidate_id: &CandidateId) -> Result<(), WorkerError> {
        *self.calls.lock().unwrap() += 1;
        let dirs = worker_runtime_dirs(
            self.roots.output_root(),
            candidate_id.as_str(),
            &self.profile,
        )?;
        for d in &dirs {
            fs::create_dir_all(d).map_err(|e| WorkerError::Io(e.to_string()))?;
            let mut perms = fs::metadata(d)
                .map_err(|e| WorkerError::Io(e.to_string()))?
                .permissions();
            perms.set_mode(0o700);
            fs::set_permissions(d, perms).map_err(|e| WorkerError::Io(e.to_string()))?;
        }
        Ok(())
    }
}

/// Controllable hermetic launcher modes for crash-matrix coverage.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LaunchMode {
    /// Mutate workspace, write correct receipt, succeed.
    Happy,
    /// Happy path then also create the exact normalized commit (post-squash crash simulation).
    PostSquashNormalize,
    /// Write receipt with changed_files/added_lines = 0 (fact mismatch).
    FactMismatch,
    /// Leave unit Succeeded but write no receipt.
    SucceededNoReceipt,
    /// Leave unit Failed, no receipt.
    UnitFailed,
    /// Stay Running forever (wait path).
    StayRunning,
    /// Record stop order for abort tests.
    TrackStop,
    /// Empty baseline-tree normalized signature + zero usage (must terminalize).
    EmptyNormalized,
    /// Valid commit+receipt then leave untracked dirty file.
    DirtyAfterReceipt,
}

/// Hermetic launcher: mutates workspace, writes receipt/raw, tracks launch count + env names.
struct FakeStageLauncher {
    launches: Arc<Mutex<u32>>,
    unit_state: Arc<Mutex<WorkerUnitState>>,
    mode: Arc<Mutex<LaunchMode>>,
    stop_log: Arc<Mutex<Vec<&'static str>>>,
    env_names: Arc<Mutex<Vec<String>>>,
    now: chrono::DateTime<Utc>,
}

impl FakeStageLauncher {
    fn write_worker_commit(ws: &Path) -> Result<String, WorkerError> {
        fs::write(ws.join("worker-change.txt"), "fake-worker-change\n")
            .map_err(|e| WorkerError::Io(e.to_string()))?;
        let status = Command::new("git")
            .args(["-C", ws.to_str().unwrap(), "add", "-A"])
            .env_clear()
            .env("PATH", "/usr/bin:/bin")
            .env("HOME", ws)
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env("GIT_CONFIG_GLOBAL", "/dev/null")
            .status()
            .map_err(|e| WorkerError::Io(e.to_string()))?;
        if !status.success() {
            return Err(WorkerError::Process("git add failed".into()));
        }
        let status = Command::new("git")
            .args([
                "-C",
                ws.to_str().unwrap(),
                "-c",
                "user.name=fake",
                "-c",
                "user.email=fake@worker",
                "commit",
                "-m",
                "fake-worker",
            ])
            .env_clear()
            .env("PATH", "/usr/bin:/bin")
            .env("HOME", ws)
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env("GIT_CONFIG_GLOBAL", "/dev/null")
            .status()
            .map_err(|e| WorkerError::Io(e.to_string()))?;
        if !status.success() {
            return Err(WorkerError::Process("git commit failed".into()));
        }
        Ok(rev_parse(ws, "HEAD"))
    }

    fn write_receipt(
        &self,
        request: &WorkerRequest,
        head: &str,
        changed_files: u32,
        added_lines: u32,
    ) -> Result<(), WorkerError> {
        let raw = br#"{"type":"session","version":3,"id":"fake-session"}
{"type":"tool_execution_start","toolCallId":"tool-1","toolName":"bash","args":{}}
{"type":"tool_execution_end","toolCallId":"tool-1","toolName":"bash","result":"ok"}
{"type":"message_end","message":{"role":"assistant","stopReason":"stop","usage":{"input":10,"output":5,"cacheRead":1,"cacheWrite":2,"totalTokens":18}}}
{"type":"agent_end","messages":[]}
"#;
        let out_dir = request.output_dir();
        fs::write(out_dir.join("raw.jsonl"), raw).map_err(|e| WorkerError::Io(e.to_string()))?;
        let output_digest = format!("sha256:{}", sha256_hex(raw));
        let started = request.issued_at() + chrono::Duration::seconds(1);
        let completed = started + chrono::Duration::seconds(2);
        let _ = self.now;
        let usage = ResourceUsage {
            wall_seconds: 2,
            attempts: 1,
            changed_files,
            added_lines,
            tool_calls: 1,
            input_tokens: 10,
            output_tokens: 5,
            energy_joules: None,
        };
        let receipt = WorkerReceipt::new(
            request.candidate_id().clone(),
            request.manifest_digest(),
            request.policy_digest(),
            request.omp_version(),
            started,
            completed,
            0,
            output_digest,
            Some(format!("git-sha1:{head}")),
            usage,
        )?;
        let bytes = receipt.canonical_bytes()?;
        fs::write(out_dir.join("receipt.json"), &bytes)
            .map_err(|e| WorkerError::Io(e.to_string()))?;
        let mut perms = fs::metadata(out_dir.join("receipt.json"))
            .unwrap()
            .permissions();
        perms.set_mode(0o600);
        fs::set_permissions(out_dir.join("receipt.json"), perms).unwrap();
        Ok(())
    }

    fn normalize_like_coordinator(
        ws: &Path,
        baseline: &str,
        mission_id: &str,
        completed_at: chrono::DateTime<Utc>,
    ) -> Result<String, WorkerError> {
        // commit-tree path similar to product squash for post-squash crash simulation.
        let tree = {
            let out = Command::new("git")
                .args(["-C", ws.to_str().unwrap(), "rev-parse", "HEAD^{tree}"])
                .env_clear()
                .env("PATH", "/usr/bin:/bin")
                .env("HOME", ws)
                .env("GIT_CONFIG_NOSYSTEM", "1")
                .env("GIT_CONFIG_GLOBAL", "/dev/null")
                .output()
                .map_err(|e| WorkerError::Io(e.to_string()))?;
            String::from_utf8_lossy(&out.stdout).trim().to_owned()
        };
        let ts = format!("{} +0000", completed_at.timestamp());
        let msg = format!("evolve({mission_id}): candidate");
        let out = Command::new("git")
            .args([
                "-C",
                ws.to_str().unwrap(),
                "commit-tree",
                &tree,
                "-p",
                baseline,
                "-m",
                &msg,
            ])
            .env_clear()
            .env("PATH", "/usr/bin:/bin")
            .env("HOME", ws)
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env("GIT_CONFIG_GLOBAL", "/dev/null")
            .env("GIT_AUTHOR_NAME", "GZMO Evolver Candidate")
            .env("GIT_AUTHOR_EMAIL", "candidate@gzmo.invalid")
            .env("GIT_AUTHOR_DATE", &ts)
            .env("GIT_COMMITTER_NAME", "GZMO Evolver Candidate")
            .env("GIT_COMMITTER_EMAIL", "candidate@gzmo.invalid")
            .env("GIT_COMMITTER_DATE", &ts)
            .output()
            .map_err(|e| WorkerError::Io(e.to_string()))?;
        if !out.status.success() {
            return Err(WorkerError::Process("commit-tree failed".into()));
        }
        let new_commit = String::from_utf8_lossy(&out.stdout).trim().to_owned();
        let old = rev_parse(ws, "HEAD");
        let branch = {
            let out = Command::new("git")
                .args([
                    "-C",
                    ws.to_str().unwrap(),
                    "rev-parse",
                    "--abbrev-ref",
                    "HEAD",
                ])
                .env_clear()
                .env("PATH", "/usr/bin:/bin")
                .env("HOME", ws)
                .env("GIT_CONFIG_NOSYSTEM", "1")
                .env("GIT_CONFIG_GLOBAL", "/dev/null")
                .output()
                .unwrap();
            String::from_utf8_lossy(&out.stdout).trim().to_owned()
        };
        let status = Command::new("git")
            .args([
                "-C",
                ws.to_str().unwrap(),
                "update-ref",
                &format!("refs/heads/{branch}"),
                &new_commit,
                &old,
            ])
            .env_clear()
            .env("PATH", "/usr/bin:/bin")
            .env("HOME", ws)
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env("GIT_CONFIG_GLOBAL", "/dev/null")
            .status()
            .map_err(|e| WorkerError::Io(e.to_string()))?;
        if !status.success() {
            return Err(WorkerError::Process("update-ref failed".into()));
        }
        Ok(new_commit)
    }
}

#[async_trait]
impl WorkerLauncher for FakeStageLauncher {
    async fn launch_and_wait(
        &self,
        request_path: &Path,
        request: &WorkerRequest,
        _roots: &WorkerRoots,
    ) -> Result<(), WorkerError> {
        {
            let mut st = self.unit_state.lock().unwrap();
            if *st == WorkerUnitState::Running || *st == WorkerUnitState::Succeeded {
                return Err(WorkerError::Invalid("refusing duplicate launch".to_owned()));
            }
            *st = WorkerUnitState::Running;
        }
        *self.launches.lock().unwrap() += 1;
        let mode = *self.mode.lock().unwrap();
        let _ = request_path;
        // Record exact allowlisted env *names* the production OMP child would receive.
        let home = request.output_dir().join(gzmo_evolver::WORKER_HOME_NAME);
        let env = gzmo_evolver::omp_child_env(&home)
            .map_err(|e| WorkerError::Invalid(format!("omp_child_env: {e}")))?;
        let mut names: Vec<String> = env.keys().cloned().collect();
        names.sort();
        *self.env_names.lock().unwrap() = names;

        match mode {
            LaunchMode::StayRunning => {
                return Ok(());
            }
            LaunchMode::UnitFailed => {
                *self.unit_state.lock().unwrap() = WorkerUnitState::Failed;
                return Ok(());
            }
            LaunchMode::SucceededNoReceipt => {
                let _ = Self::write_worker_commit(request.workspace())?;
                *self.unit_state.lock().unwrap() = WorkerUnitState::Succeeded;
                return Ok(());
            }
            LaunchMode::EmptyNormalized => {
                // Commit-tree baseline tree with normalized signature; zero usage.
                let baseline = rev_parse(request.workspace(), "HEAD");
                let man = request_path.parent().unwrap().join("manifest.json");
                let v: serde_json::Value =
                    serde_json::from_slice(&fs::read(&man).unwrap()).unwrap();
                let mission_id = v["mission_id"].as_str().unwrap().to_owned();
                let completed = request.issued_at() + chrono::Duration::seconds(2);
                let empty_head = Self::normalize_like_coordinator(
                    request.workspace(),
                    &baseline,
                    &mission_id,
                    completed,
                )?;
                self.write_receipt(request, &empty_head, 0, 0)?;
                // Rewrite receipt completed_at to match normalize timestamp.
                let started = request.issued_at() + chrono::Duration::seconds(1);
                let raw = fs::read(request.output_dir().join("raw.jsonl")).unwrap();
                let output_digest = format!("sha256:{}", sha256_hex(&raw));
                let usage = ResourceUsage {
                    wall_seconds: 2,
                    attempts: 1,
                    changed_files: 0,
                    added_lines: 0,
                    tool_calls: 1,
                    input_tokens: 10,
                    output_tokens: 5,
                    energy_joules: None,
                };
                let receipt = WorkerReceipt::new(
                    request.candidate_id().clone(),
                    request.manifest_digest(),
                    request.policy_digest(),
                    request.omp_version(),
                    started,
                    completed,
                    0,
                    output_digest,
                    Some(format!("git-sha1:{empty_head}")),
                    usage,
                )?;
                fs::write(
                    request.output_dir().join("receipt.json"),
                    receipt.canonical_bytes()?,
                )
                .unwrap();
                *self.unit_state.lock().unwrap() = WorkerUnitState::Succeeded;
                return Ok(());
            }
            LaunchMode::Happy
            | LaunchMode::PostSquashNormalize
            | LaunchMode::FactMismatch
            | LaunchMode::TrackStop
            | LaunchMode::DirtyAfterReceipt => {}
        }

        let head = Self::write_worker_commit(request.workspace())?;
        let (files, lines) = match mode {
            LaunchMode::FactMismatch => (0, 0),
            _ => (1, 1),
        };
        self.write_receipt(request, &head, files, lines)?;

        if mode == LaunchMode::PostSquashNormalize {
            let baseline = {
                let out = Command::new("git")
                    .args([
                        "-C",
                        request.workspace().to_str().unwrap(),
                        "rev-parse",
                        "HEAD^",
                    ])
                    .env_clear()
                    .env("PATH", "/usr/bin:/bin")
                    .env("HOME", request.workspace())
                    .env("GIT_CONFIG_NOSYSTEM", "1")
                    .env("GIT_CONFIG_GLOBAL", "/dev/null")
                    .output()
                    .unwrap();
                String::from_utf8_lossy(&out.stdout).trim().to_owned()
            };
            let mission_id = {
                let man = request_path.parent().unwrap().join("manifest.json");
                let v: serde_json::Value = serde_json::from_slice(&fs::read(man).unwrap()).unwrap();
                v["mission_id"].as_str().unwrap().to_owned()
            };
            let completed = request.issued_at() + chrono::Duration::seconds(3);
            let _ = Self::normalize_like_coordinator(
                request.workspace(),
                &baseline,
                &mission_id,
                completed,
            )?;
            let started = request.issued_at() + chrono::Duration::seconds(1);
            let raw = fs::read(request.output_dir().join("raw.jsonl")).unwrap();
            let output_digest = format!("sha256:{}", sha256_hex(&raw));
            let usage = ResourceUsage {
                wall_seconds: 2,
                attempts: 1,
                changed_files: 1,
                added_lines: 1,
                tool_calls: 1,
                input_tokens: 10,
                output_tokens: 5,
                energy_joules: None,
            };
            let receipt = WorkerReceipt::new(
                request.candidate_id().clone(),
                request.manifest_digest(),
                request.policy_digest(),
                request.omp_version(),
                started,
                completed,
                0,
                output_digest,
                Some(format!("git-sha1:{head}")),
                usage,
            )?;
            let bytes = receipt.canonical_bytes()?;
            fs::write(request.output_dir().join("receipt.json"), bytes).unwrap();
        }

        if mode == LaunchMode::DirtyAfterReceipt {
            fs::write(request.workspace().join("untracked-dirt.txt"), "x\n").unwrap();
        }

        *self.unit_state.lock().unwrap() = WorkerUnitState::Succeeded;
        Ok(())
    }

    async fn inspect(&self, _candidate_id: &CandidateId) -> Result<WorkerUnitState, WorkerError> {
        Ok(*self.unit_state.lock().unwrap())
    }

    async fn wait_existing(
        &self,
        _candidate_id: &CandidateId,
        _deadline: chrono::DateTime<Utc>,
    ) -> Result<WorkerUnitState, WorkerError> {
        Ok(*self.unit_state.lock().unwrap())
    }

    async fn stop(&self, _candidate_id: &CandidateId) -> Result<(), WorkerError> {
        self.stop_log.lock().unwrap().push("stop");
        *self.unit_state.lock().unwrap() = WorkerUnitState::NotFound;
        self.stop_log.lock().unwrap().push("inactive");
        Ok(())
    }
}

/// Combined process runner: hermetic git + mission fake + omp --version.
struct RunnerHybrid {
    git: HermeticGitRunner,
    fake_mission: FakeProcessRunner,
    omp_version: String,
}

impl ProcessRunner for RunnerHybrid {
    fn run(&self, spec: &ProcessSpec) -> Result<ProcessOutput, ProcessError> {
        let prog = spec.program.to_string_lossy();
        if prog.ends_with("git") || prog == "git" {
            return self.git.run(spec);
        }
        if prog.ends_with("bash") || prog == "bash" || prog == "/bin/bash" {
            return self.fake_mission.run(spec);
        }
        // OMP executable probe / fake.
        if spec.args.iter().any(|a| a == "--version") {
            return Ok(ProcessOutput {
                status: 0,
                stdout: format!("{}\n", self.omp_version).into_bytes(),
                stderr: Vec::new(),
            });
        }
        self.git.run(spec)
    }
}
struct RepoHarness {
    fixture: Fixture,
    evolver: RepoEvolver<
        RunnerHybrid,
        FakeStageLauncher,
        FakeRuntimeProvisioner,
        TestWorkerIdentity,
        ManualClock,
    >,
    store: StateStore,
    initial_main: String,
    roots: WorkerRoots,
    launches: Arc<Mutex<u32>>,
    provision_calls: Arc<Mutex<u32>>,
    unit_state: Arc<Mutex<WorkerUnitState>>,
    launch_mode: Arc<Mutex<LaunchMode>>,
    stop_log: Arc<Mutex<Vec<&'static str>>>,
    env_names: Arc<Mutex<Vec<String>>>,
    omp_version: String,
    coordinator_uid: u32,
    worker_uid: u32,
    worker_gid: u32,
}
impl RepoHarness {
    async fn new() -> Self {
        let fixture = Fixture::new();
        let clock = ManualClock::new(fixed_now());
        let fake_mission = FakeProcessRunner::new();
        install_mission_fake(&fake_mission, &clock);

        let request_root = fixture.state_dir.join("run-requests");
        let output_root = fixture.state_dir.join("worker-out");
        let profile_root = fixture.state_dir.join("profiles");
        let netns = fixture.state_dir.join("netns");
        fs::create_dir_all(&request_root).unwrap();
        fs::create_dir_all(&output_root).unwrap();
        fs::create_dir_all(&profile_root).unwrap();
        fs::create_dir_all(&netns).unwrap();

        // Valid profile tree.
        let profile = fixture.config.worker().profile().to_owned();
        let profile_dir = profile_root.join(&profile);
        fs::create_dir_all(profile_dir.join("agent")).unwrap();
        fs::write(
            profile_dir.join("agent/config.yml"),
            "modelRoles:\n  code_candidate: local/code\n",
        )
        .unwrap();
        fs::write(
            profile_dir.join("agent/models.yml"),
            "providers:\n  local:\n    auth: none\n    baseUrl: http://127.0.0.1:9\n    models:\n      - id: code\n        maxTokens: 16384\n        contextWindow: 32768\n",
        )
        .unwrap();
        for walk in [
            profile_dir.clone(),
            profile_dir.join("agent"),
            profile_dir.join("agent/config.yml"),
            profile_dir.join("agent/models.yml"),
        ] {
            let mut p = fs::metadata(&walk).unwrap().permissions();
            p.set_mode(if walk.is_dir() { 0o755 } else { 0o644 });
            fs::set_permissions(&walk, p).unwrap();
        }

        // Replace dummy omp with fake-worker that supports --version.
        let fixture_omp =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/fake-worker.sh");
        fs::copy(&fixture_omp, fixture.config.worker().executable()).unwrap();
        let mut perms = fs::metadata(fixture.config.worker().executable())
            .unwrap()
            .permissions();
        perms.set_mode(0o755);
        fs::set_permissions(fixture.config.worker().executable(), perms).unwrap();

        let roots = WorkerRoots::for_test(request_root, output_root, profile_root, netns).unwrap();
        let launches = Arc::new(Mutex::new(0u32));
        let provision_calls = Arc::new(Mutex::new(0u32));
        let unit_state = Arc::new(Mutex::new(WorkerUnitState::NotFound));
        let launch_mode = Arc::new(Mutex::new(LaunchMode::Happy));
        let stop_log = Arc::new(Mutex::new(Vec::new()));
        let env_names = Arc::new(Mutex::new(Vec::new()));

        let omp_version = "18.0.11".to_owned();
        let hybrid = RunnerHybrid {
            git: fixture.runner(),
            fake_mission,
            omp_version: omp_version.clone(),
        };
        let launcher = FakeStageLauncher {
            launches: launches.clone(),
            unit_state: unit_state.clone(),
            mode: launch_mode.clone(),
            stop_log: stop_log.clone(),
            env_names: env_names.clone(),
            now: fixed_now(),
        };
        let provisioner = FakeRuntimeProvisioner {
            roots: roots.clone(),
            profile: profile.clone(),
            calls: provision_calls.clone(),
        };

        let real_uid = nix::unistd::Uid::effective().as_raw();
        let real_gid = nix::unistd::Gid::effective().as_raw();
        let worker_uid = real_uid;
        let worker_gid = real_gid.max(1);
        let coordinator_uid = real_uid.wrapping_add(1000).max(1);
        assert_ne!(coordinator_uid, real_uid);

        let evolver = RepoEvolver::with_deps(
            fixture.config.clone(),
            hybrid,
            launcher,
            provisioner,
            TestWorkerIdentity {
                uid: worker_uid,
                gid: worker_gid,
            },
            ManualClock::new(fixed_now()),
            roots.clone(),
            coordinator_uid,
        )
        .unwrap();

        let store = StateStore::open(fixture.config.state_dir()).unwrap();
        let initial_main = fixture.baseline_before.clone();

        Self {
            fixture,
            evolver,
            store,
            initial_main,
            roots,
            launches,
            provision_calls,
            unit_state,
            launch_mode,
            stop_log,
            env_names,
            omp_version,
            coordinator_uid,
            worker_uid,
            worker_gid,
        }
    }

    fn remote_main(&self) -> String {
        rev_parse(&self.fixture.origin, "refs/heads/main")
    }

    fn initial_main(&self) -> String {
        self.initial_main.clone()
    }

    fn remote_branches(&self) -> Vec<String> {
        let out = Command::new("git")
            .args([
                "--git-dir",
                self.fixture.origin.to_str().unwrap(),
                "for-each-ref",
                "--format=%(refname:short)",
                "refs/heads",
            ])
            .env_clear()
            .env("PATH", "/usr/bin:/bin")
            .env("HOME", "/tmp")
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env("GIT_CONFIG_GLOBAL", "/dev/null")
            .output()
            .unwrap();
        assert!(out.status.success());
        String::from_utf8_lossy(&out.stdout)
            .lines()
            .map(|s| s.trim().to_owned())
            .filter(|s| !s.is_empty())
            .collect()
    }

    fn audit_states(&self) -> Vec<String> {
        let events = self.store.load_audit_events().unwrap();
        events
            .into_iter()
            .filter_map(|e| {
                e.event_type
                    .strip_prefix("candidate.")
                    .map(|s| s.to_owned())
            })
            .collect()
    }

    fn trusted_main(&self) -> String {
        self.fixture.trusted_main_oid()
    }
}

/// Bounded retry for LockBusy caused by fork/exec lock-fd inheritance under parallel tests.
/// Never used by production code or the dedicated lock-race test.
/// Retries only exact `RunnerError::LockBusy`; every other error breaks out immediately.
async fn run_once_retry_lock_busy<R, C>(
    evolver: &RepoEvolver<R, FakeStageLauncher, FakeRuntimeProvisioner, TestWorkerIdentity, C>,
) -> Result<RunOutcome, gzmo_evolver::RunnerError>
where
    R: ProcessRunner + Send + Sync + 'static,
    C: Clock + Send + Sync + 'static,
{
    let mut last = None;
    for attempt in 0..8 {
        match evolver.run_once().await {
            Ok(o) => return Ok(o),
            Err(gzmo_evolver::RunnerError::LockBusy) => {
                last = Some(gzmo_evolver::RunnerError::LockBusy);
                tokio::time::sleep(std::time::Duration::from_millis(25 + attempt * 15)).await;
            }
            Err(e) => return Err(e),
        }
    }
    Err(last.unwrap_or(gzmo_evolver::RunnerError::LockBusy))
}

async fn resume_retry_lock_busy<R>(
    evolver: &RepoEvolver<
        R,
        FakeStageLauncher,
        FakeRuntimeProvisioner,
        TestWorkerIdentity,
        ManualClock,
    >,
) -> Result<RunOutcome, gzmo_evolver::RunnerError>
where
    R: ProcessRunner + Send + Sync + 'static,
{
    let mut last = None;
    for attempt in 0..8 {
        match evolver.resume().await {
            Ok(o) => return Ok(o),
            Err(gzmo_evolver::RunnerError::LockBusy) => {
                last = Some(gzmo_evolver::RunnerError::LockBusy);
                tokio::time::sleep(std::time::Duration::from_millis(25 + attempt * 15)).await;
            }
            Err(e) => return Err(e),
        }
    }
    Err(last.unwrap_or(gzmo_evolver::RunnerError::LockBusy))
}

async fn abort_retry_lock_busy<R>(
    evolver: &RepoEvolver<
        R,
        FakeStageLauncher,
        FakeRuntimeProvisioner,
        TestWorkerIdentity,
        ManualClock,
    >,
    id: &str,
    reason: &str,
) -> Result<RunOutcome, gzmo_evolver::RunnerError>
where
    R: ProcessRunner + Send + Sync + 'static,
{
    let mut last = None;
    for attempt in 0..8 {
        match evolver.abort(id, reason).await {
            Ok(o) => return Ok(o),
            Err(gzmo_evolver::RunnerError::LockBusy) => {
                last = Some(gzmo_evolver::RunnerError::LockBusy);
                tokio::time::sleep(std::time::Duration::from_millis(25 + attempt * 15)).await;
            }
            Err(e) => return Err(e),
        }
    }
    Err(last.unwrap_or(gzmo_evolver::RunnerError::LockBusy))
}

/// Production-shaped sealed bundle for boundary seeds; version/uids from harness.
fn seal_for_test(harness: &RepoHarness, prep: &PrepareOutcome) -> WorkerRequest {
    let id = prep.record.id().clone();
    let ws = prep
        .record
        .workspace()
        .expect("prepared workspace")
        .to_path_buf();
    let dirs = worker_runtime_dirs(
        harness.roots.output_root(),
        id.as_str(),
        harness.fixture.config.worker().profile(),
    )
    .unwrap();
    for d in &dirs {
        fs::create_dir_all(d).unwrap();
        let mut p = fs::metadata(d).unwrap().permissions();
        p.set_mode(0o700);
        fs::set_permissions(d, p).unwrap();
    }
    let hybrid = HybridRunner {
        git: harness.fixture.runner(),
        fake_mission: {
            let f = FakeProcessRunner::new();
            install_mission_fake(&f, &ManualClock::new(fixed_now()));
            f
        },
    };
    let mission = MissionAdapter::new(
        &harness.fixture.config,
        &hybrid,
        &ManualClock::new(fixed_now()),
    )
    .load_current()
    .unwrap();
    let manifest_json = evolution_contracts::canonical_json_bytes(prep.record.manifest()).unwrap();
    let policy_rel = harness
        .fixture
        .config
        .policy()
        .repo_path()
        .to_str()
        .unwrap()
        .replace('\\', "/");
    let baseline = prep
        .record
        .manifest()
        .baseline_digest
        .strip_prefix("git-sha1:")
        .unwrap();
    let hermetic = harness.fixture.runner();
    let git = GitRepository::open(&harness.fixture.config, &hermetic).unwrap();
    let policy_toml = git.read_file_at(baseline, &policy_rel).unwrap();
    let system = gzmo_evolver::render_system_prompt(
        &id,
        &prep.record.manifest().baseline_digest,
        &ws,
        &prep.record.manifest().protected_paths,
        &prep.record.manifest().required_gates,
        &prep.record.manifest().budget,
    )
    .unwrap();
    let mission_md = gzmo_evolver::render_mission_prompt(&mission.markdown).unwrap();
    let overlay = gzmo_evolver::render_omp_overlay("local/code");
    let input = gzmo_evolver::SealWorkerInput {
        candidate_id: id,
        workspace: ws,
        output_dir: dirs[0].clone(),
        omp_executable: harness.fixture.config.worker().executable().to_path_buf(),
        omp_profile: harness.fixture.config.worker().profile().to_owned(),
        omp_version: harness.omp_version.clone(),
        coordinator_uid: harness.coordinator_uid,
        expected_uid: harness.worker_uid,
        expected_gid: harness.worker_gid,
        budget: prep.record.manifest().budget.clone(),
        issued_at: fixed_now(),
        companions: gzmo_evolver::WorkerCompanions {
            manifest_json,
            policy_toml,
            system_prompt_md: system.into_bytes(),
            mission_md: mission_md.into_bytes(),
            omp_overlay_yml: overlay.into_bytes(),
        },
        manifest_digest: prep.record.manifest_digest().to_owned(),
        policy_digest: prep.record.policy_digest().to_owned(),
    };
    gzmo_evolver::seal_worker_bundle(&harness.roots, input).unwrap()
}
#[tokio::test]
async fn one_run_stops_at_evaluation_boundary() {
    let harness = RepoHarness::new().await;
    let outcome = run_once_retry_lock_busy(&harness.evolver).await.unwrap();
    assert_eq!(outcome.state, CandidateState::Evaluating);
    assert!(outcome
        .candidate_digest
        .as_deref()
        .unwrap()
        .starts_with("git-sha1:"));
    assert_eq!(harness.remote_main(), harness.initial_main());
    assert!(harness.remote_branches().iter().all(|b| b == "main"));
    assert_eq!(harness.trusted_main(), harness.initial_main());
    assert!(harness.store.verify_audit_chain().is_ok());
    assert_eq!(
        harness.audit_states(),
        ["observed", "prepared", "building", "evaluating"]
    );
    assert_eq!(*harness.launches.lock().unwrap(), 1);
    assert!(*harness.provision_calls.lock().unwrap() >= 1);

    let again = run_once_retry_lock_busy(&harness.evolver).await.unwrap();
    assert_eq!(again.state, CandidateState::Evaluating);
    assert_eq!(again.candidate_id, outcome.candidate_id);
    assert_eq!(*harness.launches.lock().unwrap(), 1);
    assert_eq!(harness.remote_main(), harness.initial_main());
}

#[tokio::test]
async fn resume_after_building_receipt_without_relaunch() {
    let harness = RepoHarness::new().await;
    let outcome = run_once_retry_lock_busy(&harness.evolver).await.unwrap();
    assert_eq!(outcome.state, CandidateState::Evaluating);
    let launches = *harness.launches.lock().unwrap();

    // Explicit resume at Evaluating returns unchanged.
    let resumed = resume_retry_lock_busy(&harness.evolver).await.unwrap();
    assert_eq!(resumed.state, CandidateState::Evaluating);
    assert_eq!(resumed.candidate_id, outcome.candidate_id);
    assert_eq!(*harness.launches.lock().unwrap(), launches);
}

#[tokio::test]
async fn abort_prepared_preserves_artifacts() {
    let harness = RepoHarness::new().await;
    let (id, ws) = {
        let lock = CoordinatorLock::try_acquire(harness.fixture.config.state_dir()).unwrap();
        let store = StateStore::open(harness.fixture.config.state_dir()).unwrap();
        let clock = ManualClock::new(fixed_now());
        let fake = FakeProcessRunner::new();
        install_mission_fake(&fake, &clock);
        let hybrid = HybridRunner {
            git: harness.fixture.runner(),
            fake_mission: fake,
        };
        let prep = prepare_candidate(&harness.fixture.config, &hybrid, &clock, &store).unwrap();
        assert_eq!(prep.record.state(), CandidateState::Prepared);
        let id = prep.record.id().as_str().to_owned();
        let ws = prep.record.workspace().unwrap().to_path_buf();
        drop(store);
        drop(lock);
        (id, ws)
    };

    let aborted = abort_retry_lock_busy(&harness.evolver, &id, "operator-abort-test")
        .await
        .unwrap();
    assert_eq!(aborted.state, CandidateState::Failed);
    assert_eq!(
        aborted.terminal_reason.as_deref(),
        Some("operator-abort-test")
    );
    assert!(ws.exists(), "workspace preserved after abort");
}

#[tokio::test]
async fn status_null_vs_zero_before_receipt() {
    let harness = RepoHarness::new().await;
    let status = harness.evolver.status().await.unwrap();
    assert_eq!(status.schema, "gzmo.repo_evolver.status/v1");
    // No candidate yet: used remains null-shaped when present after empty.
    assert!(status.candidate_id.is_none());
    assert_eq!(status.next_action, "run");
}

#[tokio::test]
async fn post_squash_crash_resume_reaches_evaluating_without_relaunch() {
    let harness = RepoHarness::new().await;
    *harness.launch_mode.lock().unwrap() = LaunchMode::PostSquashNormalize;
    let outcome = run_once_retry_lock_busy(&harness.evolver).await.unwrap();
    assert_eq!(outcome.state, CandidateState::Evaluating);
    assert!(outcome
        .candidate_digest
        .as_deref()
        .unwrap()
        .starts_with("git-sha1:"));
    assert_eq!(*harness.launches.lock().unwrap(), 1);
    let again = resume_retry_lock_busy(&harness.evolver).await.unwrap();
    assert_eq!(again.candidate_digest, outcome.candidate_digest);
    assert_eq!(*harness.launches.lock().unwrap(), 1);
}

#[tokio::test]
async fn receipt_fact_mismatch_terminalizes() {
    let harness = RepoHarness::new().await;
    *harness.launch_mode.lock().unwrap() = LaunchMode::FactMismatch;
    let err = run_once_retry_lock_busy(&harness.evolver)
        .await
        .unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("fact mismatch"), "{msg}");
    let repo = format!(
        "{}/{}",
        harness.fixture.config.repo().owner(),
        harness.fixture.config.repo().repository()
    );
    let store = StateStore::open(harness.fixture.config.state_dir()).unwrap();
    let latest = store.latest_candidate(&repo).unwrap().unwrap();
    assert_eq!(latest.state(), CandidateState::Failed);
}

#[tokio::test]
async fn empty_normalized_candidate_terminalizes() {
    let harness = RepoHarness::new().await;
    *harness.launch_mode.lock().unwrap() = LaunchMode::EmptyNormalized;
    let err = run_once_retry_lock_busy(&harness.evolver)
        .await
        .unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("no changes") || msg.contains("empty") || msg.contains("diff"),
        "{msg}"
    );
    let repo = format!(
        "{}/{}",
        harness.fixture.config.repo().owner(),
        harness.fixture.config.repo().repository()
    );
    let store = StateStore::open(harness.fixture.config.state_dir()).unwrap();
    let latest = store.latest_candidate(&repo).unwrap().unwrap();
    assert_eq!(latest.state(), CandidateState::Failed);
}

#[tokio::test]
async fn dirty_workspace_after_receipt_terminalizes() {
    let harness = RepoHarness::new().await;
    *harness.launch_mode.lock().unwrap() = LaunchMode::DirtyAfterReceipt;
    let err = run_once_retry_lock_busy(&harness.evolver)
        .await
        .unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("dirty") || msg.contains("untracked"), "{msg}");
    let repo = format!(
        "{}/{}",
        harness.fixture.config.repo().owner(),
        harness.fixture.config.repo().repository()
    );
    let store = StateStore::open(harness.fixture.config.state_dir()).unwrap();
    let latest = store.latest_candidate(&repo).unwrap().unwrap();
    assert_eq!(latest.state(), CandidateState::Failed);
}

#[tokio::test]
async fn building_not_found_without_receipt_never_relaunches() {
    let harness = RepoHarness::new().await;
    *harness.launch_mode.lock().unwrap() = LaunchMode::SucceededNoReceipt;
    let err = run_once_retry_lock_busy(&harness.evolver)
        .await
        .unwrap_err();
    assert!(
        err.to_string().contains("worker_lost_without_receipt"),
        "{err}"
    );
    let launches = *harness.launches.lock().unwrap();
    assert_eq!(launches, 1);
    let resumed = resume_retry_lock_busy(&harness.evolver).await.unwrap();
    assert_eq!(resumed.state, CandidateState::Failed);
    assert_eq!(
        resumed.terminal_reason.as_deref(),
        Some("worker_lost_without_receipt")
    );
    assert_eq!(*harness.launches.lock().unwrap(), launches);
}

#[tokio::test]
async fn building_unit_failed_without_receipt_terminalizes() {
    let harness = RepoHarness::new().await;
    *harness.launch_mode.lock().unwrap() = LaunchMode::UnitFailed;
    let err = run_once_retry_lock_busy(&harness.evolver)
        .await
        .unwrap_err();
    assert!(
        err.to_string().contains("worker_lost_without_receipt")
            || err.to_string().contains("worker unit failed"),
        "{err}"
    );
    assert_eq!(*harness.launches.lock().unwrap(), 1);
}

#[tokio::test]
async fn abort_building_stops_before_failed_and_preserves_artifacts() {
    // Deliberate seed: prepare workspace then transition to Building so abort's
    // production stop-before-Failed path is the subject under test.
    let harness = RepoHarness::new().await;
    let (id, ws) = {
        let lock = CoordinatorLock::try_acquire(harness.fixture.config.state_dir()).unwrap();
        let store = StateStore::open(harness.fixture.config.state_dir()).unwrap();
        let clock = ManualClock::new(fixed_now());
        let fake = FakeProcessRunner::new();
        install_mission_fake(&fake, &clock);
        let hybrid = HybridRunner {
            git: harness.fixture.runner(),
            fake_mission: fake,
        };
        let prep = prepare_candidate(&harness.fixture.config, &hybrid, &clock, &store).unwrap();
        let id = prep.record.id().clone();
        let ws = prep.record.workspace().unwrap().to_path_buf();
        let building = store
            .transition(
                &id,
                CandidateState::Building,
                TransitionMetadata::empty(),
                fixed_now(),
            )
            .unwrap();
        assert_eq!(building.state(), CandidateState::Building);
        drop(store);
        drop(lock);
        (id, ws)
    };
    *harness.unit_state.lock().unwrap() = WorkerUnitState::Running;
    *harness.launch_mode.lock().unwrap() = LaunchMode::TrackStop;

    let aborted = abort_retry_lock_busy(&harness.evolver, id.as_str(), "building-abort")
        .await
        .unwrap();
    assert_eq!(aborted.state, CandidateState::Failed);
    let log = harness.stop_log.lock().unwrap().clone();
    assert_eq!(log, ["stop", "inactive"]);
    assert!(ws.exists(), "workspace preserved after abort");
}

#[tokio::test]
async fn receipt_directory_trust_fault_terminalizes_exact_reason() {
    // Seed Building with sealed request, then plant a directory at receipt.json.
    let harness = RepoHarness::new().await;
    let lock = CoordinatorLock::try_acquire(harness.fixture.config.state_dir()).unwrap();
    let store = StateStore::open(harness.fixture.config.state_dir()).unwrap();
    let clock = ManualClock::new(fixed_now());
    let fake = FakeProcessRunner::new();
    install_mission_fake(&fake, &clock);
    let hybrid = HybridRunner {
        git: harness.fixture.runner(),
        fake_mission: fake,
    };
    let prep = prepare_candidate(&harness.fixture.config, &hybrid, &clock, &store).unwrap();
    let id = prep.record.id().clone();
    store
        .transition(
            &id,
            CandidateState::Building,
            TransitionMetadata::empty(),
            fixed_now(),
        )
        .unwrap();
    drop(store);
    drop(lock);

    let sealed = seal_for_test(&harness, &prep);
    let receipt_path = sealed.output_dir().join("receipt.json");
    let _ = fs::remove_file(&receipt_path);
    fs::create_dir_all(&receipt_path).unwrap();

    *harness.unit_state.lock().unwrap() = WorkerUnitState::Succeeded;
    let err = resume_retry_lock_busy(&harness.evolver).await.unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("receipt must be a regular file"),
        "expected exact regular-file trust reason, got {msg}"
    );
}

#[tokio::test]
async fn lock_busy_is_contention() {
    // No retry: this test asserts LockBusy itself.
    let harness = RepoHarness::new().await;
    let _held = CoordinatorLock::try_acquire(harness.fixture.config.state_dir()).unwrap();
    let err = harness.evolver.run_once().await.unwrap_err();
    assert!(matches!(err, gzmo_evolver::RunnerError::LockBusy), "{err}");
}

#[tokio::test]
async fn terminal_resume_returns_unchanged() {
    let harness = RepoHarness::new().await;
    *harness.launch_mode.lock().unwrap() = LaunchMode::UnitFailed;
    let _ = run_once_retry_lock_busy(&harness.evolver).await;
    let resumed = resume_retry_lock_busy(&harness.evolver).await.unwrap();
    assert_eq!(resumed.state, CandidateState::Failed);
}

#[tokio::test]
async fn abort_rejects_foreign_repository_candidate() {
    let harness = RepoHarness::new().await;
    let (local_id, foreign_id) = {
        let lock = CoordinatorLock::try_acquire(harness.fixture.config.state_dir()).unwrap();
        let store = StateStore::open(harness.fixture.config.state_dir()).unwrap();
        let clock = ManualClock::new(fixed_now());
        let fake = FakeProcessRunner::new();
        install_mission_fake(&fake, &clock);
        let hybrid = HybridRunner {
            git: harness.fixture.runner(),
            fake_mission: fake,
        };
        let prep = prepare_candidate(&harness.fixture.config, &hybrid, &clock, &store).unwrap();
        let local_id = prep.record.id().as_str().to_owned();

        // Insert a real foreign-repo candidate row.
        let foreign_cid =
            CandidateId::parse("cand-20260901t120000z-foreign-repo-aaaaaaaa").unwrap();
        let mut foreign_manifest = prep.record.manifest().clone();
        foreign_manifest.id = foreign_cid.clone();
        foreign_manifest.target = evolution_contracts::CandidateTarget::Repository {
            owner: "other-owner".to_owned(),
            repository: "OTHER".to_owned(),
            base_branch: "main".to_owned(),
            candidate_branch: format!("evolve/{}", foreign_cid.as_str()),
        };
        foreign_manifest.validate().unwrap();
        store
            .create_candidate(&foreign_manifest, prep.record.policy_digest(), fixed_now())
            .unwrap();
        drop(store);
        drop(lock);
        (local_id, foreign_cid.as_str().to_owned())
    };

    let err = abort_retry_lock_busy(&harness.evolver, &foreign_id, "x")
        .await
        .expect_err("foreign abort must be rejected");
    assert!(
        err.to_string().contains("belongs to repository")
            || err.to_string().contains("not configured"),
        "{err}"
    );
    let store = StateStore::open(harness.fixture.config.state_dir()).unwrap();
    let foreign = store
        .load(&CandidateId::parse(&foreign_id).unwrap())
        .unwrap();
    assert_eq!(foreign.state(), CandidateState::Observed);
    let local = store.load(&CandidateId::parse(&local_id).unwrap()).unwrap();
    assert_eq!(local.state(), CandidateState::Prepared);
}

#[tokio::test]
async fn prepared_sealed_request_reuse_on_resume() {
    // Pre-seal at Prepared so run_once must reuse, not reseal.
    let harness = RepoHarness::new().await;
    let lock = CoordinatorLock::try_acquire(harness.fixture.config.state_dir()).unwrap();
    let store = StateStore::open(harness.fixture.config.state_dir()).unwrap();
    let clock = ManualClock::new(fixed_now());
    let fake = FakeProcessRunner::new();
    install_mission_fake(&fake, &clock);
    let hybrid = HybridRunner {
        git: harness.fixture.runner(),
        fake_mission: fake,
    };
    let prep = prepare_candidate(&harness.fixture.config, &hybrid, &clock, &store).unwrap();
    assert_eq!(prep.record.state(), CandidateState::Prepared);
    let id = prep.record.id().clone();
    drop(store);
    drop(lock);

    let _ = seal_for_test(&harness, &prep);
    let req_path = harness
        .roots
        .request_root()
        .join(id.as_str())
        .join("request.json");
    let before = fs::read(&req_path).unwrap();
    let before_meta = fs::metadata(&req_path).unwrap();

    *harness.launch_mode.lock().unwrap() = LaunchMode::Happy;
    let outcome = run_once_retry_lock_busy(&harness.evolver).await.unwrap();
    assert_eq!(outcome.state, CandidateState::Evaluating);
    let after = fs::read(&req_path).unwrap();
    assert_eq!(before, after, "sealed request must be reused unchanged");
    let after_meta = fs::metadata(&req_path).unwrap();
    assert_eq!(
        before_meta.modified().ok(),
        after_meta.modified().ok(),
        "request mtime must not change on reuse"
    );
}

#[tokio::test]
async fn no_duplicate_launch_from_building_succeeded() {
    // Seed Building with a sealed request and a Succeeded unit; resume must not launch.
    let harness = RepoHarness::new().await;
    let lock = CoordinatorLock::try_acquire(harness.fixture.config.state_dir()).unwrap();
    let store = StateStore::open(harness.fixture.config.state_dir()).unwrap();
    let clock = ManualClock::new(fixed_now());
    let fake = FakeProcessRunner::new();
    install_mission_fake(&fake, &clock);
    let hybrid = HybridRunner {
        git: harness.fixture.runner(),
        fake_mission: fake,
    };
    let prep = prepare_candidate(&harness.fixture.config, &hybrid, &clock, &store).unwrap();
    let id = prep.record.id().clone();
    store
        .transition(
            &id,
            CandidateState::Building,
            TransitionMetadata::empty(),
            fixed_now(),
        )
        .unwrap();
    drop(store);
    drop(lock);

    let _ = seal_for_test(&harness, &prep);
    *harness.unit_state.lock().unwrap() = WorkerUnitState::Succeeded;
    *harness.launches.lock().unwrap() = 0;
    let err = resume_retry_lock_busy(&harness.evolver).await.unwrap_err();
    assert!(
        err.to_string().contains("worker_lost_without_receipt"),
        "{err}"
    );
    assert_eq!(
        *harness.launches.lock().unwrap(),
        0,
        "must not launch when unit already Succeeded"
    );
}

#[derive(Debug, Clone, Copy)]
enum MirrorNetworkFault {
    Timeout,
    /// status 128 + realistic fast-fail transport stderr (DNS/connect).
    TransientNonZero,
    /// status 128 + HTTP 429 rate-limit stderr — transient.
    RateLimitNonZero,
    /// status 128 + GnuTLS handshake stderr — transient.
    GnuTlsNonZero,
    /// status 128 + auth/permission stderr — permanent.
    AuthNonZero,
    /// status 128 + repository-not-found stderr — permanent.
    RepoNotFoundNonZero,
    /// status 128 + HTTP 403 stderr — permanent.
    ForbiddenNonZero,
    /// status 128 + HTTP 404 stderr — permanent.
    NotFoundHttpNonZero,
    /// status 128 + unknown/empty stderr — permanent.
    UnknownNonZero,
}

/// Secret marker embedded only in injected stderr; must never appear in errors.
const MIRROR_STDERR_SECRET: &str = "SECRET_TOKEN_do_not_leak_xyz";

/// Injects structural Timeout or classified nonzero only on mirror clone/fetch argv.
struct FaultyMirrorRunner {
    inner: HermeticGitRunner,
    fault: MirrorNetworkFault,
}

impl ProcessRunner for FaultyMirrorRunner {
    fn run(&self, spec: &ProcessSpec) -> Result<ProcessOutput, ProcessError> {
        let injected = self.inner.inject(spec)?;
        let has_clone = injected.args.iter().any(|a| a == "clone");
        let has_mirror = injected.args.iter().any(|a| a == "--mirror");
        let has_fetch = injected.args.iter().any(|a| a == "fetch");
        let has_origin = injected.args.iter().any(|a| a == "origin");
        let mirror_network = (has_clone && has_mirror) || (has_fetch && has_origin);
        if mirror_network {
            match self.fault {
                MirrorNetworkFault::Timeout => {
                    return Err(ProcessError::Timeout { timeout_ms: 1 });
                }
                MirrorNetworkFault::TransientNonZero => {
                    let stderr = format!(
                        "fatal: unable to access 'https://github.com/example/x.git/': Could not resolve host: github.com\nleak:{MIRROR_STDERR_SECRET}\n"
                    );
                    return Ok(ProcessOutput {
                        status: 128,
                        stdout: Vec::new(),
                        stderr: stderr.into_bytes(),
                    });
                }
                MirrorNetworkFault::RateLimitNonZero => {
                    let stderr = format!(
                        "fatal: unable to access 'https://github.com/example/x.git/': The requested URL returned error: 429\nleak:{MIRROR_STDERR_SECRET}\n"
                    );
                    return Ok(ProcessOutput {
                        status: 128,
                        stdout: Vec::new(),
                        stderr: stderr.into_bytes(),
                    });
                }
                MirrorNetworkFault::GnuTlsNonZero => {
                    let stderr = format!(
                        "fatal: unable to access 'https://github.com/example/x.git/': gnutls_handshake() failed: Error in the pull function.\nleak:{MIRROR_STDERR_SECRET}\n"
                    );
                    return Ok(ProcessOutput {
                        status: 128,
                        stdout: Vec::new(),
                        stderr: stderr.into_bytes(),
                    });
                }
                MirrorNetworkFault::AuthNonZero => {
                    let stderr = format!(
                        "fatal: Authentication failed for 'https://github.com/example/x.git/'\nleak:{MIRROR_STDERR_SECRET}\n"
                    );
                    return Ok(ProcessOutput {
                        status: 128,
                        stdout: Vec::new(),
                        stderr: stderr.into_bytes(),
                    });
                }
                MirrorNetworkFault::RepoNotFoundNonZero => {
                    let stderr = format!(
                        "remote: Repository not found.\nfatal: repository 'https://github.com/example/x.git/' not found\nleak:{MIRROR_STDERR_SECRET}\n"
                    );
                    return Ok(ProcessOutput {
                        status: 128,
                        stdout: Vec::new(),
                        stderr: stderr.into_bytes(),
                    });
                }
                MirrorNetworkFault::ForbiddenNonZero => {
                    let stderr = format!(
                        "fatal: unable to access 'https://github.com/example/x.git/': The requested URL returned error: 403\nleak:{MIRROR_STDERR_SECRET}\n"
                    );
                    return Ok(ProcessOutput {
                        status: 128,
                        stdout: Vec::new(),
                        stderr: stderr.into_bytes(),
                    });
                }
                MirrorNetworkFault::NotFoundHttpNonZero => {
                    let stderr = format!(
                        "fatal: unable to access 'https://github.com/example/x.git/': The requested URL returned error: 404\nleak:{MIRROR_STDERR_SECRET}\n"
                    );
                    return Ok(ProcessOutput {
                        status: 128,
                        stdout: Vec::new(),
                        stderr: stderr.into_bytes(),
                    });
                }
                MirrorNetworkFault::UnknownNonZero => {
                    let stderr = format!(
                        "fatal: something unexplained happened\nleak:{MIRROR_STDERR_SECRET}\n"
                    );
                    return Ok(ProcessOutput {
                        status: 128,
                        stdout: Vec::new(),
                        stderr: stderr.into_bytes(),
                    });
                }
            }
        }
        self.inner.inner.run(&injected)
    }
}

fn observed_evolver_with_mirror_fault(
    fixture: &Fixture,
    fault: MirrorNetworkFault,
) -> (
    RepoEvolver<
        FaultyMirrorRunner,
        FakeStageLauncher,
        FakeRuntimeProvisioner,
        TestWorkerIdentity,
        ManualClock,
    >,
    WorkerRoots,
    CandidateId,
) {
    let request_root = fixture.state_dir.join("run-requests");
    let output_root = fixture.state_dir.join("worker-out");
    let profile_root = fixture.state_dir.join("profiles");
    let netns = fixture.state_dir.join("netns");
    fs::create_dir_all(&request_root).unwrap();
    fs::create_dir_all(&output_root).unwrap();
    fs::create_dir_all(&profile_root).unwrap();
    fs::create_dir_all(&netns).unwrap();
    let profile = fixture.config.worker().profile().to_owned();
    let profile_dir = profile_root.join(&profile);
    fs::create_dir_all(profile_dir.join("agent")).unwrap();
    fs::write(
        profile_dir.join("agent/config.yml"),
        "modelRoles:\n  code_candidate: local/code\n",
    )
    .unwrap();
    fs::write(
        profile_dir.join("agent/models.yml"),
        "providers:\n  local:\n    auth: none\n    baseUrl: http://127.0.0.1:9\n    models:\n      - id: code\n        maxTokens: 16384\n        contextWindow: 32768\n",
    )
    .unwrap();
    let fixture_omp =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/fake-worker.sh");
    fs::copy(&fixture_omp, fixture.config.worker().executable()).unwrap();
    let mut perms = fs::metadata(fixture.config.worker().executable())
        .unwrap()
        .permissions();
    perms.set_mode(0o755);
    fs::set_permissions(fixture.config.worker().executable(), perms).unwrap();

    let roots = WorkerRoots::for_test(request_root, output_root, profile_root, netns).unwrap();
    let launches = Arc::new(Mutex::new(0u32));
    let provision_calls = Arc::new(Mutex::new(0u32));
    let unit_state = Arc::new(Mutex::new(WorkerUnitState::NotFound));
    let launch_mode = Arc::new(Mutex::new(LaunchMode::Happy));
    let stop_log = Arc::new(Mutex::new(Vec::new()));

    let runner = FaultyMirrorRunner {
        inner: fixture.runner(),
        fault,
    };
    let real_uid = nix::unistd::Uid::effective().as_raw();
    let real_gid = nix::unistd::Gid::effective().as_raw().max(1);
    let coordinator_uid = real_uid.wrapping_add(1000).max(1);
    let evolver = RepoEvolver::with_deps(
        fixture.config.clone(),
        runner,
        FakeStageLauncher {
            launches,
            unit_state,
            mode: launch_mode,
            stop_log,
            env_names: Arc::new(Mutex::new(Vec::new())),
            now: fixed_now(),
        },
        FakeRuntimeProvisioner {
            roots: roots.clone(),
            profile,
            calls: provision_calls,
        },
        TestWorkerIdentity {
            uid: real_uid,
            gid: real_gid,
        },
        ManualClock::new(fixed_now()),
        roots.clone(),
        coordinator_uid,
    )
    .unwrap();

    // Seed Observed without a mirror so resume hits clone/fetch under the fault.
    let baseline = fixture.baseline_before.clone();
    let id = CandidateId::parse("cand-20260901t120000z-bet-mirrorfault1").unwrap();
    let lock = CoordinatorLock::try_acquire(fixture.config.state_dir()).unwrap();
    let store = StateStore::open(fixture.config.state_dir()).unwrap();
    let manifest = manifest_for(&baseline, id.as_str());
    store
        .create_candidate(
            &manifest,
            fixture.config.working_policy_digest(),
            fixed_now(),
        )
        .unwrap();
    drop(store);
    drop(lock);
    let _ = fs::remove_dir_all(fixture.state_dir.join(gzmo_evolver::MIRROR_NAME));
    (evolver, roots, id)
}

fn assert_no_stderr_secret(err: &gzmo_evolver::RunnerError) {
    let msg = err.to_string();
    assert!(
        !msg.contains(MIRROR_STDERR_SECRET),
        "stderr secret must not appear in error: {msg}"
    );
    let dbg = format!("{err:?}");
    assert!(
        !dbg.contains(MIRROR_STDERR_SECRET),
        "stderr secret must not appear in debug: {dbg}"
    );
}

#[tokio::test]
async fn observed_mirror_timeout_is_contention_leaves_observed() {
    let fixture = Fixture::new();
    let (evolver, _roots, id) =
        observed_evolver_with_mirror_fault(&fixture, MirrorNetworkFault::Timeout);
    let err = resume_retry_lock_busy(&evolver)
        .await
        .expect_err("timeout must not succeed");
    assert_no_stderr_secret(&err);
    assert!(
        matches!(err, gzmo_evolver::RunnerError::Contention(_)),
        "timeout must be Contention, got {err}"
    );
    let store = StateStore::open(fixture.config.state_dir()).unwrap();
    let rec = store.load(&id).unwrap();
    assert_eq!(rec.state(), CandidateState::Observed);
    if let Some(reason) = rec.terminal_reason() {
        assert!(
            !reason.contains(MIRROR_STDERR_SECRET),
            "secret in terminal_reason: {reason}"
        );
    }
}

#[tokio::test]
async fn observed_mirror_transient_nonzero_is_contention_leaves_observed() {
    let fixture = Fixture::new();
    let (evolver, _roots, id) =
        observed_evolver_with_mirror_fault(&fixture, MirrorNetworkFault::TransientNonZero);
    let err = resume_retry_lock_busy(&evolver)
        .await
        .expect_err("transient transport must not succeed");
    assert_no_stderr_secret(&err);
    assert!(
        matches!(err, gzmo_evolver::RunnerError::Contention(_)),
        "fast transient status128 must be Contention, got {err}"
    );
    let msg = err.to_string();
    assert!(
        msg.contains("transport") || msg.contains("Contention") || msg.contains("contention"),
        "{msg}"
    );
    assert!(!msg.contains("Could not resolve host"), "{msg}");
    let store = StateStore::open(fixture.config.state_dir()).unwrap();
    let rec = store.load(&id).unwrap();
    assert_eq!(rec.state(), CandidateState::Observed);
    if let Some(reason) = rec.terminal_reason() {
        assert!(
            !reason.contains(MIRROR_STDERR_SECRET),
            "secret in terminal_reason: {reason}"
        );
    }
}

#[tokio::test]
async fn observed_mirror_auth_nonzero_terminalizes_not_contention() {
    let fixture = Fixture::new();
    let (evolver, _roots, id) =
        observed_evolver_with_mirror_fault(&fixture, MirrorNetworkFault::AuthNonZero);
    let err = resume_retry_lock_busy(&evolver)
        .await
        .expect_err("auth failure must terminalize");
    assert_no_stderr_secret(&err);
    assert!(
        !matches!(err, gzmo_evolver::RunnerError::Contention(_)),
        "auth status128 must not be Contention, got {err}"
    );
    assert!(!err.to_string().contains("Authentication failed"), "{err}");
    let store = StateStore::open(fixture.config.state_dir()).unwrap();
    let rec = store.load(&id).unwrap();
    assert_eq!(rec.state(), CandidateState::Failed);
    if let Some(reason) = rec.terminal_reason() {
        assert!(
            !reason.contains(MIRROR_STDERR_SECRET),
            "secret in terminal_reason: {reason}"
        );
        assert!(
            !reason.contains("Authentication failed"),
            "auth stderr must not leak into reason: {reason}"
        );
    }
}

#[tokio::test]
async fn observed_mirror_repo_not_found_terminalizes_not_contention() {
    let fixture = Fixture::new();
    let (evolver, _roots, id) =
        observed_evolver_with_mirror_fault(&fixture, MirrorNetworkFault::RepoNotFoundNonZero);
    let err = resume_retry_lock_busy(&evolver)
        .await
        .expect_err("repository-not-found must terminalize");
    assert_no_stderr_secret(&err);
    assert!(
        !matches!(err, gzmo_evolver::RunnerError::Contention(_)),
        "repository-not-found must not be Contention, got {err}"
    );
    assert!(!err.to_string().contains("Repository not found"), "{err}");
    let store = StateStore::open(fixture.config.state_dir()).unwrap();
    let rec = store.load(&id).unwrap();
    assert_eq!(rec.state(), CandidateState::Failed);
    if let Some(reason) = rec.terminal_reason() {
        assert!(
            !reason.contains(MIRROR_STDERR_SECRET),
            "secret in terminal_reason: {reason}"
        );
    }
}

#[tokio::test]
async fn observed_mirror_unknown_nonzero_terminalizes_not_contention() {
    let fixture = Fixture::new();
    let (evolver, _roots, id) =
        observed_evolver_with_mirror_fault(&fixture, MirrorNetworkFault::UnknownNonZero);
    let err = resume_retry_lock_busy(&evolver)
        .await
        .expect_err("unknown nonzero must terminalize");
    assert_no_stderr_secret(&err);
    assert!(
        !matches!(err, gzmo_evolver::RunnerError::Contention(_)),
        "unknown status128 must not be Contention, got {err}"
    );
    assert!(!err.to_string().contains("something unexplained"), "{err}");
    let store = StateStore::open(fixture.config.state_dir()).unwrap();
    let rec = store.load(&id).unwrap();
    assert_eq!(rec.state(), CandidateState::Failed);
    if let Some(reason) = rec.terminal_reason() {
        assert!(
            !reason.contains(MIRROR_STDERR_SECRET),
            "secret in terminal_reason: {reason}"
        );
        assert!(
            !reason.contains("something unexplained"),
            "unknown stderr must not leak: {reason}"
        );
    }
}

#[tokio::test]
async fn observed_mirror_rate_limit_nonzero_is_contention_leaves_observed() {
    let fixture = Fixture::new();
    let (evolver, _roots, id) =
        observed_evolver_with_mirror_fault(&fixture, MirrorNetworkFault::RateLimitNonZero);
    let err = resume_retry_lock_busy(&evolver)
        .await
        .expect_err("429 must not succeed");
    assert_no_stderr_secret(&err);
    assert!(
        matches!(err, gzmo_evolver::RunnerError::Contention(_)),
        "HTTP 429 must be Contention, got {err}"
    );
    let msg = err.to_string();
    assert!(!msg.contains("429"), "{msg}");
    assert!(!msg.contains(MIRROR_STDERR_SECRET), "{msg}");
    let store = StateStore::open(fixture.config.state_dir()).unwrap();
    let rec = store.load(&id).unwrap();
    assert_eq!(rec.state(), CandidateState::Observed);
    if let Some(reason) = rec.terminal_reason() {
        assert!(
            !reason.contains(MIRROR_STDERR_SECRET),
            "secret in terminal_reason: {reason}"
        );
    }
}

#[tokio::test]
async fn observed_mirror_gnutls_nonzero_is_contention_leaves_observed() {
    let fixture = Fixture::new();
    let (evolver, _roots, id) =
        observed_evolver_with_mirror_fault(&fixture, MirrorNetworkFault::GnuTlsNonZero);
    let err = resume_retry_lock_busy(&evolver)
        .await
        .expect_err("GnuTLS must not succeed");
    assert_no_stderr_secret(&err);
    assert!(
        matches!(err, gzmo_evolver::RunnerError::Contention(_)),
        "GnuTLS handshake must be Contention, got {err}"
    );
    let msg = err.to_string();
    assert!(!msg.contains("gnutls_handshake"), "{msg}");
    assert!(!msg.contains(MIRROR_STDERR_SECRET), "{msg}");
    let store = StateStore::open(fixture.config.state_dir()).unwrap();
    let rec = store.load(&id).unwrap();
    assert_eq!(rec.state(), CandidateState::Observed);
    if let Some(reason) = rec.terminal_reason() {
        assert!(
            !reason.contains(MIRROR_STDERR_SECRET),
            "secret in terminal_reason: {reason}"
        );
    }
}

#[tokio::test]
async fn observed_mirror_forbidden_nonzero_terminalizes_not_contention() {
    let fixture = Fixture::new();
    let (evolver, _roots, id) =
        observed_evolver_with_mirror_fault(&fixture, MirrorNetworkFault::ForbiddenNonZero);
    let err = resume_retry_lock_busy(&evolver)
        .await
        .expect_err("403 must terminalize");
    assert_no_stderr_secret(&err);
    assert!(
        !matches!(err, gzmo_evolver::RunnerError::Contention(_)),
        "HTTP 403 must not be Contention, got {err}"
    );
    assert!(!err.to_string().contains("403"), "{err}");
    let store = StateStore::open(fixture.config.state_dir()).unwrap();
    let rec = store.load(&id).unwrap();
    assert_eq!(rec.state(), CandidateState::Failed);
    if let Some(reason) = rec.terminal_reason() {
        assert!(
            !reason.contains(MIRROR_STDERR_SECRET),
            "secret in terminal_reason: {reason}"
        );
        assert!(!reason.contains("403"), "status must not leak: {reason}");
    }
}

// --- Task 7: hermetic vertical fixture (real mission script + fake worker only) ---

/// Git hermetic + real SystemProcessRunner for bash mission + omp --version fake.
struct RealMissionHybrid {
    git: HermeticGitRunner,
    system: SystemProcessRunner,
    omp_version: String,
}

impl ProcessRunner for RealMissionHybrid {
    fn run(&self, spec: &ProcessSpec) -> Result<ProcessOutput, ProcessError> {
        let prog = spec.program.to_string_lossy();
        if prog.ends_with("git") || prog == "git" {
            return self.git.run(spec);
        }
        if prog.ends_with("bash") || prog == "bash" || prog == "/bin/bash" {
            // Production MissionAdapter path: real SystemProcessRunner executes the
            // committed fixture scripts/opportunity-next-mission.sh.
            return self.system.run(spec);
        }
        if spec.args.iter().any(|a| a == "--version") {
            return Ok(ProcessOutput {
                status: 0,
                stdout: format!("{}\n", self.omp_version).into_bytes(),
                stderr: Vec::new(),
            });
        }
        self.git.run(spec)
    }
}
struct FixtureVertical {
    fixture: Fixture,
    evolver: RepoEvolver<
        RealMissionHybrid,
        FakeStageLauncher,
        FakeRuntimeProvisioner,
        TestWorkerIdentity,
        gzmo_evolver::SystemClock,
    >,
    store: StateStore,
    initial_main: String,
    roots: WorkerRoots,
    launches: Arc<Mutex<u32>>,
    provision_calls: Arc<Mutex<u32>>,
    env_names: Arc<Mutex<Vec<String>>>,
    trusted_main_blob: String,
    remote_main_blob: String,
}

impl FixtureVertical {
    async fn new() -> Self {
        let fixture = Fixture::new();
        // Prove mission script is the committed fixture (non-empty, executable mode).
        let script = fixture.checkout.join("scripts/opportunity-next-mission.sh");
        let meta = fs::metadata(&script).unwrap();
        assert!(
            meta.len() > 100,
            "committed mission script must be nonempty"
        );
        assert!(
            meta.permissions().mode() & 0o111 != 0,
            "mission script should be executable"
        );

        let request_root = fixture.state_dir.join("run-requests");
        let output_root = fixture.state_dir.join("worker-out");
        let profile_root = fixture.state_dir.join("profiles");
        let netns = fixture.state_dir.join("netns");
        fs::create_dir_all(&request_root).unwrap();
        fs::create_dir_all(&output_root).unwrap();
        fs::create_dir_all(&profile_root).unwrap();
        fs::create_dir_all(&netns).unwrap();

        let profile = fixture.config.worker().profile().to_owned();
        let profile_dir = profile_root.join(&profile);
        fs::create_dir_all(profile_dir.join("agent")).unwrap();
        fs::write(
            profile_dir.join("agent/config.yml"),
            "modelRoles:\n  code_candidate: local/code\n",
        )
        .unwrap();
        fs::write(
            profile_dir.join("agent/models.yml"),
            "providers:\n  local:\n    auth: none\n    baseUrl: http://127.0.0.1:9\n    models:\n      - id: code\n        maxTokens: 16384\n        contextWindow: 32768\n",
        )
        .unwrap();
        for walk in [
            profile_dir.clone(),
            profile_dir.join("agent"),
            profile_dir.join("agent/config.yml"),
            profile_dir.join("agent/models.yml"),
        ] {
            let mut p = fs::metadata(&walk).unwrap().permissions();
            p.set_mode(if walk.is_dir() { 0o755 } else { 0o644 });
            fs::set_permissions(&walk, p).unwrap();
        }

        let fixture_omp =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/fake-worker.sh");
        fs::copy(&fixture_omp, fixture.config.worker().executable()).unwrap();
        let mut perms = fs::metadata(fixture.config.worker().executable())
            .unwrap()
            .permissions();
        perms.set_mode(0o755);
        fs::set_permissions(fixture.config.worker().executable(), perms).unwrap();

        let roots = WorkerRoots::for_test(request_root, output_root, profile_root, netns).unwrap();
        let launches = Arc::new(Mutex::new(0u32));
        let provision_calls = Arc::new(Mutex::new(0u32));
        let unit_state = Arc::new(Mutex::new(WorkerUnitState::NotFound));
        let launch_mode = Arc::new(Mutex::new(LaunchMode::Happy));
        let stop_log = Arc::new(Mutex::new(Vec::new()));
        let env_names = Arc::new(Mutex::new(Vec::new()));

        let omp_version = "18.0.11".to_owned();
        let hybrid = RealMissionHybrid {
            git: fixture.runner(),
            system: SystemProcessRunner,
            omp_version: omp_version.clone(),
        };
        let launcher = FakeStageLauncher {
            launches: launches.clone(),
            unit_state: unit_state.clone(),
            mode: launch_mode.clone(),
            stop_log: stop_log.clone(),
            env_names: env_names.clone(),
            now: fixed_now(),
        };
        let provisioner = FakeRuntimeProvisioner {
            roots: roots.clone(),
            profile: profile.clone(),
            calls: provision_calls.clone(),
        };

        let real_uid = nix::unistd::Uid::effective().as_raw();
        let real_gid = nix::unistd::Gid::effective().as_raw();
        let worker_uid = real_uid;
        let worker_gid = real_gid.max(1);
        let coordinator_uid = real_uid.wrapping_add(1000).max(1);
        assert_ne!(coordinator_uid, real_uid);

        let evolver = RepoEvolver::with_deps(
            fixture.config.clone(),
            hybrid,
            launcher,
            provisioner,
            TestWorkerIdentity {
                uid: worker_uid,
                gid: worker_gid,
            },
            gzmo_evolver::SystemClock,
            roots.clone(),
            coordinator_uid,
        )
        .unwrap();

        let store = StateStore::open(fixture.config.state_dir()).unwrap();
        let initial_main = fixture.baseline_before.clone();
        let trusted_main_blob = rev_parse(&fixture.checkout, "refs/heads/main^{tree}");
        let remote_main_blob = rev_parse(&fixture.origin, "refs/heads/main^{tree}");
        assert_eq!(trusted_main_blob, remote_main_blob);

        Self {
            fixture,
            evolver,
            store,
            initial_main,
            roots,
            launches,
            provision_calls,
            env_names,
            trusted_main_blob,
            remote_main_blob,
        }
    }

    fn remote_main(&self) -> String {
        rev_parse(&self.fixture.origin, "refs/heads/main")
    }

    fn remote_branches(&self) -> Vec<String> {
        let out = Command::new("git")
            .args([
                "--git-dir",
                self.fixture.origin.to_str().unwrap(),
                "for-each-ref",
                "--format=%(refname:short)",
                "refs/heads",
            ])
            .env_clear()
            .env("PATH", "/usr/bin:/bin")
            .env("HOME", "/tmp")
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env("GIT_CONFIG_GLOBAL", "/dev/null")
            .output()
            .unwrap();
        assert!(out.status.success());
        String::from_utf8_lossy(&out.stdout)
            .lines()
            .map(|s| s.trim().to_owned())
            .filter(|s| !s.is_empty())
            .collect()
    }

    fn audit_states(&self) -> Vec<String> {
        let events = self.store.load_audit_events().unwrap();
        events
            .into_iter()
            .filter_map(|e| {
                e.event_type
                    .strip_prefix("candidate.")
                    .map(|s| s.to_owned())
            })
            .collect()
    }

    fn workspace_count(&self) -> usize {
        let dir = self.fixture.config.state_dir().join(WORKSPACES_DIR);
        if !dir.exists() {
            return 0;
        }
        fs::read_dir(&dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
            .count()
    }

    fn trusted_main_oid(&self) -> String {
        self.fixture.trusted_main_oid()
    }

    fn trusted_tree(&self) -> String {
        rev_parse(&self.fixture.checkout, "refs/heads/main^{tree}")
    }

    fn remote_tree(&self) -> String {
        rev_parse(&self.fixture.origin, "refs/heads/main^{tree}")
    }
}

#[tokio::test]
async fn fixture_run_reaches_evaluating_without_remote_mutation() {
    let harness = FixtureVertical::new().await;

    let outcome = run_once_retry_lock_busy(&harness.evolver).await.unwrap();
    assert_eq!(outcome.state, CandidateState::Evaluating);
    assert!(
        outcome
            .candidate_digest
            .as_deref()
            .unwrap()
            .starts_with("git-sha1:"),
        "candidate digest: {:?}",
        outcome.candidate_digest
    );
    assert!(outcome.receipt_digest.is_some());
    assert_eq!(*harness.launches.lock().unwrap(), 1);
    let provisions_after_first = *harness.provision_calls.lock().unwrap();
    assert!(provisions_after_first >= 1);

    // Exact audit chain Observed→Prepared→Building→Evaluating.
    assert!(harness.store.verify_audit_chain().is_ok());
    assert_eq!(
        harness.audit_states(),
        ["observed", "prepared", "building", "evaluating"]
    );

    // Remote + trusted main byte/OID stable; only main ref.
    assert_eq!(harness.remote_main(), harness.initial_main);
    assert_eq!(harness.trusted_main_oid(), harness.initial_main);
    assert_eq!(harness.trusted_tree(), harness.trusted_main_blob);
    assert_eq!(harness.remote_tree(), harness.remote_main_blob);
    assert_eq!(harness.remote_branches(), vec!["main".to_owned()]);

    // Independent clone: one normalized one-parent candidate commit, nonempty diff.
    let ws = PathBuf::from(outcome.workspace.as_ref().expect("workspace path"));
    assert!(ws.join(".git").exists());
    let head = rev_parse(&ws, "HEAD");
    let digest = outcome.candidate_digest.as_ref().unwrap();
    assert_eq!(digest, &format!("git-sha1:{head}"));
    let parents = {
        let out = Command::new("git")
            .args([
                "-C",
                ws.to_str().unwrap(),
                "rev-list",
                "--parents",
                "-n",
                "1",
                "HEAD",
            ])
            .env_clear()
            .env("PATH", "/usr/bin:/bin")
            .env("HOME", "/tmp")
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env("GIT_CONFIG_GLOBAL", "/dev/null")
            .output()
            .unwrap();
        assert!(out.status.success());
        String::from_utf8_lossy(&out.stdout)
            .split_whitespace()
            .map(|s| s.to_owned())
            .collect::<Vec<_>>()
    };
    assert_eq!(parents.len(), 2, "exactly one parent: {parents:?}");
    assert_eq!(parents[0], head);
    assert_eq!(parents[1], harness.initial_main);

    // Diff facts nonempty and match receipt.
    let numstat = {
        let range = format!("{}..{}", harness.initial_main, head);
        let out = Command::new("git")
            .args(["-C", ws.to_str().unwrap(), "diff", "--numstat", &range])
            .env_clear()
            .env("PATH", "/usr/bin:/bin")
            .env("HOME", "/tmp")
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env("GIT_CONFIG_GLOBAL", "/dev/null")
            .output()
            .unwrap();
        assert!(out.status.success());
        String::from_utf8(out.stdout).expect("numstat utf8")
    };
    assert!(!numstat.trim().is_empty(), "diff must be nonempty");
    let mut added_lines: u32 = 0;
    let mut changed_files: u32 = 0;
    for line in numstat.lines() {
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() >= 3 {
            changed_files += 1;
            if parts[0] != "-" {
                added_lines += parts[0].parse::<u32>().unwrap_or(0);
            }
        }
    }
    assert!(changed_files >= 1);
    assert!(added_lines >= 1);

    let store = StateStore::open(harness.fixture.config.state_dir()).unwrap();
    let repo = format!(
        "{}/{}",
        harness.fixture.config.repo().owner(),
        harness.fixture.config.repo().repository()
    );
    let record = store.latest_candidate(&repo).unwrap().unwrap();
    assert_eq!(record.state(), CandidateState::Evaluating);
    let receipt_json = record.worker_receipt_json().expect("receipt json");
    let receipt: serde_json::Value = serde_json::from_str(receipt_json).unwrap();
    assert_eq!(
        receipt["usage"]["changed_files"].as_u64().unwrap() as u32,
        changed_files
    );
    assert_eq!(
        receipt["usage"]["added_lines"].as_u64().unwrap() as u32,
        added_lines
    );
    let output_digest = receipt["output_digest"].as_str().unwrap();
    assert!(output_digest.starts_with("sha256:"));
    assert_eq!(output_digest.len(), "sha256:".len() + 64);

    // Fake worker saw exact allowlisted env names; no forbidden.
    let seen = harness.env_names.lock().unwrap().clone();
    assert!(!seen.is_empty(), "launcher must record env names");
    for f in gzmo_evolver::FORBIDDEN_ENV {
        assert!(
            !seen.iter().any(|n| n == *f),
            "forbidden env name present: {f}"
        );
    }
    assert!(!seen.iter().any(|n| n == "LOCAL_MODEL_BASE_URL"));
    let expected = {
        let home = PathBuf::from("/tmp/probe-home");
        let env = gzmo_evolver::omp_child_env(&home).unwrap();
        let mut keys: Vec<String> = env.keys().cloned().collect();
        keys.sort();
        keys
    };
    assert_eq!(seen, expected, "exact allowlisted env name set");

    // Mission generation binding: mission_id is UUID generation, not bet slug.
    assert_ne!(outcome.mission_id, "fixture-opportunity");
    assert!(
        outcome.mission_id.len() >= 32,
        "mission_id should be generation UUID: {}",
        outcome.mission_id
    );
    // Published CURRENT exists under missions/.
    let current = harness
        .fixture
        .config
        .state_dir()
        .join(gzmo_evolver::MISSIONS_DIR)
        .join(gzmo_evolver::CURRENT_POINTER);
    assert!(current.is_file());
    let gen = fs::read_to_string(&current).unwrap();
    assert_eq!(gen.trim(), outcome.mission_id);

    let ws_count = harness.workspace_count();
    assert_eq!(ws_count, 1);

    // Repeated run: no extra provision/launch/workspace/state/remote mutation.
    let again = run_once_retry_lock_busy(&harness.evolver).await.unwrap();
    assert_eq!(again.state, CandidateState::Evaluating);
    assert_eq!(again.candidate_id, outcome.candidate_id);
    assert_eq!(again.candidate_digest, outcome.candidate_digest);
    assert_eq!(again.receipt_digest, outcome.receipt_digest);
    assert_eq!(*harness.launches.lock().unwrap(), 1);
    assert_eq!(
        *harness.provision_calls.lock().unwrap(),
        provisions_after_first
    );
    assert_eq!(harness.workspace_count(), ws_count);
    assert_eq!(harness.remote_main(), harness.initial_main);
    assert_eq!(harness.trusted_main_oid(), harness.initial_main);
    assert_eq!(harness.remote_branches(), vec!["main".to_owned()]);
    assert_eq!(
        harness.audit_states(),
        ["observed", "prepared", "building", "evaluating"]
    );
}

/// Fixture mission script fails closed on two active bets (ok=false path).
#[test]
fn fixture_mission_script_rejects_two_active_bets() {
    let dir = TempDir::new().unwrap();
    let root = dir.path().join("repo");
    fs::create_dir_all(root.join("research/opportunities")).unwrap();
    fs::create_dir_all(root.join("scripts")).unwrap();
    let script_src = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/fixture-repo/scripts/opportunity-next-mission.sh");
    fs::copy(
        &script_src,
        root.join("scripts/opportunity-next-mission.sh"),
    )
    .unwrap();
    let mut perms = fs::metadata(root.join("scripts/opportunity-next-mission.sh"))
        .unwrap()
        .permissions();
    perms.set_mode(0o755);
    fs::set_permissions(root.join("scripts/opportunity-next-mission.sh"), perms).unwrap();

    for (name, id) in [("a.md", "bet-a"), ("b.md", "bet-b")] {
        fs::write(
            root.join("research/opportunities").join(name),
            format!("status: active\nid: {id}\ntitle: T\nscore: 1\nship_bar: true\n"),
        )
        .unwrap();
    }
    let staging = dir.path().join("staging");
    fs::create_dir_all(&staging).unwrap();
    let out = Command::new("bash")
        .arg("scripts/opportunity-next-mission.sh")
        .current_dir(&root)
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .env("HOME", dir.path())
        .env("GZMO_DATA_NEXT", &staging)
        .output()
        .unwrap();
    assert_ne!(out.status.code(), Some(0), "two-active must fail");
    let json = fs::read_to_string(staging.join("opportunity-discovery/next-mission.json")).unwrap();
    let v: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(v["ok"], false);
    assert!(
        v["advice"]
            .as_str()
            .unwrap_or("")
            .contains("need_exactly_one_active_bet"),
        "{json}"
    );
}

/// Trailing-whitespace candidate is rejected at Evaluating boundary.
#[tokio::test]
async fn whitespace_errors_in_candidate_diff_terminalize() {
    let harness = RepoHarness::new().await;
    // Replace launcher behavior: commit a trailing-whitespace file.
    *harness.launch_mode.lock().unwrap() = LaunchMode::Happy;
    // Inject via a one-shot custom path: prepare then mutate launcher mode is insufficient;
    // use a wrapper by writing after Happy would be too late. Instead seed Building with
    // a receipt whose workspace has trailing spaces and force finish via resume.
    let lock = CoordinatorLock::try_acquire(harness.fixture.config.state_dir()).unwrap();
    let store = StateStore::open(harness.fixture.config.state_dir()).unwrap();
    let clock = ManualClock::new(fixed_now());
    let fake = FakeProcessRunner::new();
    install_mission_fake(&fake, &clock);
    let hybrid = HybridRunner {
        git: harness.fixture.runner(),
        fake_mission: fake,
    };
    let prep = prepare_candidate(&harness.fixture.config, &hybrid, &clock, &store).unwrap();
    let id = prep.record.id().clone();
    let ws = prep.record.workspace().unwrap().to_path_buf();
    // Worker-shaped commit with trailing whitespace.
    fs::write(ws.join("ws-bad.txt"), "line with trailing space \n").unwrap();
    let status = Command::new("git")
        .args(["-C", ws.to_str().unwrap(), "add", "-A"])
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .env("HOME", &ws)
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .status()
        .unwrap();
    assert!(status.success());
    let status = Command::new("git")
        .args([
            "-C",
            ws.to_str().unwrap(),
            "-c",
            "user.name=fake",
            "-c",
            "user.email=fake@worker",
            "commit",
            "-m",
            "ws-bad",
        ])
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .env("HOME", &ws)
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .status()
        .unwrap();
    assert!(status.success());
    let head = rev_parse(&ws, "HEAD");
    store
        .transition(
            &id,
            CandidateState::Building,
            TransitionMetadata::empty(),
            fixed_now(),
        )
        .unwrap();
    drop(store);
    drop(lock);

    let sealed = seal_for_test(&harness, &prep);
    // Write receipt matching the whitespace commit (1 file, 1 line).
    let raw = br#"{"type":"session","version":3,"id":"fake-session"}
{"type":"tool_execution_start","toolCallId":"tool-1","toolName":"bash","args":{}}
{"type":"tool_execution_end","toolCallId":"tool-1","toolName":"bash","result":"ok"}
{"type":"message_end","message":{"role":"assistant","stopReason":"stop","usage":{"input":10,"output":5,"cacheRead":1,"cacheWrite":2,"totalTokens":18}}}
{"type":"agent_end","messages":[]}
"#;
    fs::write(sealed.output_dir().join("raw.jsonl"), raw).unwrap();
    let output_digest = format!("sha256:{}", sha256_hex(raw));
    let started = sealed.issued_at() + chrono::Duration::seconds(1);
    let completed = started + chrono::Duration::seconds(2);
    let usage = ResourceUsage {
        wall_seconds: 2,
        attempts: 1,
        changed_files: 1,
        added_lines: 1,
        tool_calls: 1,
        input_tokens: 10,
        output_tokens: 5,
        energy_joules: None,
    };
    let receipt = WorkerReceipt::new(
        sealed.candidate_id().clone(),
        sealed.manifest_digest(),
        sealed.policy_digest(),
        sealed.omp_version(),
        started,
        completed,
        0,
        output_digest,
        Some(format!("git-sha1:{head}")),
        usage,
    )
    .unwrap();
    fs::write(
        sealed.output_dir().join("receipt.json"),
        receipt.canonical_bytes().unwrap(),
    )
    .unwrap();
    let mut perms = fs::metadata(sealed.output_dir().join("receipt.json"))
        .unwrap()
        .permissions();
    perms.set_mode(0o600);
    fs::set_permissions(sealed.output_dir().join("receipt.json"), perms).unwrap();
    *harness.unit_state.lock().unwrap() = WorkerUnitState::Succeeded;
    let err = resume_retry_lock_busy(&harness.evolver).await.unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("whitespace"),
        "expected whitespace terminalization branch, got {msg}"
    );
    let store = StateStore::open(harness.fixture.config.state_dir()).unwrap();
    let latest = store
        .latest_candidate(&format!(
            "{}/{}",
            harness.fixture.config.repo().owner(),
            harness.fixture.config.repo().repository()
        ))
        .unwrap()
        .unwrap();
    assert_eq!(latest.state(), CandidateState::Failed);
    assert_eq!(
        latest.terminal_reason().as_deref(),
        Some("whitespace errors in candidate diff")
    );
}

#[tokio::test]
async fn observed_mirror_http_404_nonzero_terminalizes_not_contention() {
    let fixture = Fixture::new();
    let (evolver, _roots, id) =
        observed_evolver_with_mirror_fault(&fixture, MirrorNetworkFault::NotFoundHttpNonZero);
    let err = resume_retry_lock_busy(&evolver)
        .await
        .expect_err("404 must terminalize");
    assert_no_stderr_secret(&err);
    assert!(
        !matches!(err, gzmo_evolver::RunnerError::Contention(_)),
        "HTTP 404 must not be Contention, got {err}"
    );
    assert!(!err.to_string().contains("404"), "{err}");
    let store = StateStore::open(fixture.config.state_dir()).unwrap();
    let rec = store.load(&id).unwrap();
    assert_eq!(rec.state(), CandidateState::Failed);
    if let Some(reason) = rec.terminal_reason() {
        assert!(
            !reason.contains(MIRROR_STDERR_SECRET),
            "secret in terminal_reason: {reason}"
        );
    }
}
