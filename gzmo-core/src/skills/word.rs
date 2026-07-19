use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Result;
use async_trait::async_trait;

use gzmo_chaos::feedback::ChaosEvent;

use super::attractor_common::{
    body_hash, fingerprint_too_similar, format_attractor_display, load_recent_hashes,
    next_call_serial, normalize_fingerprint, record_fingerprint, resolve_chaos_seed,
    save_recent_hashes, themes_from_fingerprints,
};
use super::dispatch::{data_dir_from_skills, load_live_chaos_snapshot};
use super::generative::{
    accept_creative, clean_llm_output, line_value, llm_complete, persona_constraint_gate,
    quality_gate_word, require_gateway,
};
use super::word_brief::{WordBrief, WordBriefInput};
use super::{Skill, SkillContext, SkillOutput, SkillType};

pub struct WordSkill;

fn extract_word_candidate(text: &str) -> Option<String> {
    line_value(text, "WORD:").map(|w| w.split('(').next().unwrap_or(w).trim().to_string())
}

#[async_trait]
impl Skill for WordSkill {
    fn name(&self) -> &str {
        "word"
    }
    fn description(&self) -> &str {
        "Invent a brand new word with definition and example sentence (live Lorenz coordinates)"
    }
    fn skill_type(&self) -> SkillType {
        SkillType::Generative
    }

    async fn execute(&self, ctx: SkillContext<'_>) -> Result<SkillOutput> {
        let gw = require_gateway(&ctx)?;
        let data_dir = data_dir_from_skills(ctx.skills_dir);
        let skills_data = data_dir.join("skills");
        let ledger_path = skills_data.join(".word_recent_hashes");
        let words_path = skills_data.join(".word_recent_words");
        let serial_path = skills_data.join(".word_call_serial");

        let call_serial = next_call_serial(&serial_path)?;
        let mut recent_hashes = load_recent_hashes(&ledger_path);
        let mut recent_words = load_recent_hashes(&words_path);
        let mut word_data = None;
        let mut recent_themes = themes_from_fingerprints(&recent_words, "the invented word");
        let persona_gate = persona_constraint_gate(ctx.skills_dir);
        let arg_seed = ctx.args.trim().to_string();

        for attempt in 1..=3 {
            let snap = load_live_chaos_snapshot(&data_dir, ctx.chaos);
            gw.set_chaos_overrides(snap.llm_temperature, snap.llm_max_tokens);

            let seed = resolve_chaos_seed(&arg_seed, &snap, call_serial);

            let instant_nanos = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_nanos() as u64)
                .unwrap_or(0);

            let brief = WordBrief::new(WordBriefInput {
                seed: &seed,
                snap: &snap,
                recent_themes: &recent_themes,
                call_serial,
                attempt,
                instant_nanos,
            });

            let raw = match llm_complete(
                gw,
                ctx.skills_dir,
                brief.system_prompt(),
                &brief.user_prompt(),
            )
            .await
            {
                Ok(r) => r,
                Err(_) => continue,
            };
            let cleaned = clean_llm_output(&raw);

            if accept_creative(&cleaned, 512, quality_gate_word) && persona_gate(&cleaned) {
                if let Some(candidate) = extract_word_candidate(&cleaned) {
                    if fingerprint_too_similar(&candidate, &recent_words) {
                        recent_themes.push(format!(
                            "the invented word '{candidate}' or similar spellings"
                        ));
                        continue;
                    }
                }

                let h = body_hash(&cleaned);
                if recent_hashes.contains(&h) {
                    let words: Vec<&str> = cleaned.split_whitespace().take(3).collect();
                    recent_themes.push(format!(
                        "the word/phrasing starting with '{}'",
                        words.join(" ")
                    ));
                    continue;
                }

                recent_hashes.push(h);
                if recent_hashes.len() > 20 {
                    recent_hashes.remove(0);
                }
                let _ = save_recent_hashes(&ledger_path, &recent_hashes);
                if let Some(candidate) = extract_word_candidate(&cleaned) {
                    record_fingerprint(
                        &mut recent_words,
                        &words_path,
                        normalize_fingerprint(&candidate),
                        30,
                    );
                }
                word_data = Some((cleaned, brief));
                break;
            }
        }

        let (result_text, brief) = word_data.ok_or_else(|| {
            anyhow::anyhow!("LLM offline, word failed quality gate, or repeated too many times")
        })?;

        let word = line_value(&result_text, "WORD:")
            .unwrap_or("unknown")
            .split('(')
            .next()
            .unwrap_or("unknown")
            .trim()
            .to_string();
        let definition = line_value(&result_text, "DEFINITION:")
            .unwrap_or("")
            .to_string();

        let event = ChaosEvent::WordGenerated {
            word: word.clone(),
            definition: definition.clone(),
        };
        let _ = ctx.feedback_tx.send(event.clone()).await;

        let display = format_attractor_display(
            "🔤 ATTRACTOR WORD",
            &brief.meta,
            "theme",
            &result_text,
            45,
            "friction -0.02",
        );

        Ok(SkillOutput {
            display,
            feedback: vec![event],
            inject_to_conversation: true,
            evidence: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::extract_word_candidate;
    use crate::skills::attractor_common::{body_hash, normalize_fingerprint};
    use crate::skills::generative::quality_gate_word;

    #[test]
    fn test_body_hash_stable() {
        let h1 = body_hash("WORD: flibber");
        let h2 = body_hash("WORD: flibber");
        assert_eq!(h1, h2);
    }

    #[test]
    fn test_word_quality_gate() {
        assert!(!quality_gate_word("This is lorem ipsum dolor sit amet."));
        assert!(quality_gate_word("WORD: glim\nDEFINITION: A tiny sparkle."));
    }

    #[test]
    fn test_extract_word_candidate() {
        assert_eq!(
            extract_word_candidate("WORD: KRAKTIC (KRAK-tik)\nDEFINITION: x").as_deref(),
            Some("KRAKTIC")
        );
    }

    #[test]
    fn test_normalize_invented_word() {
        assert_eq!(
            normalize_fingerprint("KRAKTIC (KRAK-tik)"),
            "kraktickraktik"
        );
        assert_eq!(normalize_fingerprint("KRAKTIC"), "kraktic");
    }
}
