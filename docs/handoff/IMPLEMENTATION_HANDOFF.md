# GZMO Autonomous Loop — Implementation Handoff Guide

**Version:** 1.0  
**Date:** 2026-07-09  
**Target Model:** Ornith-35B (Q4_K_M quantization)  
**Priority:** Critical — autonomous loop quality is degraded

---

## Executive Summary

The autonomous loop (orchestrator, dream engine, spark engine) produces poor quality output due to a combination of:
1. **Over-constrained prompts** that assume stronger model capabilities
2. **Rigid validation gates** that fail on valid but imperfect model output
3. **Missing few-shot examples** in complex extraction tasks
4. **Temperature settings** too high for structured tasks

This guide provides a complete implementation plan to fix these issues.

---

## Architecture Overview

### Core Components

```
┌─────────────────────────────────────────────────────────────┐
│                    Orchestrator (cron jobs)                  │
│  - Simple mode: single prompt → agent loop                  │
│  - Pipeline mode: multi-step with dependency waves          │
└─────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│                   Agent Loop (agent_loop.rs)                 │
│  - Prompt → LLM → Tool Dispatch → Result Injection → LLM   │
│  - Max iterations: 10 (configurable)                        │
│  - Context window management with archive/distill           │
└─────────────────────────────────────────────────────────────┘
                            │
            ┌───────────────┼───────────────┐
            ▼               ▼               ▼
┌─────────────────┐ ┌─────────────┐ ┌──────────────┐
│  Dream Engine   │ │ Spark Engine│ │  Ingest      │
│  (nightly)      │ │ (serendipity)│ │  (file watch)│
└─────────────────┘ └─────────────┘ └──────────────┘
```

### Key Files

| File | Purpose | Lines |
|------|---------|-------|
| `gzmo-core/src/agent_loop.rs` | Core agentic loop | ~300 |
| `gzmo-core/src/dreams.rs` | Nightly consolidation | ~500 |
| `gzmo-core/src/spark.rs` | Serendipitous recall | ~600 |
| `gzmo-core/src/orchestrator.rs` | Cron job execution | ~700 |
| `gzmo-core/src/gateway.rs` | LLM gateway interface | ~200 |
| `gzmo-core/src/memory/` | Memory subsystems | ~1500 |

---

## Issue Analysis

### Issue 1: Over-Constrained JSON Schemas

**Location:** `gzmo-core/src/spark.rs` (lines ~580-620)

**Problem:**
```rust
fn hypothesis_schema() -> serde_json::Value {
    serde_json::json!({
        "type": "object",
        "properties": {
            "internal_analysis": { "type": "string", "maxLength": 600 },
            "anchor_label": { "type": "string" },
            "recent_label": { "type": "string" },
            "connection": { "type": "string" },
            "what_to_remember": {
                "type": "array",
                "items": { "type": "string" }
            }
        },
        "required": ["internal_analysis", "anchor_label", "recent_label", "connection", "what_to_remember"],
        "additionalProperties": false
    })
}
```

**Why it fails:**
- 35B model struggles with strict JSON formatting
- `additionalProperties: false` causes rejection on extra fields
- `maxLength: 600` is arbitrary and may truncate reasoning
- All fields required — model may skip optional ones

**Fix:**
```rust
fn hypothesis_schema() -> serde_json::Value {
    serde_json::json!({
        "type": "object",
        "properties": {
            "internal_analysis": { 
                "type": "string", 
                "description": "Your reasoning process (200-400 words)"
            },
            "anchor_label": { 
                "type": "string",
                "description": "Short name for the anchor fact"
            },
            "recent_label": { 
                "type": "string",
                "description": "Short name for the recent context"
            },
            "connection": { 
                "type": "string",
                "description": "3-5 sentence hypothesis connecting anchor and recent"
            },
            "what_to_remember": {
                "type": "array",
                "items": { "type": "string" },
                "description": "1-2 bullet points of durable insight"
            }
        },
        "required": ["connection"],
        "additionalProperties": true
    })
}
```

**Changes:**
1. Remove `maxLength` constraint
2. Make most fields optional (only `connection` required)
3. Add `description` to each field
4. Set `additionalProperties: true`

---

### Issue 2: Missing Few-Shot Examples

**Location:** `gzmo-core/src/dreams.rs` (line ~35)

**Problem:**
```rust
const DREAM_EXTRACT_SYSTEM: &str = concat!(
    "You are a memory consolidation engine. Extract structured knowledge from a daily log.\n\n",
    "Rules:\n",
    "1. Use internal_analysis to reason first.\n",
    "2. Extract PEOPLE, SYSTEMS, PROJECTS, TOOLS, DECISIONS as entities — not generic section labels.\n",
    "3. Each entity needs 1+ concrete observations from the log.\n",
    "4. Relations: one edge per link; USES, MANAGES, DEPENDS_ON, RELATED_TO, AUTHORED_BY.\n",
    "5. Disambiguate pronouns to real names.\n",
    "6. Empty arrays if the log is trivial."
);
```

**Why it fails:**
- No examples of desired output format
- Complex instructions without demonstration
- Model doesn't know what "good" looks like

**Fix:**
```rust
const DREAM_EXTRACT_SYSTEM: &str = concat!(
    "You are a memory consolidation engine. Extract structured knowledge from a daily log.\n\n",
    "Rules:\n",
    "1. Use internal_analysis to reason first.\n",
    "2. Extract PEOPLE, SYSTEMS, PROJECTS, TOOLS, DECISIONS as entities — not generic section labels.\n",
    "3. Each entity needs 1+ concrete observations from the log.\n",
    "4. Relations: one edge per link; USES, MANAGES, DEPENDS_ON, RELATED_TO, AUTHORED_BY.\n",
    "5. Disambiguate pronouns to real names.\n",
    "6. Empty arrays if the log is trivial.\n\n",
    "Example output:\n",
    "{\n",
    "  \"internal_analysis\": \"The log shows work on GZMO memory system. Key decisions about vault schema and ingest pipeline.\",\n",
    "  \"entities\": [\n",
    "    {\"name\": \"GZMO\", \"type\": \"PROJECT\", \"observations\": [\"local-first strategy\", \"Rust implementation\"]}\n",
    "  ],\n",
    "  \"relations\": [\n",
    "    {\"from\": \"GZMO\", \"to\": \"SQLite\", \"relationType\": \"USES\"}\n",
    "  ]\n",
    "}"
);
```

**Changes:**
1. Add complete JSON example
2. Show entity structure with observations
3. Show relation structure with types
4. Demonstrate internal_analysis format

---

### Issue 3: Rigid Citation Validation

**Location:** `gzmo-core/src/spark.rs` (lines ~400-450)

**Problem:**
```rust
fn citations_valid(
    selection: &SparkSelection,
    verdict: &SparkVerdict,
    min_chars: usize,
) -> bool {
    let a = verdict.evidence_anchor.trim();
    let r = verdict.evidence_recent.trim();
    if a.len() < min_chars || r.len() < min_chars {
        return false;
    }
    anchor_citation_valid(&selection.anchor.content, a, min_chars)
        && recent_citation_valid(&selection.recent, r, min_chars)
}
```

**Why it fails:**
- Model often paraphrases instead of quoting verbatim
- `min_chars` (default 12) may be too strict
- Exact string matching fails on minor formatting differences

**Fix:**
```rust
fn citations_valid(
    selection: &SparkSelection,
    verdict: &SparkVerdict,
    min_chars: usize,
) -> bool {
    let a = verdict.evidence_anchor.trim();
    let r = verdict.evidence_recent.trim();
    
    // Lower minimum for paraphrased content
    let effective_min = if a.len() < 20 || r.len() < 20 {
        min_chars / 2
    } else {
        min_chars
    };
    
    if a.len() < effective_min || r.len() < effective_min {
        return false;
    }
    
    // Use fuzzy matching instead of exact
    anchor_citation_valid_fuzzy(&selection.anchor.content, a, effective_min)
        && recent_citation_valid_fuzzy(&selection.recent, r, effective_min)
}

fn anchor_citation_valid_fuzzy(anchor_content: &str, quote: &str, min_chars: usize) -> bool {
    if quote.len() < min_chars {
        return false;
    }
    
    // Try exact match first
    if source_contains_quote(anchor_content, quote) {
        return true;
    }
    
    // Try normalized match (lowercase, remove punctuation)
    let norm_src = normalize_text(anchor_content);
    let norm_q = normalize_text(quote);
    
    // Check if key terms appear
    let q_words: Vec<&str> = norm_q.split_whitespace().collect();
    let key_terms: Vec<&str> = q_words.iter()
        .filter(|w| w.len() > 4)
        .collect();
    
    if key_terms.is_empty() {
        return false;
    }
    
    let matched = key_terms.iter()
        .filter(|t| norm_src.contains(t))
        .count();
    
    // At least 60% of key terms should match
    matched as f64 / key_terms.len() as f64 >= 0.6
}

fn normalize_text(text: &str) -> String {
    text.to_lowercase()
        .chars()
        .filter(|c| c.is_alphanumeric() || c.is_whitespace())
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}
```

**Changes:**
1. Lower minimum for short quotes
2. Add fuzzy matching with key term extraction
3. Require 60% key term match instead of exact
4. Normalize text before comparison

---

### Issue 4: Temperature Too High for Structured Tasks

**Location:** `gzmo-core/src/spark.rs` (config)

**Problem:**
```rust
// Current settings (assumed)
hypothesis_temperature: 0.7,
verify_temperature: 0.7,
```

**Why it fails:**
- High temperature increases creativity but reduces consistency
- Structured tasks need deterministic output
- JSON formatting becomes unreliable

**Fix:**
```rust
// In config or when calling gateway
hypothesis_temperature: 0.2,  // Lower for structured output
verify_temperature: 0.1,      // Even lower for verification
```

**Changes:**
1. Reduce hypothesis temperature to 0.2
2. Reduce verify temperature to 0.1
3. Keep higher temperature (0.7-0.9) for creative tasks only

---

### Issue 5: No Retry with Feedback

**Location:** `gzmo-core/src/dreams.rs` (line ~130)

**Problem:**
```rust
match self.promoter.run_pipeline(chunk, "dream_extraction", DREAM_EXTRACT_SYSTEM, &label).await {
    Ok(p) => chunk_results.push(p),
    Err(e) => {
        warn!("REM/verify pipeline failed: {e}");
        return Ok(DreamReport { /* error report */ });
    }
}
```

**Why it fails:**
- Single attempt with no recovery
- Model errors are terminal
- No feedback loop to improve output

**Fix:**
```rust
async fn run_extraction_with_retry(
    &self,
    chunk: &str,
    label: &str,
    max_retries: usize,
) -> Result<KgPipeline> {
    let mut last_error: Option<String> = None;
    
    for attempt in 0..=max_retries {
        let feedback = if let Some(ref err) = last_error {
            format!("\n\nPrevious attempt failed: {err}\nPlease fix the formatting and try again.")
        } else {
            String::new()
        };
        
        match self.promoter.run_pipeline(
            chunk, 
            "dream_extraction", 
            DREAM_EXTRACT_SYSTEM,
            &label
        ).await {
            Ok(p) => return Ok(p),
            Err(e) => {
                if attempt < max_retries {
                    warn!("Extraction failed (attempt {}): {e}", attempt + 1);
                    last_error = Some(e.to_string());
                    // Add small delay before retry
                    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                } else {
                    return Err(e);
                }
            }
        }
    }
    
    unreachable!()
}
```

**Changes:**
1. Add retry loop (max 3 attempts)
2. Include error feedback in retry prompt
3. Add delay between retries to avoid rate limits

---

### Issue 6: Missing Self-Correction Phase

**Location:** `gzmo-core/src/dreams.rs` (after extraction)

**Problem:**
Extraction happens in one pass with no validation or correction.

**Fix:**
```rust
// After extraction, add self-correction phase
async fn self_correct_extraction(
    &self,
    entities: &[VerifiedEntity],
    relations: &[VerifiedRelation],
) -> Result<(Vec<VerifiedEntity>, Vec<VerifiedRelation>)> {
    let messages = vec![
        Message {
            role: Role::System,
            content: "You are reviewing extracted knowledge. Check for:\n1. Missing observations\n2. Incorrect entity types\n3. Incomplete relations\n4. Formatting issues\n\nProvide corrections in JSON format.".to_string(),
            is_meta: false,
            tool_calls: None,
            tool_call_id: None,
        },
        Message {
            role: Role::User,
            content: format!(
                "Review and correct this extraction:\n\nEntities: {:?}\nRelations: {:?}",
                entities, relations
            ),
            is_meta: false,
            tool_calls: None,
            tool_call_id: None,
        },
    ];
    
    let raw = self.gateway.complete_structured_bounded(
        &messages,
        "self_correction",
        correction_schema(),
        Some(0.1),
        Some(1000),
    ).await?;
    
    // Parse and apply corrections
    let corrections: CorrectionRequest = parse_json_lenient(&raw)?;
    let corrected_entities = apply_corrections(entities, &corrections.entities);
    let corrected_relations = apply_corrections(relations, &corrections.relations);
    
    Ok((corrected_entities, corrected_relations))
}
```

**Changes:**
1. Add post-extraction review phase
2. Ask model to identify and fix issues
3. Apply corrections to final output

---

## Implementation Plan

### Phase 1: Immediate Fixes (1-2 hours)

**Priority: High**

1. **Update JSON schemas** (`spark.rs`)
   - Remove strict constraints
   - Add descriptions
   - Make fields optional

2. **Add few-shot examples** (`dreams.rs`)
   - Add complete JSON example to DREAM_EXTRACT_SYSTEM
   - Show entity and relation structure

3. **Adjust temperatures** (config)
   - Set hypothesis_temperature: 0.2
   - Set verify_temperature: 0.1

4. **Test with sample data**
   - Run dream cycle on recent episodic log
   - Verify extraction quality
   - Check citation validation

### Phase 2: Robustness Improvements (2-4 hours)

**Priority: Medium**

5. **Add fuzzy citation matching** (`spark.rs`)
   - Implement normalize_text function
   - Add key term extraction
   - Update citations_valid logic

6. **Add retry with feedback** (`dreams.rs`)
   - Implement run_extraction_with_retry
   - Add error feedback in retry prompt
   - Test with failing cases

7. **Add self-correction phase** (`dreams.rs`)
   - Implement self_correct_extraction
   - Add correction_schema
   - Apply corrections to output

### Phase 3: Advanced Improvements (4-8 hours)

**Priority: Low (optional)**

8. **Chain-of-thought prompting**
   - Add explicit reasoning steps to prompts
   - Ask model to list entities before relations
   - Improve consistency

9. **Two-model approach**
   - Use stronger model for extraction
   - Use 35B for simple tasks
   - Implement gateway routing

10. **Monitor and log**
    - Add metrics for extraction quality
    - Track retry rates
    - Log validation failures

---

## Testing Strategy

### Unit Tests

```rust
#[cfg(test)]
mod extraction_tests {
    use super::*;
    
    #[test]
    fn test_fuzzy_citation_matching() {
        let anchor = "GZMO runs on air-gapped infrastructure with real decisions.";
        let quote = "GZMO operates on air gapped infrastructure";
        assert!(anchor_citation_valid_fuzzy(anchor, quote, 10));
    }
    
    #[test]
    fn test_json_schema_validation() {
        let schema = hypothesis_schema();
        assert!(schema["required"].as_array().unwrap().len() == 1);
        assert_eq!(schema["required"][0], "connection");
    }
}
```

### Integration Tests

```rust
#[tokio::test]
async fn test_dream_cycle_with_retry() {
    let engine = DreamEngine::new(/* ... */);
    let date = NaiveDate::from_ymd_opt(2026, 7, 9).unwrap();
    let report = engine.consolidate(date).await.unwrap();
    
    assert!(report.entities_extracted > 0);
    assert!(report.kg_entities_written > 0);
}
```

### Manual Testing

1. **Run dream cycle on recent data**
   ```bash
   cd ~/github-clone/GZMO
   cargo run -- dream --date 2026-07-09
   ```

2. **Check DREAMS.md output**
   - Verify entities are extracted correctly
   - Check relations are meaningful
   - Ensure citations are valid

3. **Run spark cycle**
   ```bash
   cargo run -- spark --date 2026-07-09
   ```

4. **Monitor logs**
   ```bash
   tail -f ~/.config/gzmo/gzmo-server.log
   ```

---

## Success Criteria

### Phase 1 (Immediate)

- [ ] JSON schemas accept valid output with minor formatting issues
- [ ] Few-shot examples improve extraction consistency
- [ ] Lower temperature reduces JSON formatting errors
- [ ] Dream cycle extracts 5+ entities from typical log

### Phase 2 (Robustness)

- [ ] Fuzzy citation matching accepts paraphrased quotes
- [ ] Retry mechanism recovers from extraction failures
- [ ] Self-correction fixes 80%+ of extraction issues
- [ ] Spark engine promotes 2+ hypotheses per day

### Phase 3 (Advanced)

- [ ] Chain-of-thought improves entity type accuracy
- [ ] Two-model approach reduces errors by 50%
- [ ] Monitoring shows <10% retry rate
- [ ] Validation failure rate <5%

---

## Troubleshooting

### Problem: Model still generates invalid JSON

**Solution:**
1. Add more explicit examples to prompt
2. Reduce temperature further (0.1)
3. Add post-processing to fix JSON
4. Consider using a stronger model for extraction

### Problem: Citations still fail validation

**Solution:**
1. Lower min_chars threshold
2. Use more aggressive fuzzy matching
3. Accept paraphrased citations with lower confidence
4. Skip citation validation for low-stakes links

### Problem: Extraction quality is still poor

**Solution:**
1. Add chain-of-thought prompting
2. Break extraction into smaller steps
3. Use two-model approach (stronger model for extraction)
4. Add manual review for critical data

---

## References

- [Dream Engine Documentation](docs/DREAM_ENGINE.md)
- [Spark Engine Documentation](docs/SPARK_ENGINE.md)
- [Agent Loop Architecture](docs/AGENT_LOOP.md)
- [Memory System Overview](docs/MEMORY_SYSTEM.md)

---

## Contact

For questions or issues, refer to:
- Project README: `~/github-clone/GZMO/README.md`
- Architecture docs: `~/github-clone/GZMO/docs/`
- Issue tracker: GitHub issues

---

**Last Updated:** 2026-07-09  
**Author:** GZMO Implementation Team  
**Status:** Ready for Implementation
