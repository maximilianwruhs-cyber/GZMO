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
    accept_creative, clean_llm_output, llm_complete, persona_constraint_gate,
    quality_gate_joke, require_gateway,
};
use super::joke_brief::{JokeBrief, JokeBriefInput};
use super::{Skill, SkillContext, SkillOutput, SkillType};

pub struct JokeSkill;

#[async_trait]
impl Skill for JokeSkill {
    fn name(&self) -> &str {
        "joke"
    }
    fn description(&self) -> &str {
        "Tell chaos-coupled Attractor Comedy (BVT, live Lorenz coordinates)"
    }
    fn skill_type(&self) -> SkillType {
        SkillType::Generative
    }

    async fn execute(&self, ctx: SkillContext<'_>) -> Result<SkillOutput> {
        let gw = require_gateway(&ctx)?;
        let data_dir = data_dir_from_skills(ctx.skills_dir);
        let skills_data = data_dir.join("skills");
        let ledger_path = skills_data.join(".joke_recent_hashes");
        let openings_path = skills_data.join(".joke_recent_openings");
        let serial_path = skills_data.join(".joke_call_serial");

        let call_serial = next_call_serial(&serial_path)?;
        let mut recent_hashes = load_recent_hashes(&ledger_path);
        let mut recent_openings = load_recent_hashes(&openings_path);
        let mut joke = None;
        let mut recent_themes = themes_from_fingerprints(&recent_openings, "joke openings");
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

            let brief = JokeBrief::new(JokeBriefInput {
                seed: &seed,
                snap: &snap,
                recent_themes: &recent_themes,
                call_serial,
                attempt,
                instant_nanos,
            });

            let raw = match llm_complete(gw, ctx.skills_dir, brief.system_prompt(), &brief.user_prompt()).await
            {
                Ok(r) => r,
                Err(_) => continue,
            };
            let cleaned = clean_llm_output(&raw);

            if accept_creative(&cleaned, 280, quality_gate_joke) && persona_gate(&cleaned) {
                let opening = opening_fingerprint(&cleaned);
                if fingerprint_too_similar(&opening, &recent_openings) {
                    recent_themes.push(format!(
                        "joke openings like '{}'",
                        cleaned.lines().next().unwrap_or(&cleaned).chars().take(40).collect::<String>()
                    ));
                    continue;
                }

                let h = body_hash(&cleaned);
                if recent_hashes.contains(&h) {
                    let words: Vec<&str> = cleaned.split_whitespace().take(3).collect();
                    recent_themes.push(format!("openings like '{}'", words.join(" ")));
                    continue;
                }
                recent_hashes.push(h);
                if recent_hashes.len() > 20 {
                    recent_hashes.remove(0);
                }
                let _ = save_recent_hashes(&ledger_path, &recent_hashes);
                record_fingerprint(&mut recent_openings, &openings_path, opening, 30);
                joke = Some((cleaned, brief));
                break;
            }
        }

        let (joke_text, brief) = joke.ok_or_else(|| {
            anyhow::anyhow!("LLM offline, joke exceeded quality limits, or repeated too many times")
        })?;

        let event = ChaosEvent::JokeGenerated {
            text: joke_text.clone(),
        };
        let _ = ctx.feedback_tx.send(event.clone()).await;

        let display = format_attractor_display(
            "😂 ATTRACTOR COMEDY",
            &brief.meta,
            "topic",
            &joke_text,
            15,
            "−0.2",
        );

        Ok(SkillOutput {
            display,
            feedback: vec![event],
            inject_to_conversation: true,
            evidence: None,
        })
    }
}
