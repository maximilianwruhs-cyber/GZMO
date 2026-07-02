//! Output Evaluation — Measures quality and diversity of generated outputs

use std::collections::HashSet;
use std::time::{Duration, Instant};

/// Metrics for evaluating an output
#[derive(Debug, Clone, Copy)]
pub struct EvaluationMetrics {
    /// Lexical diversity (distinct n-grams / total)
    pub lexical_diversity: f64,
    /// Task success (if applicable)
    pub task_success: Option<bool>,
    /// Response latency
    pub latency_ms: u64,
    /// Pattern novelty compared to history
    pub pattern_novelty: f64,
    /// Token count (if available)
    pub token_count: Option<u32>,
    /// Overall quality score (0-1)
    pub quality_score: f64,
}

/// Evaluates output quality and diversity
pub struct OutputEvaluator {
    history: Vec<String>,
    max_history: usize,
    ngram_size: usize,
}

impl OutputEvaluator {
    pub fn new() -> Self {
        Self {
            history: Vec::with_capacity(100),
            max_history: 100,
            ngram_size: 3,
        }
    }

    pub fn evaluate(&mut self, output: &str, latency: Duration) -> EvaluationMetrics {
        let lexical_diversity = self.calculate_diversity(output);
        let pattern_novelty = self.calculate_novelty(output);
        
        self.add_to_history(output);

        EvaluationMetrics {
            lexical_diversity,
            task_success: None,
            latency_ms: latency.as_millis() as u64,
            pattern_novelty,
            token_count: Some(approximate_token_count(output)),
            quality_score: self.compute_quality(lexical_diversity, pattern_novelty, latency),
        }
    }

    fn calculate_diversity(&self, text: &str) -> f64 {
        let words: Vec<&str> = text.split_whitespace().collect();
        if words.len() < self.ngram_size {
            return 1.0;
        }

        let mut total_ngrams = 0usize;
        let mut unique_ngrams = HashSet::new();

        for window in words.windows(self.ngram_size) {
            total_ngrams += 1;
            unique_ngrams.insert(window.join(" "));
        }

        if total_ngrams == 0 {
            return 1.0;
        }

        unique_ngrams.len() as f64 / total_ngrams as f64
    }

    fn calculate_novelty(&self, output: &str) -> f64 {
        if self.history.is_empty() {
            return 1.0;
        }

        let similarities: Vec<f64> = self.history.iter()
            .map(|h| jaccard_similarity(output, h))
            .collect();

        let avg_sim = similarities.iter().sum::<f64>() / similarities.len() as f64;
        1.0 - avg_sim.clamp(0.0, 1.0)
    }

    fn add_to_history(&mut self, output: &str) {
        self.history.push(output.to_string());
        if self.history.len() > self.max_history {
            self.history.remove(0);
        }
    }

    fn compute_quality(&self, diversity: f64, novelty: f64, latency: Duration) -> f64 {
        let latency_score = if latency.as_millis() > 5000 {
            0.5
        } else {
            1.0 - (latency.as_millis() as f64 / 10000.0)
        };

        (diversity * 0.4) + (novelty * 0.4) + (latency_score * 0.2)
    }
}

impl Default for OutputEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

fn jaccard_similarity(a: &str, b: &str) -> f64 {
    let words_a: HashSet<&str> = a.split_whitespace().collect();
    let words_b: HashSet<&str> = b.split_whitespace().collect();

    if words_a.is_empty() || words_b.is_empty() {
        return 0.0;
    }

    let intersection: HashSet<_> = words_a.intersection(&words_b).collect();
    let union: HashSet<_> = words_a.union(&words_b).collect();

    intersection.len() as f64 / union.len() as f64
}

fn approximate_token_count(text: &str) -> u32 {
    let words = text.split_whitespace().count() as f64;
    (words * 1.3).ceil() as u32
}
