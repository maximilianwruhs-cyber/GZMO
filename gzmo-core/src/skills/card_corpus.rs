//! Persistent forge ledger — every accepted card is archived for audit and replay.

use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;

use anyhow::Result;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct ForgeCorpusEntry {
    pub inv: u64,
    pub tick: u64,
    pub name: String,
    pub cost: String,
    pub type_line: String,
    pub rarity: String,
    pub color: String,
    pub card_type: String,
    pub forge_mode: String,
    pub keyword_spark: String,
    pub subtype_hint: String,
    pub name_seed: String,
    pub set_code: String,
    pub body_hash: String,
}

pub fn corpus_path(data_dir: &Path) -> std::path::PathBuf {
    data_dir.join("skills").join("card_forge_corpus.jsonl")
}

pub fn append_forge(data_dir: &Path, entry: &ForgeCorpusEntry) -> Result<()> {
    let path = corpus_path(data_dir);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let line = serde_json::to_string(entry)?;
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)?;
    writeln!(file, "{line}")?;
    Ok(())
}
