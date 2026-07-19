use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Result;
use async_trait::async_trait;

use gzmo_chaos::feedback::ChaosEvent;

use super::attractor_common::{
    body_hash, fingerprint_too_similar, format_attractor_display, load_recent_hashes,
    next_call_serial, opening_fingerprint, record_fingerprint, resolve_chaos_seed,
    save_recent_hashes, themes_from_fingerprints,
};
use super::dispatch::{data_dir_from_skills, load_live_chaos_snapshot};
use super::generative::{
    accept_creative, clean_llm_output, llm_complete, persona_constraint_gate, quality_gate_story,
    require_gateway,
};
use super::story_brief::{StoryBrief, StoryBriefInput};
use super::{Skill, SkillContext, SkillOutput, SkillType};

pub struct StorySkill;

#[async_trait]
impl Skill for StorySkill {
    fn name(&self) -> &str {
        "story"
    }
    fn description(&self) -> &str {
        "Generate chaos-coupled Attractor Fiction from a keyword (live Lorenz coordinates)"
    }
    fn skill_type(&self) -> SkillType {
        SkillType::Generative
    }

    async fn execute(&self, ctx: SkillContext<'_>) -> Result<SkillOutput> {
        let gw = require_gateway(&ctx)?;
        let data_dir = data_dir_from_skills(ctx.skills_dir);
        let skills_data = data_dir.join("skills");
        let ledger_path = skills_data.join(".story_recent_hashes");
        let openings_path = skills_data.join(".story_recent_openings");
        let serial_path = skills_data.join(".story_call_serial");

        let call_serial = next_call_serial(&serial_path)?;
        let mut recent_hashes = load_recent_hashes(&ledger_path);
        let mut recent_openings = load_recent_hashes(&openings_path);
        let mut story = None;
        let mut recent_themes = themes_from_fingerprints(&recent_openings, "story openings");
        let persona_gate = persona_constraint_gate(ctx.skills_dir);
        let arg_seed = ctx.args.trim().to_string();

        for attempt in 1..=3 {
            let snap = load_live_chaos_snapshot(&data_dir, ctx.chaos);
            gw.set_chaos_overrides(snap.llm_temperature, snap.llm_max_tokens);

            let keyword = resolve_chaos_seed(&arg_seed, &snap, call_serial);

            let instant_nanos = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_nanos() as u64)
                .unwrap_or(0);

            let brief = StoryBrief::new(StoryBriefInput {
                keyword: &keyword,
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

            if accept_creative(&cleaned, 1000, quality_gate_story) && persona_gate(&cleaned) {
                let opening = opening_fingerprint(&cleaned);
                if fingerprint_too_similar(&opening, &recent_openings) {
                    recent_themes.push(format!(
                        "story openings like '{}'",
                        cleaned
                            .lines()
                            .next()
                            .unwrap_or(&cleaned)
                            .chars()
                            .take(40)
                            .collect::<String>()
                    ));
                    continue;
                }

                let h = body_hash(&cleaned);
                if recent_hashes.contains(&h) {
                    let words: Vec<&str> = cleaned.split_whitespace().take(3).collect();
                    recent_themes.push(format!("the imagery starting with '{}'", words.join(" ")));
                    continue;
                }

                recent_hashes.push(h);
                if recent_hashes.len() > 20 {
                    recent_hashes.remove(0);
                }
                let _ = save_recent_hashes(&ledger_path, &recent_hashes);
                record_fingerprint(&mut recent_openings, &openings_path, opening, 30);
                story = Some((cleaned, brief));
                break;
            }
        }

        let (story_text, brief) = story.ok_or_else(|| {
            anyhow::anyhow!(
                "LLM offline, story exceeded quality limits, or repeated too many times"
            )
        })?;

        let event = ChaosEvent::StoryGenerated {
            text: story_text.clone(),
        };
        let _ = ctx.feedback_tx.send(event.clone()).await;

        let meta = super::attractor_common::AttractorMeta {
            seed: brief.keyword.clone(),
            tick: brief.tick,
            phase: brief.phase,
            valence: brief.valence,
            rho_effective: brief.rho_effective,
            call_serial: brief.call_serial,
            nonce: brief.nonce,
            cabinet_echo: brief.cabinet_echo.clone(),
            anti_repeat_hint: brief.anti_repeat_hint.clone(),
        };

        let display = format_attractor_display(
            "📖 ATTRACTOR FICTION",
            &meta,
            "keyword",
            &story_text,
            40,
            "+0.5",
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
    use crate::skills::attractor_common::body_hash;
    use crate::skills::generative::quality_gate_story;

    #[test]
    fn test_body_hash_stable() {
        let h1 = body_hash("The oil was cold.");
        let h2 = body_hash("The oil was cold.");
        assert_eq!(h1, h2);
    }

    #[test]
    fn test_story_quality_gate() {
        assert!(!quality_gate_story("Once upon a time, there was a king."));
        assert!(quality_gate_story(
            "The oil was cold. The metal did not turn."
        ));
    }
}
