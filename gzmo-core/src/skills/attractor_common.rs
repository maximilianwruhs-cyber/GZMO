//! Shared Attractor Fiction infrastructure for CCL-4 generative skills.

use std::fs::OpenOptions;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

use anyhow::Result;
use gzmo_chaos::chaos::Phase;
use gzmo_chaos::pulse::ChaosSnapshot;
use sha2::{Digest, Sha256};

use super::generative::chaos_index;

/// Concrete motifs used when a generative skill receives empty args.
pub const DEFAULT_SEED_POOL: &[&str] = &[
    "copper", "static", "fern", "delta", "mirror", "ash", "circuit", "tide", "ember", "glass",
    "rust", "orbit", "velvet", "granite", "prism", "socket", "vapor", "lichen", "gear", "echo",
    "signal", "harbor", "flint", "cinder", "thread", "anchor", "pulse", "moss", "wire", "storm",
];

pub fn resolve_chaos_seed(arg: &str, snap: &ChaosSnapshot, call_serial: u64) -> String {
    let trimmed = arg.trim();
    if trimmed.is_empty() {
        let idx = (chaos_index(snap, DEFAULT_SEED_POOL.len()) + call_serial as usize)
            % DEFAULT_SEED_POOL.len();
        DEFAULT_SEED_POOL[idx].to_string()
    } else {
        trimmed.to_string()
    }
}

pub fn select_chaos_mode<T: Copy>(snap: &ChaosSnapshot, modes: &[T]) -> T {
    let phase_offset = match snap.phase {
        Phase::Idle => 0,
        Phase::Build => 1,
        Phase::Drop => 2,
    };
    modes[(phase_offset + chaos_index(snap, modes.len())) % modes.len()]
}

pub fn normalize_fingerprint(text: &str) -> String {
    text.chars()
        .filter(|c| c.is_alphanumeric())
        .collect::<String>()
        .to_lowercase()
}

pub fn fingerprint_too_similar(candidate: &str, recent: &[String]) -> bool {
    let norm = normalize_fingerprint(candidate);
    if norm.len() < 6 {
        return false;
    }
    let prefix_len = norm.len().min(8);
    let prefix = &norm[..prefix_len];
    recent.iter().any(|r| {
        if r == &norm {
            return true;
        }
        if r.len() >= 8 && norm.len() >= 8 {
            r.starts_with(prefix) || norm.starts_with(&r[..8.min(r.len())])
        } else {
            let min_len = r.len().min(norm.len());
            if min_len >= 4 {
                r[..4] == norm[..4]
            } else {
                false
            }
        }
    })
}

pub fn opening_fingerprint(text: &str) -> String {
    let opening: String = text
        .lines()
        .next()
        .unwrap_or(text)
        .chars()
        .take(48)
        .collect();
    normalize_fingerprint(&opening)
}

pub fn themes_from_fingerprints(fingerprints: &[String], label: &str) -> Vec<String> {
    fingerprints
        .iter()
        .map(|f| format!("{label} '{f}' or very similar phrasing"))
        .collect()
}

pub fn record_fingerprint(ledger: &mut Vec<String>, path: &Path, fingerprint: String, max: usize) {
    ledger.push(fingerprint);
    if ledger.len() > max {
        ledger.remove(0);
    }
    let _ = save_recent_hashes(path, ledger);
}

#[derive(Debug, Clone)]
pub struct AttractorMeta {
    pub seed: String,
    pub tick: u64,
    pub phase: Phase,
    pub valence: f32,
    pub rho_effective: f64,
    pub call_serial: u64,
    pub nonce: u64,
    pub cabinet_echo: Option<String>,
    pub anti_repeat_hint: String,
}

pub struct AttractorPromptInput<'a> {
    pub seed_label: &'a str,
    pub seed: &'a str,
    pub snap: &'a ChaosSnapshot,
    pub recent_themes: &'a [String],
    pub call_serial: u64,
    pub attempt: u32,
    pub instant_nanos: u64,
    pub max_chars: usize,
    pub extra_rules: &'a [&'a str],
}

impl AttractorMeta {
    pub fn from_input(input: AttractorPromptInput<'_>) -> Self {
        let anti_repeat_hint = if input.recent_themes.is_empty() {
            String::new()
        } else {
            format!(
                "Avoid repeating these prior outputs: {}.",
                input.recent_themes.join(", ")
            )
        };

        Self {
            seed: input.seed.trim().to_string(),
            tick: input.snap.tick,
            phase: input.snap.phase,
            valence: input.snap.llm_valence,
            rho_effective: input.snap.rho_effective,
            call_serial: input.call_serial,
            nonce: build_nonce(
                input.seed,
                input.snap.tick,
                input.call_serial,
                input.attempt,
                input.instant_nanos,
            ),
            cabinet_echo: None,
            anti_repeat_hint,
        }
    }

    pub fn user_prompt(&self, seed_label: &str, max_chars: usize, extra_rules: &[&str]) -> String {
        let mut lines = Vec::new();
        lines.push(format!("{seed_label}: {}", self.seed));
        lines.push(format!(
            "Attractor state: tick {}, phase {}, valence {:.2}, rho {:.2}, invocation #{}",
            self.tick, self.phase, self.valence, self.rho_effective, self.call_serial
        ));
        lines.push(format!("Nonce: {} (unique per invocation)", self.nonce));

        if let Some(echo) = &self.cabinet_echo {
            lines.push(format!(
                "Incorporate or contrast this incubating thought: \"{echo}\""
            ));
        }

        if !self.anti_repeat_hint.is_empty() {
            lines.push(self.anti_repeat_hint.clone());
        }

        lines.push("Rules:".to_string());
        lines.push(format!("- Maximum {max_chars} characters."));
        for rule in extra_rules {
            lines.push(format!("- {rule}"));
        }
        lines
            .push("- Output ONLY the creative text, no title, no quotes, no markdown.".to_string());

        lines.join("\n")
    }
}

pub fn build_nonce(
    seed: &str,
    tick: u64,
    call_serial: u64,
    attempt: u32,
    instant_nanos: u64,
) -> u64 {
    let mut h = 0_u64;
    for b in seed.bytes() {
        h = h.wrapping_mul(31).wrapping_add(b as u64);
    }
    tick ^ h
        ^ call_serial.wrapping_mul(0x9E37_79B9_7F4A_7C15)
        ^ ((attempt as u64) << 32)
        ^ instant_nanos
}

pub fn next_call_serial(path: &Path) -> Result<u64> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let current = std::fs::read_to_string(path)
        .ok()
        .and_then(|s| s.trim().parse::<u64>().ok())
        .unwrap_or(0);
    let next = current.saturating_add(1);
    std::fs::write(path, format!("{next}\n"))?;
    Ok(next)
}

pub fn body_hash(text: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(text.trim().as_bytes());
    format!("{:x}", hasher.finalize())
}

pub fn load_recent_hashes(path: &Path) -> Vec<String> {
    if !path.exists() {
        return Vec::new();
    }
    let file = match std::fs::File::open(path) {
        Ok(f) => f,
        Err(_) => return Vec::new(),
    };
    BufReader::new(file)
        .lines()
        .map_while(Result::ok)
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty())
        .collect()
}

pub fn save_recent_hashes(path: &Path, hashes: &[String]) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(path)?;
    for h in hashes {
        writeln!(file, "{}", h)?;
    }
    Ok(())
}

pub fn format_attractor_display(
    title: &str,
    meta: &AttractorMeta,
    seed_label: &str,
    body: &str,
    incubation_ticks: u64,
    rho_delta: &str,
) -> String {
    let header = format!(
        "{title}\n  {seed_label}: {} · inv #{} · tick {} · phase {} · valence {:.2} · ρ {:.2}",
        meta.seed, meta.call_serial, meta.tick, meta.phase, meta.valence, meta.rho_effective
    );

    let footer = if let Some(echo) = &meta.cabinet_echo {
        let truncated = if echo.chars().count() > 40 {
            let taken: String = echo.chars().take(37).collect();
            format!("{taken}...")
        } else {
            echo.clone()
        };
        format!(
            "incubating echo: \"{truncated}\"\n  crystallize: ~{incubation_ticks} ticks → {rho_delta} ρ_mod"
        )
    } else {
        format!("crystallize: ~{incubation_ticks} ticks → {rho_delta} ρ_mod")
    };

    let indented: String = body
        .lines()
        .map(|l| format!("  {l}"))
        .collect::<Vec<_>>()
        .join("\n");

    format!(
        "\n┌─────────────────────────────────────────────────┐\n  {header}\n├─────────────────────────────────────────────────┤\n\n{indented}\n\n├─────────────────────────────────────────────────┤\n  {footer}\n└─────────────────────────────────────────────────┘\n"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use gzmo_chaos::pulse::ChaosSnapshot;

    #[test]
    fn nonce_varies_by_serial() {
        assert_ne!(
            build_nonce("wit", 10, 1, 1, 0),
            build_nonce("wit", 10, 2, 1, 0)
        );
    }

    #[test]
    fn resolve_chaos_seed_uses_pool_when_empty() {
        let snap = ChaosSnapshot::default();
        let seed = resolve_chaos_seed("", &snap, 1);
        assert!(DEFAULT_SEED_POOL.contains(&seed.as_str()));
    }

    #[test]
    fn resolve_chaos_seed_varies_by_call_serial() {
        let snap = ChaosSnapshot::default();
        let a = resolve_chaos_seed("", &snap, 1);
        let b = resolve_chaos_seed("", &snap, 2);
        assert_ne!(a, b);
    }

    #[test]
    fn fingerprint_catches_similar_prefix() {
        let recent = vec!["luvia".to_string()];
        assert!(fingerprint_too_similar("Luvielle", &recent));
        assert!(!fingerprint_too_similar("kraktic", &recent));
    }
}
