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
