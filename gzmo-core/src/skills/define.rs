use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Result;
use async_trait::async_trait;

use gzmo_chaos::feedback::ChaosEvent;

use super::attractor_common::{
    body_hash, format_attractor_display, load_recent_hashes, next_call_serial, save_recent_hashes,
};
use super::define_brief::{DefineBrief, DefineBriefInput};
use super::dispatch::{data_dir_from_skills, load_live_chaos_snapshot};
use super::generative::{
    accept_creative, clean_llm_output, line_value, llm_complete, persona_constraint_gate,
    quality_gate_define, require_gateway,
};
use super::{Skill, SkillContext, SkillOutput, SkillType};

pub struct DefineSkill;

#[async_trait]
impl Skill for DefineSkill {
    fn name(&self) -> &str {
        "define"
    }
    fn description(&self) -> &str {
        "Provide definition, pronunciation (IPA), and etymology"
    }
    fn skill_type(&self) -> SkillType {
        SkillType::Generative
    }

    async fn execute(&self, ctx: SkillContext<'_>) -> Result<SkillOutput> {
        let term = ctx.args.trim();
        if term.is_empty() {
            return Ok(SkillOutput {
                display: "✗ Usage: /define <term>".to_string(),
                feedback: vec![],
                inject_to_conversation: false,
            evidence: None,
            });
        }

        let data_dir = data_dir_from_skills(ctx.skills_dir);
        let skills_data = data_dir.join("skills");
        let ledger_path = skills_data.join(".define_recent_hashes");
        let serial_path = skills_data.join(".define_call_serial");

        let call_serial = next_call_serial(&serial_path)?;
        let mut recent_hashes = load_recent_hashes(&ledger_path);
        let mut define_data = None;
        let mut recent_themes = Vec::new();
        let persona_gate = persona_constraint_gate(ctx.skills_dir);

        if let Ok(gw) = require_gateway(&ctx) {
            for attempt in 1..=3 {
                let snap = load_live_chaos_snapshot(&data_dir, ctx.chaos);
                gw.set_chaos_overrides(snap.llm_temperature, snap.llm_max_tokens);

                let instant_nanos = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map(|d| d.as_nanos() as u64)
                    .unwrap_or(0);

                let brief = DefineBrief::new(DefineBriefInput {
                    term,
                    snap: &snap,
                    recent_themes: &recent_themes,
                    call_serial,
                    attempt,
                    instant_nanos,
                });

                let raw = match llm_complete(gw, ctx.skills_dir, brief.system_prompt(), &brief.user_prompt()).await {
                    Ok(r) => r,
                    Err(_) => continue,
                };
                let cleaned = clean_llm_output(&raw);

                if accept_creative(&cleaned, 800, quality_gate_define) && persona_gate(&cleaned) {
                    let h = body_hash(&cleaned);
                    if recent_hashes.contains(&h) {
                        let words: Vec<&str> = cleaned.split_whitespace().take(3).collect();
                        recent_themes.push(format!(
                            "the definition starting with '{}'",
                            words.join(" ")
                        ));
                        continue;
                    }

                    recent_hashes.push(h);
                    if recent_hashes.len() > 20 {
                        recent_hashes.remove(0);
                    }
                    let _ = save_recent_hashes(&ledger_path, &recent_hashes);
                    define_data = Some((cleaned, brief));
                    break;
                }
            }
        }

        let (result_text, brief) = match define_data {
            Some(data) => data,
            None => {
                let fallback_text = fetch_dictionary_fallback(term).await?;
                let snap = load_live_chaos_snapshot(&data_dir, ctx.chaos);
                let instant_nanos = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map(|d| d.as_nanos() as u64)
                    .unwrap_or(0);
                let brief = DefineBrief::new(DefineBriefInput {
                    term,
                    snap: &snap,
                    recent_themes: &[],
                    call_serial,
                    attempt: 1,
                    instant_nanos,
                });
                (fallback_text, brief)
            }
        };

        let word = line_value(&result_text, "WORD:")
            .unwrap_or(term)
            .trim()
            .to_string();
        let definition = line_value(&result_text, "DEFINITION:")
            .unwrap_or("")
            .trim()
            .to_string();

        let event = ChaosEvent::WordGenerated {
            word,
            definition,
        };
        let _ = ctx.feedback_tx.send(event.clone()).await;

        let display = format_attractor_display(
            "📚 ATTRACTOR DEFINE",
            &brief.meta,
            "term",
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

async fn fetch_dictionary_fallback(term: &str) -> Result<String> {
    let encoded = term.replace(' ', "%20");
    let url = format!("https://api.dictionaryapi.dev/api/v2/entries/en/{encoded}");
    let resp = reqwest::get(&url).await?;
    if !resp.status().is_success() {
        anyhow::bail!("Term not found and LLM offline");
    }
    let json: serde_json::Value = resp.json().await?;
    let word = json[0]["word"].as_str().unwrap_or(term);
    let phonetic = json[0]["phonetic"].as_str().unwrap_or("N/A");
    let meaning = json[0]["meanings"][0]["definitions"][0]["definition"]
        .as_str()
        .unwrap_or("N/A");
    let pos = json[0]["meanings"][0]["partOfSpeech"]
        .as_str()
        .unwrap_or("N/A");
    Ok(format!(
        "WORD: {word}\nPRONUNCIATION: {phonetic}\nPART OF SPEECH: {pos}\nDEFINITION: {meaning}\nETYMOLOGY: (API fallback)\nUSAGE: (API fallback)"
    ))
}

#[cfg(test)]
mod tests {
    use crate::skills::attractor_common::body_hash;
    use crate::skills::generative::quality_gate_define;

    #[test]
    fn test_body_hash_stable() {
        let h1 = body_hash("WORD: flibber\nDEFINITION: A flibbering thing.");
        let h2 = body_hash("WORD: flibber\nDEFINITION: A flibbering thing.");
        assert_eq!(h1, h2);
    }

    #[test]
    fn test_define_quality_gate() {
        assert!(!quality_gate_define("This is lorem ipsum dolor sit amet."));
        assert!(quality_gate_define("WORD: glim\nDEFINITION: A tiny sparkle."));
    }
}
