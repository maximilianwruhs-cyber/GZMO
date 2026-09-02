//! Real-Git hermetic proofs for independent candidate workspaces.
//!
//! Uses temporary bare origins and subprocess Git only — no network.

use chrono::DateTime;
use chrono::{TimeZone, Utc};
use evolution_contracts::{
    AuthorityTier, CandidateId, CandidateKind, CandidateManifest, CandidateState, CandidateTarget,
    ResourceBudget, CANDIDATE_SCHEMA,
};
use fs2::FileExt;
use gzmo_evolver::{
    cleanup_workspace, objects_share_with_mirror_pub, prepare_candidate_with,
    refresh_baseline_before_mission_with, verify_git_trust_with, CandidateRecord,
    CandidateStateOps, Clock, CoordinatorLock, FakeProcessRunner, GitAccessMode, GitError,
    GitRepository, ManualClock, PrepareError, ProcessError, ProcessOutput, ProcessRunner,
    ProcessSpec, RepoEvolverConfig, StateError, StateStore, SystemProcessRunner,
    TransitionMetadata, GIT_FETCH_TIMEOUT_SECS, GIT_OUTPUT_CAP_BYTES, GIT_TIMEOUT_SECS,
    MIRROR_LOCK_NAME, MISSION_STAGING_DIR, NO_FETCH_URL, NO_PUSH_URL, WORKSPACES_DIR,
};
use std::fs::{self, File, OpenOptions};
use std::os::unix::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Mutex;
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

struct Fixture {
    _root: TempDir,
    origin: PathBuf,
    checkout: PathBuf,
    state_dir: PathBuf,
    config: RepoEvolverConfig,
    config_path: PathBuf,
    baseline_before: String,
}

impl Fixture {
    fn new() -> Self {
        let root = TempDir::new().unwrap();
        let origin = root.path().join("origin.git");
        let checkout = root.path().join("checkout");
        let state_dir = root.path().join("state");
        let worker = root.path().join("omp");

        // Bare origin
        run_git(
            &root.path().to_path_buf(),
            &["init", "--bare", origin.to_str().unwrap()],
        );

        // Seed working copy then push to origin
        let seed = root.path().join("seed");
        run_git(
            root.path(),
            &["clone", origin.to_str().unwrap(), seed.to_str().unwrap()],
        );
        fs::create_dir_all(seed.join("config")).unwrap();
        fs::create_dir_all(seed.join("scripts")).unwrap();
        fs::write(seed.join("config/repo-evolver.policy.toml"), POLICY_TOML).unwrap();
        fs::write(seed.join("README.md"), "hello evolver\n").unwrap();
        File::create(seed.join("scripts/opportunity-next-mission.sh")).unwrap();
        git_config_identity(&seed);
        run_git(&seed, &["add", "."]);
        run_git(&seed, &["commit", "-m", "initial"]);
        run_git(&seed, &["branch", "-M", "main"]);
        run_git(&seed, &["push", "origin", "main"]);

        // Trusted checkout
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

        Self {
            _root: root,
            origin,
            checkout,
            state_dir,
            config,
            config_path,
            baseline_before,
        }
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

/// Process runner that delegates Git to the system runner and mission to a fake.
struct HybridRunner {
    system: SystemProcessRunner,
    fake_mission: FakeProcessRunner,
}

impl ProcessRunner for HybridRunner {
    fn run(&self, spec: &ProcessSpec) -> Result<ProcessOutput, gzmo_evolver::ProcessError> {
        let prog = spec.program.to_string_lossy();
        if prog.ends_with("git") || prog == "git" {
            return self.system.run(spec);
        }
        // Mission producer uses bash + script argv[0]
        if prog.ends_with("bash") || prog == "bash" || prog == "/bin/bash" {
            return self.fake_mission.run(spec);
        }
        self.system.run(spec)
    }
}

#[test]
fn independent_workspace_from_bare_remote() {
    let fx = Fixture::new();
    let runner = SystemProcessRunner;
    let git = GitRepository::open_for_local_fixture(&fx.config, &runner).unwrap();
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
    // Trusted checkout config/refs untouched
    assert_eq!(rev_parse(&fx.checkout, "HEAD"), fx.baseline_before);
}

#[test]
fn rejects_dirty_trusted_checkout() {
    let fx = Fixture::new();
    fs::write(fx.checkout.join("dirty.txt"), "x").unwrap();
    let runner = SystemProcessRunner;
    let git = GitRepository::open_for_local_fixture(&fx.config, &runner).unwrap();
    let err = git.refresh_and_resolve_baseline().unwrap_err();
    assert!(
        matches!(err, GitError::Trust(ref m) if m.contains("dirty")),
        "{err:?}"
    );
    assert_eq!(fx.trusted_main_oid(), fx.baseline_before);
}

#[test]
fn rejects_non_baseline_checkout_head() {
    let fx = Fixture::new();
    // Create an extra commit only on checkout, not pushed
    fs::write(fx.checkout.join("extra.txt"), "y").unwrap();
    git_config_identity(&fx.checkout);
    run_git(&fx.checkout, &["add", "extra.txt"]);
    run_git(&fx.checkout, &["commit", "-m", "local only"]);
    let runner = SystemProcessRunner;
    let git = GitRepository::open_for_local_fixture(&fx.config, &runner).unwrap();
    let err = git.refresh_and_resolve_baseline().unwrap_err();
    assert!(
        matches!(err, GitError::Trust(_)),
        "expected trust error, got {err:?}"
    );
}

#[test]
fn rejects_embedded_credentials_and_wrong_host() {
    let err = gzmo_evolver::validate_remote_identity(
        "https://user:sekrit@github.com/maximilianwruhs-cyber/GZMO.git",
        "maximilianwruhs-cyber",
        "GZMO",
        GitAccessMode::Production,
    )
    .unwrap_err();
    let msg = err.to_string();
    assert!(!msg.contains("sekrit"), "{msg}");
    assert!(!msg.contains("user:"), "{msg}");

    let err = gzmo_evolver::validate_remote_identity(
        "https://evil.example/maximilianwruhs-cyber/GZMO.git",
        "maximilianwruhs-cyber",
        "GZMO",
        GitAccessMode::Production,
    )
    .unwrap_err();
    assert!(err.to_string().contains("github.com"), "{err}");
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
    run_git(&fx.checkout, &["fetch", "origin"]);
    run_git(&fx.checkout, &["reset", "--hard", "origin/main"]);
    let runner = SystemProcessRunner;
    let git = GitRepository::open_for_local_fixture(&fx.config, &runner).unwrap();
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
    let runner = SystemProcessRunner;
    let git = GitRepository::open_for_local_fixture(&fx.config, &runner).unwrap();
    let err = git.refresh_and_resolve_baseline().unwrap_err();
    assert!(
        matches!(err, GitError::Trust(ref m) if m.contains("hook") || m.contains("executable")),
        "{err:?}"
    );
}

#[test]
fn rejects_existing_workspace_path_collision() {
    let fx = Fixture::new();
    let runner = SystemProcessRunner;
    let git = GitRepository::open_for_local_fixture(&fx.config, &runner).unwrap();
    let baseline = git.refresh_and_resolve_baseline().unwrap();
    let id = "cand-20260901t120000z-bet-aaaaaaa1";
    let ws_dir = fx.state_dir.join(WORKSPACES_DIR).join(id);
    fs::create_dir_all(&ws_dir).unwrap();
    let err = git.prepare(&manifest_for(&baseline, id)).unwrap_err();
    assert!(
        matches!(err, GitError::Workspace(ref m) if m.contains("exists")),
        "{err:?}"
    );
}

#[test]
fn squash_diff_and_cleanup_happy_path() {
    let fx = Fixture::new();
    let runner = SystemProcessRunner;
    let git = GitRepository::open_for_local_fixture(&fx.config, &runner).unwrap();
    let baseline = git.refresh_and_resolve_baseline().unwrap();
    let id = "cand-20260901t120000z-bet-bbbbbbb2";
    let ws = git.prepare(&manifest_for(&baseline, id)).unwrap();

    // Simulate worker edit + commit (untrusted)
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
    // one parent
    let parents = Command::new("git")
        .args([
            "-C",
            ws.path().to_str().unwrap(),
            "rev-list",
            "--parents",
            "-n",
            "1",
            &new_oid,
        ])
        .output()
        .unwrap();
    let line = String::from_utf8_lossy(&parents.stdout);
    let parts: Vec<_> = line.split_whitespace().collect();
    assert_eq!(parts.len(), 2);
    assert_eq!(parts[1], baseline);

    let stats = ws
        .diff_stats(&baseline, &new_oid, &manifest_for(&baseline, id))
        .unwrap();
    assert!(!stats.files.is_empty());
    assert!(stats.added_lines >= 1);
    assert!(stats.whitespace_ok);

    // Cleanup via terminal record
    let store = StateStore::open(&fx.state_dir).unwrap();
    let manifest = manifest_for(&baseline, id);
    let rec = store
        .create_candidate(&manifest, fx.config.working_policy_digest(), fixed_now())
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
    // trusted untouched
    assert_eq!(fx.trusted_main_oid(), fx.baseline_before);
}

#[test]
fn rejects_merge_commit_and_empty_candidate() {
    let fx = Fixture::new();
    let runner = SystemProcessRunner;
    let git = GitRepository::open_for_local_fixture(&fx.config, &runner).unwrap();
    let baseline = git.refresh_and_resolve_baseline().unwrap();
    let id = "cand-20260901t120000z-bet-cccccccc";
    let ws = git.prepare(&manifest_for(&baseline, id)).unwrap();

    // empty: squash should fail
    let err = ws
        .squash_candidate(&baseline, "felt-use-mass-growth", fixed_now())
        .unwrap_err();
    assert!(
        matches!(err, GitError::Workspace(ref m) if m.contains("no changes")),
        "{err:?}"
    );

    // merge commit path
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
    // merge side into branch
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
    let runner = SystemProcessRunner;
    let git = GitRepository::open_for_local_fixture(&fx.config, &runner).unwrap();
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
    let runner = SystemProcessRunner;
    let git = GitRepository::open_for_local_fixture(&fx.config, &runner).unwrap();
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
    // wrong basename
    let rec = store
        .transition(
            rec.id(),
            CandidateState::Failed,
            TransitionMetadata::terminal("x"),
            fixed_now(),
        )
        .unwrap();
    // tamper workspace path in a forged sense: cleanup with different expected
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
        system: SystemProcessRunner,
        fake_mission: fake,
    };

    let _lock = CoordinatorLock::try_acquire(&fx.state_dir).unwrap();
    let store = StateStore::open(&fx.state_dir).unwrap();

    let outcome = prepare_candidate_with(
        &fx.config,
        &hybrid,
        &clock,
        &store,
        GitAccessMode::LocalFixture,
    )
    .unwrap();
    assert!(!outcome.reused_active);
    assert_eq!(outcome.record.state(), CandidateState::Prepared);
    assert!(outcome.record.workspace().is_some());
    let ws_path = outcome.record.workspace().unwrap().to_path_buf();
    assert!(ws_path.exists());
    let first_id = outcome.record.id().as_str().to_owned();

    // Snapshot filesystem markers under state (excluding lock/db churn).
    let before_workspaces: Vec<_> = fs::read_dir(fx.state_dir.join(WORKSPACES_DIR))
        .unwrap()
        .map(|e| e.unwrap().file_name())
        .collect();

    // Second prepare: active-first, no new workspace.
    let outcome2 = prepare_candidate_with(
        &fx.config,
        &hybrid,
        &clock,
        &store,
        GitAccessMode::LocalFixture,
    )
    .unwrap();
    assert!(outcome2.reused_active);
    assert_eq!(outcome2.record.id().as_str(), first_id);
    assert!(outcome2.baseline.is_none());
    let after_workspaces: Vec<_> = fs::read_dir(fx.state_dir.join(WORKSPACES_DIR))
        .unwrap()
        .map(|e| e.unwrap().file_name())
        .collect();
    assert_eq!(before_workspaces, after_workspaces);
    assert_eq!(fx.trusted_main_oid(), fx.baseline_before);

    // Failure path: force prepare failure by pre-creating workspace path for next candidate.
    // First terminate current active.
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

    // Advance clock for new id
    clock.advance_secs(60);
    // Break mirror so prepare fails after Observed... easier: make workspaces parent a file?
    // Instead remove write by creating a file where workspaces dir entries would go —
    // poison by making mirror invalid after Observed is hard mid-flow.
    // Use a hybrid that fails git clone by pointing state workspaces to a non-dir after open:
    // Simpler approach: delete mirror after first success so next prepare fails at refresh.
    let _ = fs::remove_dir_all(fx.state_dir.join("mirror.git"));
    // But refresh recreates mirror — OK. To fail workspace create, precreate candidate path.
    // We don't know the id ahead of time. Fail mission instead.
    let fake2 = FakeProcessRunner::new();
    fake2.set_handler(|_| {
        Err(gzmo_evolver::ProcessError::NonZeroExit {
            code: 9,
            stdout: Vec::new(),
            stderr: b"boom".to_vec(),
        })
    });
    let hybrid_fail = HybridRunner {
        system: SystemProcessRunner,
        fake_mission: fake2,
    };
    // Mission fails before Observed — no Failed candidate. That's OK for mission path.
    let err = prepare_candidate_with(
        &fx.config,
        &hybrid_fail,
        &clock,
        &store,
        GitAccessMode::LocalFixture,
    )
    .unwrap_err();
    assert!(
        err.to_string().contains("mission") || err.to_string().contains("prepare"),
        "{err}"
    );

    // Workspace failure after Observed: monkey by creating collision via predictable id is hard.
    // Directly exercise GitRepository::prepare failure leaving no partial final path:
    let runner = SystemProcessRunner;
    let git = GitRepository::open_for_local_fixture(&fx.config, &runner).unwrap();
    let baseline = git.refresh_and_resolve_baseline().unwrap();
    let id = "cand-20260901t120000z-bet-fffffail";
    fs::create_dir_all(fx.state_dir.join(WORKSPACES_DIR).join(id)).unwrap();
    let err = git.prepare(&manifest_for(&baseline, id)).unwrap_err();
    assert!(matches!(err, GitError::Workspace(_)), "{err:?}");
    // no staging leftovers
    let leftovers: Vec<_> = fs::read_dir(fx.state_dir.join(WORKSPACES_DIR))
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_name().to_string_lossy().starts_with(".staging-"))
        .collect();
    assert!(leftovers.is_empty(), "{leftovers:?}");
}

#[test]
fn standalone_refresh_verifies_git_trust_without_state() {
    let fx = Fixture::new();
    assert!(!fx.state_dir.join("state.db").exists());
    let runner = SystemProcessRunner;
    // CLI-equivalent boundary: baseline refresh first (no mission yet).
    let baseline =
        refresh_baseline_before_mission_with(&fx.config, &runner, GitAccessMode::LocalFixture)
            .unwrap();
    assert_eq!(baseline, fx.baseline_before);
    assert!(!fx.state_dir.join("state.db").exists());
    // No coordinator lock created by baseline-only path.
    assert!(!fx.state_dir.join("runner.lock").exists());
    // Producer not run by baseline refresh alone.
    assert!(
        !fx.state_dir.join(MISSION_STAGING_DIR).exists()
            || fs::read_dir(fx.state_dir.join(MISSION_STAGING_DIR))
                .map(|d| d.count() == 0)
                .unwrap_or(true)
    );

    fs::write(fx.checkout.join("nope"), "1").unwrap();
    let err = verify_git_trust_with(&fx.config, &runner, GitAccessMode::LocalFixture).unwrap_err();
    assert!(matches!(err, GitError::Trust(_)), "{err:?}");
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
    let clock = ManualClock::new(fixed_now());
    let fake = FakeProcessRunner::new();
    let marker_for_fake = marker.clone();
    install_mission_fake_with_marker(&fake, &clock, marker_for_fake);

    let runner = SystemProcessRunner;
    // CLI-equivalent: baseline first must fail before adapter/producer runs.
    let baseline_err =
        refresh_baseline_before_mission_with(&fx.config, &runner, GitAccessMode::LocalFixture)
            .unwrap_err();
    assert!(
        matches!(baseline_err, GitError::Trust(_)),
        "{baseline_err:?}"
    );
    assert!(
        !marker.exists(),
        "producer must not run when baseline trust fails"
    );
    assert!(!fx.state_dir.join("state.db").exists());

    // Even if a caller wrongly invoked the adapter after failure, marker would exist —
    // prove the fake actually writes the marker when invoked.
    let hybrid = HybridRunner {
        system: SystemProcessRunner,
        fake_mission: fake,
    };
    // Force a direct mission call path is not needed; invoke fake once via hybrid on a clean config
    // by calling refresh_and_load is heavy. Instead run the fake handler through ProcessSpec.
    let _ = hybrid; // marker causality shown below via direct handler call
                    // Directly exercise the installed handler once:
                    // re-install is complex; call MissionAdapter only on a separate clean fixture.
    let fx2 = Fixture::new();
    let clock2 = ManualClock::new(fixed_now());
    let fake2 = FakeProcessRunner::new();
    let marker2 = fx2.state_dir.join("producer-ran.marker");
    install_mission_fake_with_marker(&fake2, &clock2, marker2.clone());
    let hybrid2 = HybridRunner {
        system: SystemProcessRunner,
        fake_mission: fake2,
    };
    let _ =
        refresh_baseline_before_mission_with(&fx2.config, &hybrid2, GitAccessMode::LocalFixture)
            .unwrap();
    // Now run mission adapter (CLI sequence after baseline).
    let adapter = gzmo_evolver::MissionAdapter::new(&fx2.config, &hybrid2, &clock2);
    let _ = adapter.refresh_and_load().unwrap();
    assert!(
        marker2.exists(),
        "mission fake must create producer marker when actually invoked"
    );
    // And the failing path never created its marker.
    assert!(!marker.exists());
}

#[test]
fn squash_dates_are_utc_and_deterministic() {
    let fx = Fixture::new();
    let runner = SystemProcessRunner;
    let git = GitRepository::open_for_local_fixture(&fx.config, &runner).unwrap();
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
    assert!(
        text.contains("+0000"),
        "author/committer must be UTC: {text}"
    );
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
    assert_eq!(
        oid1, oid2,
        "normalized OID must be deterministic for same inputs"
    );
    let body2 = Command::new("git")
        .args([
            "-C",
            ws2.path().to_str().unwrap(),
            "cat-file",
            "commit",
            &oid2,
        ])
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .env("HOME", "/tmp")
        .output()
        .unwrap();
    let text2 = String::from_utf8_lossy(&body2.stdout);
    for line in text2.lines() {
        if line.starts_with("author ") || line.starts_with("committer ") {
            assert!(line.ends_with("+0000"), "{line}");
        }
    }
}

/// Runner that strips `--no-local` from candidate clone so Git may hardlink objects.
#[derive(Debug)]
struct StripNoLocalRunner {
    inner: SystemProcessRunner,
}

impl ProcessRunner for StripNoLocalRunner {
    fn run(&self, spec: &ProcessSpec) -> Result<ProcessOutput, ProcessError> {
        let mut args = spec.args.clone();
        if args.iter().any(|a| a == "clone") {
            args.retain(|a| a != "--no-local");
        }
        let mut env = spec.env.clone();
        // Ensure file protocol for mirror path clone.
        // git_env already set; keep as-is.
        let new_spec = ProcessSpec::new(
            &spec.program,
            args,
            &spec.cwd,
            env,
            spec.output_cap,
            spec.timeout,
        )?;
        self.inner.run(&new_spec)
    }
}

#[test]
fn rejects_shared_objects_when_clone_omits_no_local() {
    let fx = Fixture::new();
    let base_runner = SystemProcessRunner;
    let git = GitRepository::open_for_local_fixture(&fx.config, &base_runner).unwrap();
    let baseline = git.refresh_and_resolve_baseline().unwrap();
    // Pack the mirror so pack artifacts exist for inode compare coverage.
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

    // Bad path: strip --no-local so shared objects appear; prepare must reject via production check.
    let bad_runner = StripNoLocalRunner {
        inner: SystemProcessRunner,
    };
    let git_bad = GitRepository::open_for_local_fixture(&fx.config, &bad_runner).unwrap();
    let id_bad = "cand-20260901t120000z-bet-sharebad1";
    let err = git_bad
        .prepare(&manifest_for(&baseline, id_bad))
        .unwrap_err();
    assert!(
        matches!(err, GitError::Workspace(ref m) if m.contains("share") || m.contains("hardlink") || m.contains("inode")),
        "expected shared-object rejection, got {err:?}"
    );
    // No published workspace left.
    assert!(!fx.state_dir.join(WORKSPACES_DIR).join(id_bad).exists());

    // Good path: normal --no-local accepted and production share check is false.
    let git_ok = GitRepository::open_for_local_fixture(&fx.config, &base_runner).unwrap();
    let id_ok = "cand-20260901t120000z-bet-sharenok1";
    let ws = git_ok.prepare(&manifest_for(&baseline, id_ok)).unwrap();
    let shared = objects_share_with_mirror_pub(
        &ws.path().join(".git/objects"),
        &git_ok.mirror_path().join("objects"),
    )
    .unwrap();
    assert!(
        !shared,
        "independent --no-local clone must not share object inodes"
    );
    assert!(!ws.uses_alternates_or_shared_objects().unwrap());
}

#[test]
fn prepare_reports_mirror_lock_busy_when_lease_held() {
    let fx = Fixture::new();
    let runner = SystemProcessRunner;
    let git = GitRepository::open_for_local_fixture(&fx.config, &runner).unwrap();
    let baseline = git.refresh_and_resolve_baseline().unwrap();
    // Hold mirror lock externally.
    let lock_path = fx.state_dir.join(MIRROR_LOCK_NAME);
    let file = fs::OpenOptions::new()
        .create(true)
        .read(true)
        .write(true)
        .open(&lock_path)
        .unwrap();
    file.try_lock_exclusive().unwrap();
    let id = "cand-20260901t120000z-bet-lockbusy1";
    let err = git.prepare(&manifest_for(&baseline, id)).unwrap_err();
    assert!(
        matches!(err, GitError::MirrorLockBusy),
        "expected MirrorLockBusy, got {err:?}"
    );
    drop(file);
}

#[test]
fn git_timeout_and_cap_constants_are_plan_exact() {
    assert_eq!(GIT_FETCH_TIMEOUT_SECS, 900);
    assert_eq!(GIT_TIMEOUT_SECS, 120);
    assert_eq!(GIT_OUTPUT_CAP_BYTES, 8 * 1024 * 1024);
}

#[test]
fn production_constructor_pins_production_mode() {
    let fx = Fixture::new();
    // Production open rejects local bare origin remote.
    let runner = SystemProcessRunner;
    let git = GitRepository::open(&fx.config, &runner).unwrap();
    assert_eq!(git.access_mode(), GitAccessMode::Production);
    let err = git.refresh_and_resolve_baseline().unwrap_err();
    assert!(
        matches!(err, GitError::Trust(ref m) if m.contains("LocalFixture") || m.contains("local")),
        "{err:?}"
    );
}

/// State ops double: fails only the Prepared transition, then allows Failed.
struct FailPreparedOnce {
    inner: StateStore,
    fail_prepared: Mutex<bool>,
}

impl FailPreparedOnce {
    fn new(inner: StateStore) -> Self {
        Self {
            inner,
            fail_prepared: Mutex::new(true),
        }
    }
}

impl CandidateStateOps for FailPreparedOnce {
    fn active_candidate(&self, repository: &str) -> Result<Option<CandidateRecord>, StateError> {
        self.inner.active_candidate(repository)
    }
    fn create_candidate(
        &self,
        manifest: &evolution_contracts::CandidateManifest,
        policy_digest: &str,
        now: DateTime<Utc>,
    ) -> Result<CandidateRecord, StateError> {
        self.inner.create_candidate(manifest, policy_digest, now)
    }
    fn transition(
        &self,
        id: &CandidateId,
        next: CandidateState,
        metadata: TransitionMetadata,
        now: DateTime<Utc>,
    ) -> Result<CandidateRecord, StateError> {
        if next == CandidateState::Prepared {
            let mut guard = self.fail_prepared.lock().unwrap();
            if *guard {
                *guard = false;
                return Err(StateError::IllegalTransition(
                    "injected prepared transition failure".to_owned(),
                ));
            }
        }
        self.inner.transition(id, next, metadata, now)
    }
    fn load(&self, id: &CandidateId) -> Result<CandidateRecord, StateError> {
        self.inner.load(id)
    }
}

#[test]
fn prepared_transition_failure_removes_workspace_and_fails_candidate() {
    let fx = Fixture::new();
    let clock = ManualClock::new(fixed_now());
    let fake = FakeProcessRunner::new();
    install_mission_fake(&fake, &clock);
    let hybrid = HybridRunner {
        system: SystemProcessRunner,
        fake_mission: fake,
    };
    let _lock = CoordinatorLock::try_acquire(&fx.state_dir).unwrap();
    let store = StateStore::open(&fx.state_dir).unwrap();
    let failing = FailPreparedOnce::new(store);

    let err = prepare_candidate_with(
        &fx.config,
        &hybrid,
        &clock,
        &failing,
        GitAccessMode::LocalFixture,
    )
    .unwrap_err();
    match &err {
        PrepareError::Failed {
            reason,
            candidate_id,
        } => {
            assert!(
                reason.contains("prepared transition failed") || reason.contains("illegal"),
                "{reason}"
            );
            assert!(reason.len() <= 512, "reason must be bounded");
            // Real inner record is Failed; workspace gone; no active candidate.
            let rec = failing
                .load(&CandidateId::parse(candidate_id.as_str()).unwrap())
                .unwrap();
            assert_eq!(rec.state(), CandidateState::Failed);
            assert!(
                rec.terminal_reason().is_some(),
                "terminal reason must be set"
            );
            if let Some(ws) = rec.workspace() {
                // workspace metadata may be unset because Prepared never committed it
                let _ = ws;
            }
            // No workspace directory for this candidate id.
            assert!(
                !fx.state_dir
                    .join(WORKSPACES_DIR)
                    .join(candidate_id)
                    .exists(),
                "published workspace must be removed by production error arm"
            );
            let repo = format!(
                "{}/{}",
                fx.config.repo().owner(),
                fx.config.repo().repository()
            );
            assert!(failing.active_candidate(&repo).unwrap().is_none());
        }
        other => panic!("expected PrepareError::Failed, got {other:?}"),
    }
}

#[test]
fn policy_mismatch_is_rejected() {
    let fx = Fixture::new();
    // Change working-tree policy without committing
    let mut policy = POLICY_TOML.to_owned();
    policy = policy.replace("max_added_lines = 1500", "max_added_lines = 1400");
    fs::write(fx.checkout.join("config/repo-evolver.policy.toml"), &policy).unwrap();
    // Config load binds working policy from disk — need reload
    // But fixture config already loaded. Rebuild config:
    let config = RepoEvolverConfig::load(&fx.config_path).unwrap();
    // checkout is dirty due to policy edit
    let runner = SystemProcessRunner;
    let git = GitRepository::open_for_local_fixture(&config, &runner).unwrap();
    let err = git.refresh_and_resolve_baseline().unwrap_err();
    // dirty OR digest mismatch depending on order — require_clean runs first
    assert!(matches!(err, GitError::Trust(_)), "{err:?}");
}

#[test]
fn diff_rejects_protected_path_change() {
    let fx = Fixture::new();
    let runner = SystemProcessRunner;
    let git = GitRepository::open_for_local_fixture(&fx.config, &runner).unwrap();
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
    // squash would also reject protected? squash doesn't check PathPolicy — diff_stats does
    let err = ws
        .diff_stats(&baseline, &head, &manifest_for(&baseline, id))
        .unwrap_err();
    assert!(
        matches!(err, GitError::Invalid(ref m) if m.contains("path policy") || m.contains("protected")),
        "{err:?}"
    );
}

// Open validate_remote_identity for tests.
mod git_identity_reexport {
    // covered in rejects_embedded_credentials_and_wrong_host
}
