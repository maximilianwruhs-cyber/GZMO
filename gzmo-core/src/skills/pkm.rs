use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Result;
use async_trait::async_trait;

use gzmo_chaos::feedback::{ChaosEvent, ThoughtSeed};

use super::attractor_common::{
    body_hash, fingerprint_too_similar, load_recent_hashes, next_call_serial,
    normalize_fingerprint, record_fingerprint, save_recent_hashes, themes_from_fingerprints,
    AttractorMeta, AttractorPromptInput,
};
use super::pkm_corpus::{append_forge, ForgeCorpusEntry};
use super::pkm_forge::{
    build_pkm_evidence, build_selection, build_system_prompt, build_user_prompt, is_ultra_or_secret,
    load_pkmforge, parse_pkm, render_forge_display, resolve_pkm_category, validate_forged_pokemon,
    ChaosEventRef,
};
use super::dispatch::{data_dir_from_skills, load_live_chaos_snapshot};
use super::generative::{
    accept_creative, clean_llm_output, line_value, llm_complete, persona_constraint_gate,
    require_gateway,
};
use super::{Skill, SkillContext, SkillOutput, SkillType};

pub struct PkmSkill;

fn pkm_quality_gate(text: &str, category: &str) -> bool {
    validate_forged_pokemon(text, category)
}

#[async_trait]
impl Skill for PkmSkill {
    fn name(&self) -> &str {
        "pkm"
    }
    fn description(&self) -> &str {
        "Legendary Attractor Forge — Pokemon cards via Element, phase lens, forge sparks, ASCII frame + corpus"
    }
    fn skill_type(&self) -> SkillType {
        SkillType::Generative
    }

    async fn execute(&self, ctx: SkillContext<'_>) -> Result<SkillOutput> {
        let data_dir = data_dir_from_skills(ctx.skills_dir);
        let snap_seed = load_live_chaos_snapshot(&data_dir, ctx.chaos);

        let pkmforge = load_pkmforge(ctx.skills_dir).ok_or_else(|| {
            anyhow::anyhow!("pkmforge.toml missing under skills/ — Pokemon Forge cannot run")
        })?;

        let category = match resolve_pkm_category(&pkmforge, ctx.args, &snap_seed) {
            Ok(c) => c,
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
        let ledger_path = skills_data.join(".pkm_recent_hashes");
        let names_path = skills_data.join(".pkm_recent_names");
        let serial_path = skills_data.join(".pkm_call_serial");

        let call_serial = next_call_serial(&serial_path)?;
        let mut recent_hashes = load_recent_hashes(&ledger_path);
        let mut recent_names = load_recent_hashes(&names_path);
        let mut forged = None;
        let mut recent_name_hints = themes_from_fingerprints(&recent_names, "the card named");
        let persona_gate = persona_constraint_gate(ctx.skills_dir);

        for attempt in 1..=3 {
            let snap = load_live_chaos_snapshot(&data_dir, ctx.chaos);
            gw.set_chaos_overrides(snap.llm_temperature, snap.llm_max_tokens);

            let sel = build_selection(&pkmforge, &snap, &category, call_serial);
            let instant_nanos = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_nanos() as u64)
                .unwrap_or(0);

            let attractor = AttractorMeta::from_input(AttractorPromptInput {
                seed_label: "forge",
                seed: &format!(
                    "{}-{}-{}-{}",
                    sel.element, sel.rarity, sel.category, sel.forge_mode.label()
                ),
                snap: &snap,
                recent_themes: &recent_name_hints,
                call_serial,
                attempt,
                instant_nanos,
                max_chars: 900,
                extra_rules: &[],
            });

            let system = build_system_prompt(&pkmforge, &sel);
            let user = build_user_prompt(&attractor, &sel);

            let raw = match llm_complete(gw, ctx.skills_dir, &system, &user).await {
                Ok(r) => r,
                Err(_) => continue,
            };
            let cleaned = clean_llm_output(&raw);

            if {
                let ok_gate = pkm_quality_gate(&cleaned, &category);
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
                let parsed = parse_pkm(&cleaned, &category);
                forged = Some((parsed, sel, attractor, h));
                break;
            }
        }

        let (parsed, sel, attractor, hash) = forged.ok_or_else(|| {
            anyhow::anyhow!("LLM offline, card failed quality gate, or duplicate forge")
        })?;

        let event = ChaosEvent::PkmForged {
            name: parsed.name.clone(),
            element: parsed.element.clone(),
        };
        let mut feedback = vec![event.clone()];
        if is_ultra_or_secret(&sel) {
            feedback.push(ChaosEvent::Custom {
                tension_delta: 3.0,
                energy_delta: -4.0,
                thought_seed: Some(ThoughtSeed {
                    category: "pkm_ex".to_string(),
                    text: format!(
                        "ULTRA EX FORGE: {} — the attractor shivers",
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
                category: parsed.category.clone(),
                element: parsed.element.clone(),
                rarity: sel.rarity.clone(),
                forge_mode: sel.forge_mode.label().to_string(),
                keyword_spark: sel.sparks.keyword.clone(),
                name_seed: sel.sparks.name_seed.clone(),
                set_code: sel.set_code.clone(),
                body_hash: hash.clone(),
            },
        );

        let feedback_refs: Vec<ChaosEventRef<'_>> = feedback
            .iter()
            .map(|ev| match ev {
                ChaosEvent::PkmForged { name, element } => ChaosEventRef {
                    kind: "PkmForged",
                    detail: format!("{name} ({element})"),
                },
                ChaosEvent::Custom { thought_seed, .. } => ChaosEventRef {
                    kind: "Custom",
                    detail: thought_seed
                        .as_ref()
                        .map(|t| t.text.clone())
                        .unwrap_or_else(|| "ex resonance".into()),
                },
                other => ChaosEventRef {
                    kind: "Other",
                    detail: format!("{other:?}"),
                },
            })
            .collect();

        let evidence = build_pkm_evidence(
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
    use super::pkm_quality_gate;

    #[test]
    fn pkm_quality_gate_rejects_placeholder() {
        assert!(!pkm_quality_gate("NAME: [card name]\nELEMENT: fire", "Pokemon"));
        assert!(pkm_quality_gate(
            "NAME: Pyroclaw\nCATEGORY: Pokemon\nELEMENT: fire\nHP: 120\nSTAGE: Basic\nRARITY: Rare\nATTACK1: Fire Claw | {R}{C} | 40 | desc",
            "Pokemon"
        ));
    }
}
