//! Deterministic exporter for public evolution artifact JSON Schemas.
//!
//! Usage: export_schemas --out <directory>
//! Writes exactly five filenames in fixed order. Rejects unknown/extra args.

use evolution_contracts::export_all_schemas;
use std::env;
use std::path::PathBuf;
use std::process::ExitCode;

fn main() -> ExitCode {
    match run(env::args().skip(1).collect()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("export_schemas: {err}");
            ExitCode::FAILURE
        }
    }
}

fn run(args: Vec<String>) -> Result<(), String> {
    let out = parse_out(args)?;
    export_all_schemas(&out).map_err(|err| err.to_string())
}

fn parse_out(args: Vec<String>) -> Result<PathBuf, String> {
    if args.len() != 2 {
        return Err(
            "accepts only `--out <directory>` (exactly two argv tokens after program name)"
                .to_owned(),
        );
    }
    if args[0] != "--out" {
        return Err(format!("first argument must be --out, got {:?}", args[0]));
    }
    if args[1].is_empty() || args[1].starts_with('-') {
        return Err("`--out` requires a nonempty directory path".to_owned());
    }
    Ok(PathBuf::from(&args[1]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_extra_and_missing_args() {
        assert!(parse_out(vec![]).is_err());
        assert!(parse_out(vec!["--out".into()]).is_err());
        assert!(parse_out(vec!["--out".into(), "a".into(), "b".into()]).is_err());
        assert!(parse_out(vec!["--dir".into(), "a".into()]).is_err());
        assert_eq!(
            parse_out(vec!["--out".into(), "schemas".into()]).unwrap(),
            PathBuf::from("schemas")
        );
    }
}
