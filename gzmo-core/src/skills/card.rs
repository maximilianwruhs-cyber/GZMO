use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Result;
use async_trait::async_trait;

use gzmo_chaos::feedback::{ChaosEvent, ThoughtSeed};

use super::attractor_common::{
    body_hash, fingerprint_too_similar, load_recent_hashes, next_call_serial,
    normalize_fingerprint, record_fingerprint, save_recent_hashes, themes_from_fingerprints,
    AttractorMeta, AttractorPromptInput,
};
use super::card_corpus::{append_forge, ForgeCorpusEntry};
use super::card_forge::{
    build_card_evidence, build_selection, build_system_prompt, build_user_prompt, is_mythic,
    load_cardforge, parse_card, render_forge_display, resolve_card_type, validate_forged_card,
    ChaosEventRef,
};
use super::dispatch::{data_dir_from_skills, load_live_chaos_snapshot};
use super::generative::{
    accept_creative, clean_llm_output, line_value, llm_complete, persona_constraint_gate,
    require_gateway,
};
use super::{Skill, SkillContext, SkillOutput, SkillType};

pub struct CardSkill;

fn card_quality_gate(text: &str, requires_pt: bool) -> bool {
    validate_forged_card(text, requires_pt)
}

#[async_trait]
impl Skill for CardSkill {
    fn name(&self) -> &str {
        "card"
    }
    fn description(&self) -> &str {
        "Legendary Attractor Forge — MTG cards via Color Pie, phase lens, forge sparks, ASCII frame + corpus"
    }
    fn skill_type(&self) -> SkillType {
        SkillType::Generative
    }

    async fn execute(&self, ctx: SkillContext<'_>) -> Result<SkillOutput> {
        let data_dir = data_dir_from_skills(ctx.skills_dir);
        let snap_seed = load_live_chaos_snapshot(&data_dir, ctx.chaos);

        let forge_path = ctx.skills_dir.join("cardforge.toml");
        let cardforge = load_cardforge(ctx.skills_dir).ok_or_else(|| {
            if forge_path.is_file() {
                anyhow::anyhow!(
                    "cardforge.toml at {} exists but failed to parse — restore from git or fix TOML",
                    forge_path.display()
                )
            } else {
                anyhow::anyhow!(
                    "cardforge.toml not found at {} — Card Forge cannot run",
                    forge_path.display()
                )
            }
        })?;

        let card_type = match resolve_card_type(&cardforge, ctx.args, &snap_seed) {
            Ok(t) => t,
            Err(msg) => {
                return Ok(SkillOutput {
                    display: msg,
                    feedback: vec![],
                    inject_to_conversation: false,
                    evidence: None,
                });
            }
        };

        let gw = require_gateway(&ctx)?;
        let skills_data = data_dir.join("skills");
        let ledger_path = skills_data.join(".card_recent_hashes");
        let names_path = skills_data.join(".card_recent_names");
        let serial_path = skills_data.join(".card_call_serial");

        let call_serial = next_call_serial(&serial_path)?;
        let mut recent_hashes = load_recent_hashes(&ledger_path);
        let mut recent_names = load_recent_hashes(&names_path);
        let mut forged = None;
        let mut recent_name_hints = themes_from_fingerprints(&recent_names, "the card named");
        let persona_gate = persona_constraint_gate(ctx.skills_dir);

        for attempt in 1..=3 {
            let snap = load_live_chaos_snapshot(&data_dir, ctx.chaos);
            gw.set_chaos_overrides(snap.llm_temperature, snap.llm_max_tokens);

            let sel = build_selection(&cardforge, &snap, &card_type, call_serial);
            let instant_nanos = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_nanos() as u64)
                .unwrap_or(0);

            let attractor = AttractorMeta::from_input(AttractorPromptInput {
                seed_label: "forge",
                seed: &format!(
                    "{}-{}-{}-{}",
                    sel.color, sel.rarity, sel.card_type, sel.forge_mode.label()
                ),
                snap: &snap,
                recent_themes: &recent_name_hints,
                call_serial,
                attempt,
                instant_nanos,
                max_chars: 900,
                extra_rules: &[],
            });

            let system = build_system_prompt(&cardforge, &sel);
            let user = build_user_prompt(&attractor, &sel);

            let raw = match llm_complete(gw, ctx.skills_dir, &system, &user).await {
                Ok(r) => r,
                Err(_) => continue,
            };
            let cleaned = clean_llm_output(&raw);

            if {
                let ok_gate = card_quality_gate(&cleaned, sel.requires_pt);
                ok_gate && persona_gate(&cleaned) && {
                    let count = cleaned.chars().count();
                    count > 0 && count <= 900
                }
            } {
                if let Some(name) = line_value(&cleaned, "NAME:") {
                    if fingerprint_too_similar(name, &recent_names) {
                        recent_name_hints
                            .push(format!("the card named '{name}' or similar titles"));
                        continue;
                    }
                }

                let h = body_hash(&cleaned);
                if recent_hashes.contains(&h) {
                    if let Some(name) = line_value(&cleaned, "NAME:") {
                        recent_name_hints.push(format!("the card named '{name}'"));
                    }
                    continue;
                }
                recent_hashes.push(h.clone());
                if recent_hashes.len() > 20 {
                    recent_hashes.remove(0);
                }
                let _ = save_recent_hashes(&ledger_path, &recent_hashes);
                if let Some(name) = line_value(&cleaned, "NAME:") {
                    record_fingerprint(
                        &mut recent_names,
                        &names_path,
                        normalize_fingerprint(name),
                        30,
                    );
                }
                let parsed = parse_card(&cleaned, &card_type);
                forged = Some((parsed, sel, attractor, h));
                break;
            }
        }

        let (parsed, sel, attractor, hash) = forged.ok_or_else(|| {
            anyhow::anyhow!("LLM offline, card failed quality gate, or duplicate forge")
        })?;

        let event = ChaosEvent::CardForged {
            name: parsed.name.clone(),
            card_type: parsed.type_line.clone(),
        };
        let mut feedback = vec![event.clone()];
        if is_mythic(&sel) {
            feedback.push(ChaosEvent::Custom {
                tension_delta: 3.0,
                energy_delta: -4.0,
                thought_seed: Some(ThoughtSeed {
                    category: "card_mythic".to_string(),
                    text: format!(
                        "MYTHIC FORGE: {} — the attractor shivers",
                        parsed.name
                    ),
                }),
            });
        }
        for ev in &feedback {
            let _ = ctx.feedback_tx.send(ev.clone()).await;
        }

        let display = render_forge_display(&attractor, &sel, &parsed);

        let _ = append_forge(
            &data_dir,
            &ForgeCorpusEntry {
                inv: attractor.call_serial,
                tick: attractor.tick,
                name: parsed.name.clone(),
                cost: parsed.cost.clone(),
                type_line: parsed.type_line.clone(),
                rarity: sel.rarity.clone(),
                color: sel.color.to_string(),
                card_type: sel.card_type.clone(),
                forge_mode: sel.forge_mode.label().to_string(),
                keyword_spark: sel.sparks.keyword.clone(),
                subtype_hint: sel.sparks.subtype.clone(),
                name_seed: sel.sparks.name_seed.clone(),
                set_code: sel.set_code.clone(),
                body_hash: hash.clone(),
            },
        );

        let feedback_refs: Vec<ChaosEventRef<'_>> = feedback
            .iter()
            .map(|ev| match ev {
                ChaosEvent::CardForged { name, card_type } => ChaosEventRef {
                    kind: "CardForged",
                    detail: format!("{name} ({card_type})"),
                },
                ChaosEvent::Custom { thought_seed, .. } => ChaosEventRef {
                    kind: "Custom",
                    detail: thought_seed
                        .as_ref()
                        .map(|t| t.text.clone())
                        .unwrap_or_else(|| "mythic resonance".into()),
                },
                other => ChaosEventRef {
                    kind: "Other",
                    detail: format!("{other:?}"),
                },
            })
            .collect();

        let evidence = build_card_evidence(
            &parsed,
            &sel,
            &attractor,
            &display,
            &hash,
            &feedback_refs,
        );

        Ok(SkillOutput {
            display,
            feedback,
            inject_to_conversation: true,
            evidence: Some(evidence),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::card_quality_gate;

    #[test]
    fn card_quality_gate_rejects_placeholder() {
        assert!(!card_quality_gate("NAME: [card name]\nCOST: {1}", false));
        assert!(card_quality_gate(
            "NAME: Iron Golem\nCOST: {4}\nTYPE: Creature — Golem\nRARITY: Rare\nRULES: Trample.\nFLAVOR: Heavy.\nPT: 5/5",
            true
        ));
    }
}
