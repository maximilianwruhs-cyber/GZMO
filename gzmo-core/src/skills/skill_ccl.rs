//! Chaos Coupling Level (CCL) registry per `docs/SKILL_GOLDEN_STANDARD.md`.

/// Chaos Coupling Level — how deeply a skill uses the Lorenz attractor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChaosCouplingLevel {
    /// Disconnected — ignores chaos (legacy shell-only).
    Ccl0 = 0,
    /// Passive — snapshot visible in display only.
    Ccl1 = 1,
    /// Indexed — `chaos_index()` / `chaos_roll()`.
    Ccl2 = 2,
    /// Coupled — chaos shapes prompts or mechanical outcomes.
    Ccl3 = 3,
    /// Autopoietic — reload, inv #, anti-repeat, cabinet echo, crystallize footer.
    Ccl4 = 4,
}

impl ChaosCouplingLevel {
    pub fn as_u8(self) -> u8 {
        self as u8
    }

    pub fn badge(self) -> &'static str {
        match self {
            Self::Ccl0 => "CCL-0",
            Self::Ccl1 => "CCL-1",
            Self::Ccl2 => "CCL-2",
            Self::Ccl3 => "CCL-3",
            Self::Ccl4 => "CCL-4",
        }
    }

    pub fn gold_star(self) -> &'static str {
        if self == Self::Ccl4 {
            "★"
        } else {
            ""
        }
    }

    /// Legendary skill marker (orthogonal to CCL — `/dice` and `/card`).
    pub fn legendary_mark(self, skill_name: &str) -> &'static str {
        if skill_name == "dice" || skill_name == "card" || skill_name == "pkm" {
            "◆"
        } else {
            ""
        }
    }
}

/// Canonical CCL assignment for the Rust pantheon (update when skills upgrade).
pub fn ccl_for_skill(name: &str) -> ChaosCouplingLevel {
    match name {
        "story" | "poem" | "joke" | "card" | "pkm" | "word" | "define" => ChaosCouplingLevel::Ccl4,
        "dice" | "transform" | "stabilize" | "ops" | "learn" | "discover" => {
            ChaosCouplingLevel::Ccl3
        }
        "poker" | "quote" | "sound" | "language" | "calculate" => ChaosCouplingLevel::Ccl2,
        "visual" => ChaosCouplingLevel::Ccl1,
        "help" => ChaosCouplingLevel::Ccl1,
        _ => ChaosCouplingLevel::Ccl0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ccl4_generative_quad() {
        assert_eq!(ccl_for_skill("story"), ChaosCouplingLevel::Ccl4);
        assert_eq!(ccl_for_skill("poem"), ChaosCouplingLevel::Ccl4);
        assert_eq!(ccl_for_skill("joke"), ChaosCouplingLevel::Ccl4);
        assert_eq!(ccl_for_skill("card"), ChaosCouplingLevel::Ccl4);
        assert_eq!(ccl_for_skill("pkm"), ChaosCouplingLevel::Ccl4);
        assert_eq!(ccl_for_skill("word"), ChaosCouplingLevel::Ccl4);
        assert_eq!(ccl_for_skill("define"), ChaosCouplingLevel::Ccl4);
    }

    #[test]
    fn dice_is_ccl3_and_legendary() {
        assert_eq!(ccl_for_skill("dice"), ChaosCouplingLevel::Ccl3);
        assert_eq!(ChaosCouplingLevel::Ccl3.legendary_mark("dice"), "◆");
        assert_eq!(ChaosCouplingLevel::Ccl3.legendary_mark("card"), "◆");
        assert_eq!(ChaosCouplingLevel::Ccl4.legendary_mark("card"), "◆");
        assert_eq!(ChaosCouplingLevel::Ccl4.legendary_mark("pkm"), "◆");
        assert_eq!(ChaosCouplingLevel::Ccl3.legendary_mark("story"), "");
    }

    #[test]
    fn calculate_is_ccl2() {
        assert_eq!(ccl_for_skill("calculate"), ChaosCouplingLevel::Ccl2);
    }
}
