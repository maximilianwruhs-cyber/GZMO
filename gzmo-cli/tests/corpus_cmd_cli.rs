//! Integration test: proves `gzmo corpus <unsupported-subcommand>` really
//! terminates the process with exit status 2 — the brief-mandated usage-error
//! exit code (`Unknown flags exit 2`), matching the existing
//! `session_cmd.rs`-style explicit `std::process::exit(2)` convention. This is
//! a genuine process-level assertion (not just a parser error message), since
//! `main() -> Result<()>` under `#[tokio::main]` would otherwise turn a
//! `bail!`-based usage error into exit code 1.

use std::process::Command;

#[test]
fn corpus_unsupported_subcommand_exits_with_status_2() {
    let exe = env!("CARGO_BIN_EXE_gzmo");

    let tmp_cwd = std::env::temp_dir().join(format!(
        "gzmo-cli-corpus-exit-code-cwd-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&tmp_cwd).expect("create isolated cwd");
    // Point at a file that does not exist so `GzmoConfig::load` falls back to
    // its documented zero-config defaults, independent of any real gzmo.toml
    // on this machine.
    let missing_config = tmp_cwd.join("does-not-exist.toml");

    let output = Command::new(exe)
        .args(["corpus", "bogus-subcommand"])
        .current_dir(&tmp_cwd)
        .env("GZMO_CONFIG", &missing_config)
        .output()
        .expect("spawn gzmo binary");

    assert_eq!(
        output.status.code(),
        Some(2),
        "expected exit 2 for an unsupported corpus subcommand; stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let _ = std::fs::remove_dir_all(&tmp_cwd);
}

#[test]
fn corpus_missing_subcommand_exits_with_status_2() {
    let exe = env!("CARGO_BIN_EXE_gzmo");

    let tmp_cwd = std::env::temp_dir().join(format!(
        "gzmo-cli-corpus-exit-code-nosub-cwd-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&tmp_cwd).expect("create isolated cwd");
    let missing_config = tmp_cwd.join("does-not-exist.toml");

    let output = Command::new(exe)
        .args(["corpus"])
        .current_dir(&tmp_cwd)
        .env("GZMO_CONFIG", &missing_config)
        .output()
        .expect("spawn gzmo binary");

    assert_eq!(
        output.status.code(),
        Some(2),
        "expected exit 2 when no corpus subcommand is given; stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let _ = std::fs::remove_dir_all(&tmp_cwd);
}

/// Behavioral proof that the *compiled* `gzmo corpus ingest-dir` process
/// itself — not just `corpus_cmd::run` in-process — exits nonzero on a
/// genuine runtime/index failure (unreachable embedder + Qdrant), and that
/// this is distinct from a usage error: it must NOT exit with usage code 2.
#[test]
fn corpus_ingest_dir_exits_nonzero_on_index_failure() {
    let exe = env!("CARGO_BIN_EXE_gzmo");

    let tmp_cwd = std::env::temp_dir().join(format!(
        "gzmo-cli-corpus-runtime-fail-cwd-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&tmp_cwd).expect("create isolated cwd");

    let corpus_dir = tmp_cwd.join("corpus");
    std::fs::create_dir_all(&corpus_dir).expect("create corpus dir");
    std::fs::write(corpus_dir.join("note.md"), "# Note\n\nSome corpus content.")
        .expect("write corpus file");

    // Deterministic failing condition: point embeddings + Qdrant at an
    // instantly-refused local port — the repo's established convention for
    // "unreachable service" tests (see gzmo-core/src/corpus/index.rs). All
    // other paths (vault_db, session_distill.sessions_dir) resolve relative
    // to this config file's directory, so they stay isolated in tmp_cwd.
    let config_path = tmp_cwd.join("gzmo.toml");
    std::fs::write(
        &config_path,
        "[embeddings]\nurl = \"http://127.0.0.1:1\"\n\n[qdrant]\nurl = \"http://127.0.0.1:1\"\n",
    )
    .expect("write gzmo.toml");

    let output = Command::new(exe)
        .args([
            "corpus",
            "ingest-dir",
            corpus_dir.to_str().expect("utf8 corpus path"),
        ])
        .current_dir(&tmp_cwd)
        .env("GZMO_CONFIG", &config_path)
        .output()
        .expect("spawn gzmo binary");

    assert!(
        !output.status.success(),
        "expected nonzero exit on index/runtime failure; stdout: {} stderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_ne!(
        output.status.code(),
        Some(2),
        "this is a runtime/index failure, not a usage error — must not reuse \
         usage exit code 2; stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let _ = std::fs::remove_dir_all(&tmp_cwd);
}
