//! # Define Skill — `/define [term]`
//!
//! Definition, pronunciation (IPA), and etymology via LLM with dictionary API fallback.

use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;

use super::llm::{frame_box, llm_chat, SkillRuntime, BOLD, BLUE, CYAN, MAGENTA, RED, RESET, WHITE};
use super::{Skill, SkillContext, SkillOutput, SkillType};

const SYSTEM_PROMPT: &str = "You are a lexicographer. For the given term, provide:
1. WORD: The term
2. PRONUNCIATION: IPA notation
3. PART OF SPEECH: (noun, verb, adjective, etc.)
4. DEFINITION: Clear, precise definition
5. ETYMOLOGY: Language of origin and historical derivation
6. USAGE: One example sentence

Format each on its own line with the label prefix. No other text.";

pub struct DefineSkill {
    pub rt: Arc<SkillRuntime>,
}

fn format_define_lines(result: &str) -> String {
    let mut out = String::new();
    for line in result.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("WORD:") {
            out.push_str(&format!("  {BOLD}{CYAN}WORD:{rest}{RESET}\n"));
        } else if let Some(rest) = trimmed.strip_prefix("PRONUNCIATION:") {
            out.push_str(&format!("  {DIM}PRONUNCIATION:{rest}{RESET}\n", DIM = super::llm::DIM));
        } else if let Some(rest) = trimmed.strip_prefix("DEFINITION:") {
            out.push_str(&format!("  {WHITE}DEFINITION:{rest}{RESET}\n"));
        } else if let Some(rest) = trimmed.strip_prefix("ETYMOLOGY:") {
            out.push_str(&format!("  {MAGENTA}ETYMOLOGY:{rest}{RESET}\n"));
        } else {
            out.push_str(&format!("  {trimmed}\n"));
        }
    }
    out
}

async fn dictionary_fallback(term: &str) -> Option<String> {
    let encoded: String = term
        .chars()
        .map(|c| match c {
            'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => c.to_string(),
            ' ' => "+".to_string(),
            _ => format!("%{:02X}", c as u32),
        })
        .collect();
    let url = format!("https://api.dictionaryapi.dev/api/v2/entries/en/{encoded}");
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .ok()?;
    let body: serde_json::Value = client.get(url).send().await.ok()?.json().await.ok()?;
    let entry = body.as_array()?.first()?;
    let word = entry["word"].as_str()?;
    let phonetic = entry["phonetic"].as_str().unwrap_or("N/A");
    let meaning = entry["meanings"][0]["definitions"][0]["definition"]
        .as_str()
        .unwrap_or("N/A");
    let pos = entry["meanings"][0]["partOfSpeech"]
        .as_str()
        .unwrap_or("N/A");
    Some(format!(
        "WORD: {word}\nPRONUNCIATION: {phonetic}\nPART OF SPEECH: {pos}\nDEFINITION: {meaning}\nETYMOLOGY: (API fallback — etymology unavailable)\nUSAGE: (API fallback — example unavailable)"
    ))
}

#[async_trait]
impl Skill for DefineSkill {
    fn name(&self) -> &str {
        "define"
    }
    fn description(&self) -> &str {
        "Lexicographer definition with IPA and etymology"
    }
    fn skill_type(&self) -> SkillType {
        SkillType::Generative
    }

    async fn execute(&self, ctx: SkillContext<'_>) -> Result<SkillOutput> {
        let term = ctx.args.trim();
        if term.is_empty() {
            return Ok(SkillOutput {
                display: format!("  {RED}✗ Usage: /define <term>{RESET}"),
                feedback: vec![],
                inject_to_conversation: false,
            });
        }

        let user_prompt = format!("Define: {term}");
        let mut result = llm_chat(&self.rt, SYSTEM_PROMPT, &user_prompt, 0.3, 384, false)
            .await
            .unwrap_or_default();

        let mut used_fallback = false;
        if result.is_empty() {
            if let Some(api) = dictionary_fallback(term).await {
                result = api;
                used_fallback = true;
            } else {
                return Ok(SkillOutput {
                    display: format!("  {RED}✗ Term not found and LLM offline.{RESET}"),
                    feedback: vec![],
                    inject_to_conversation: false,
                });
            }
        }

        let mut body = String::new();
        if used_fallback {
            body.push_str(&format!(
                "  {DIM}LLM offline — trying dictionary API...{RESET}\n\n",
                DIM = super::llm::DIM
            ));
        }
        body.push_str(&format_define_lines(&result));

        let display = frame_box("DEFINE", &body, "📚", BLUE);

        Ok(SkillOutput {
            display,
            feedback: vec![],
            inject_to_conversation: true,
        })
    }
}
