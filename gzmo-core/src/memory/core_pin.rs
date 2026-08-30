//! CORE pin — Bonded Felt Use seed for crystallize / doctrine prefixes.
//!
//! Only allowlisted content gets Bonded-on-promote so ripen stays meaningful
//! for the long tail (recall still starts at 0 for normal ingest).

/// Default prefixes that may receive Bonded (+5) after honeypot promote.
pub const DEFAULT_PREFIXES: &[&str] = &["CoreCrystallize:", "[CORE]"];

/// Env kill-switch: `GZMO_CORE_PIN=0` disables Bonded seed.
pub fn enabled() -> bool {
    match std::env::var("GZMO_CORE_PIN") {
        Ok(v) => {
            let t = v.trim();
            !(t == "0" || t.eq_ignore_ascii_case("false") || t.eq_ignore_ascii_case("off"))
        }
        Err(_) => true,
    }
}

/// True when content should get a one-shot Bonded seed after honeypot land.
pub fn should_seed_bonded(content: &str, origin: &str) -> bool {
    if !enabled() {
        return false;
    }
    // Prefer nutrient origins that export accepts.
    let origin_ok = matches!(
        origin,
        "session_distill" | "ingest" | "verified_dream" | "operator_core"
    );
    if !origin_ok {
        return false;
    }

    let trimmed = content.trim();
    for &p in DEFAULT_PREFIXES {
        if let Some(idx) = trimmed.find(p) {
            let before = &trimmed[..idx];
            let after = &trimmed[idx + p.len()..];
            // Must have substantive content, not just be a bare prefix stub
            if !before.trim().is_empty() || !after.trim().is_empty() {
                return true;
            }
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_crystallize_prefix() {
        assert!(should_seed_bonded(
            "CoreCrystallize: [CONCEPT:X] hello",
            "session_distill"
        ));
        assert!(should_seed_bonded("[CORE] pinned", "ingest"));
        assert!(!should_seed_bonded("random fact", "session_distill"));
        assert!(!should_seed_bonded("CoreCrystallize: x", "spark"));
    }

    #[test]
    fn test_empty_and_bounds() {
        assert!(!should_seed_bonded("", "ingest"));
        assert!(!should_seed_bonded("   ", "ingest"));

        // Stub facts (empty payload) are refused
        assert!(!should_seed_bonded("[CORE]", "ingest"));
        assert!(!should_seed_bonded("  [CORE]  ", "ingest"));
        assert!(!should_seed_bonded("CoreCrystallize:", "ingest"));

        // Substantive bounds
        assert!(should_seed_bonded("[CORE] x", "ingest"));
        assert!(should_seed_bonded("x [CORE]", "ingest"));
        assert!(should_seed_bonded("CoreCrystallize: data", "ingest"));
    }
}
