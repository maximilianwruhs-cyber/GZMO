//! Repetition Detection — Identifies when outputs are stuck in patterns
//!
//! Uses n-gram similarity analysis to detect repetitive outputs.
//! When the system generates similar responses repeatedly, it
//! signals that exploration (higher temperature) is needed.

use std::collections::{HashSet, VecDeque};

fn normalize_text(s: &str) -> String {
    s.to_lowercase()
        .split_whitespace()
        .map(|w| w.trim_matches(|c: char| c.is_ascii_punctuation()))
        .filter(|w| !w.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

/// State of pattern detection for a sequence of outputs
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PatternState {
    /// No repetition detected
    Novel,
    /// Mild similarity detected
    Similar,
    /// Strong repetition pattern detected
    Stuck,
    /// Very high similarity — system is in a loop
    Loop,
}

impl PatternState {
    /// Whether this state indicates need for exploration
    pub fn needs_exploration(&self) -> bool {
        matches!(self, PatternState::Stuck | PatternState::Loop)
    }
    
    /// Recommended exploration level (0.0-1.0)
    pub fn exploration_level(&self) -> f64 {
        match self {
            PatternState::Novel => 0.0,
            PatternState::Similar => 0.3,
            PatternState::Stuck => 0.7,
            PatternState::Loop => 1.0,
        }
    }
}

/// Detects repetitive patterns in generated outputs
pub struct RepetitionDetector {
    /// Sliding window of recent outputs
    history: VecDeque<String>,
    /// Maximum history size
    window_size: usize,
    /// Similarity threshold for "similar" classification
    similar_threshold: f64,
    /// Similarity threshold for "stuck" classification  
    stuck_threshold: f64,
    /// Similarity threshold for "loop" classification
    loop_threshold: f64,
    /// N-gram size for analysis
    ngram_size: usize,
}

impl RepetitionDetector {
    /// Create new detector with default settings
    pub fn new() -> Self {
        Self {
            history: VecDeque::with_capacity(10),
            window_size: 10,
            similar_threshold: 0.7,
            stuck_threshold: 0.85,
            loop_threshold: 0.95,
            ngram_size: 3,
        }
    }
    
    /// Create with custom configuration
    pub fn with_config(
        window_size: usize,
        stuck_threshold: f64,
        loop_threshold: f64,
        ngram_size: usize,
    ) -> Self {
        Self {
            history: VecDeque::with_capacity(window_size),
            window_size,
            similar_threshold: stuck_threshold * 0.8,
            stuck_threshold,
            loop_threshold,
            ngram_size,
        }
    }
    
    /// Add an output to history and return pattern state
    pub fn add_output(&mut self, output: String) -> PatternState {
        // Add to history
        self.history.push_back(output);
        if self.history.len() > self.window_size {
            self.history.pop_front();
        }
        
        // Analyze current state
        self.analyze()
    }
    
    /// Analyze current history and return pattern state
    fn analyze(&self) -> PatternState {
        if self.history.len() < 2 {
            return PatternState::Novel;
        }
        
        // Compare consecutive outputs
        let similarities: Vec<f64> = self.consecutive_similarities();
        
        if similarities.is_empty() {
            return PatternState::Novel;
        }
        
        // Calculate statistics
        let avg_similarity = similarities.iter().sum::<f64>() / similarities.len() as f64;
        let max_similarity = similarities.iter().cloned().fold(0.0, f64::max);
        let recent_similar = similarities.iter().rev().take(3).sum::<f64>() 
            / similarities.iter().rev().take(3).count() as f64;
        
        // Determine state
        if max_similarity >= self.loop_threshold && recent_similar >= self.loop_threshold {
            PatternState::Loop
        } else if avg_similarity >= self.stuck_threshold && recent_similar >= self.stuck_threshold {
            PatternState::Stuck
        } else if avg_similarity >= self.similar_threshold {
            PatternState::Similar
        } else {
            PatternState::Novel
        }
    }
    
    /// Calculate n-gram similarity between two strings
    fn ngram_similarity(&self, a: &str, b: &str) -> f64 {
        let a = normalize_text(a);
        let b = normalize_text(b);
        if a.is_empty() || b.is_empty() {
            return 0.0;
        }

        let a_words: Vec<&str> = a.split_whitespace().collect();
        let b_words: Vec<&str> = b.split_whitespace().collect();

        let a_ngrams: HashSet<String> = a_words
            .windows(self.ngram_size)
            .map(|w| w.join(" "))
            .collect();

        let b_ngrams: HashSet<String> = b_words
            .windows(self.ngram_size)
            .map(|w| w.join(" "))
            .collect();
        
        if a_ngrams.is_empty() || b_ngrams.is_empty() {
            return 0.0;
        }
        
        let intersection: HashSet<_> = a_ngrams.intersection(&b_ngrams).collect();
        let union: HashSet<_> = a_ngrams.union(&b_ngrams).collect();
        
        intersection.len() as f64 / union.len() as f64
    }
    
    /// Calculate Jaccard similarity (word-based)
    fn jaccard_similarity(&self, a: &str, b: &str) -> f64 {
        let a = normalize_text(a);
        let b = normalize_text(b);
        let words_a: HashSet<&str> = a.split_whitespace().collect();
        let words_b: HashSet<&str> = b.split_whitespace().collect();
        
        if words_a.is_empty() || words_b.is_empty() {
            return 0.0;
        }
        
        let intersection: HashSet<_> = words_a.intersection(&words_b).collect();
        let union: HashSet<_> = words_a.union(&words_b).collect();
        
        intersection.len() as f64 / union.len() as f64
    }
    
    /// Calculate similarity between all consecutive outputs
    fn consecutive_similarities(&self) -> Vec<f64> {
        let history_vec: Vec<_> = self.history.iter().collect();
        
        (1..history_vec.len())
            .map(|i| {
                let a = history_vec[i - 1];
                let b = history_vec[i];
                // Blend n-gram and Jaccard similarities
                let ngram_sim = self.ngram_similarity(a, b);
                let jaccard_sim = self.jaccard_similarity(a, b);
                (ngram_sim * 0.6) + (jaccard_sim * 0.4)
            })
            .collect()
    }
    
    /// Check if system is currently stuck
    pub fn is_stuck(&self) -> bool {
        self.analyze().needs_exploration()
    }
    
    /// Get recommended exploration level (0.0-1.0)
    pub fn exploration_level(&self) -> f64 {
        self.analyze().exploration_level()
    }
    
    /// Get current pattern state
    pub fn current_state(&self) -> PatternState {
        self.analyze()
    }
    
    /// Get detailed statistics
    pub fn statistics(&self) -> DetectorStats {
        let similarities = self.consecutive_similarities();
        
        DetectorStats {
            history_size: self.history.len(),
            avg_similarity: if similarities.is_empty() {
                0.0
            } else {
                similarities.iter().sum::<f64>() / similarities.len() as f64
            },
            max_similarity: similarities.iter().cloned().fold(0.0, f64::max),
            min_similarity: similarities.iter().cloned().fold(1.0, f64::min),
            recent_outputs: self.history.iter().rev().take(3).cloned().collect(),
        }
    }
    
    /// Clear history
    pub fn clear(&mut self) {
        self.history.clear();
    }
    
    /// Get window size
    pub fn window_size(&self) -> usize {
        self.window_size
    }
}

impl Default for RepetitionDetector {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics from repetition detector
#[derive(Debug, Clone)]
pub struct DetectorStats {
    pub history_size: usize,
    pub avg_similarity: f64,
    pub max_similarity: f64,
    pub min_similarity: f64,
    pub recent_outputs: Vec<String>,
}

/// Check if a specific output appears to be a template or repetition
pub fn is_template_output(output: &str, templates: &[&str]) -> bool {
    let output_lower = output.to_lowercase();
    templates.iter().any(|&t| {
        let t_lower = t.to_lowercase();
        output_lower.contains(&t_lower) || similarity_score(output, t) > 0.8
    })
}

/// Calculate simple similarity score between two strings
fn similarity_score(a: &str, b: &str) -> f64 {
    let a_words: HashSet<&str> = a.split_whitespace().collect();
    let b_words: HashSet<&str> = b.split_whitespace().collect();
    
    if a_words.is_empty() || b_words.is_empty() {
        return 0.0;
    }
    
    let intersection: HashSet<_> = a_words.intersection(&b_words).collect();
    let union: HashSet<_> = a_words.union(&b_words).collect();
    
    intersection.len() as f64 / union.len() as f64
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn detects_novel_outputs() {
        let mut detector = RepetitionDetector::new();
        
        let state1 = detector.add_output("The cat sat on the mat.".to_string());
        assert_eq!(state1, PatternState::Novel);
        
        let state2 = detector.add_output("Dogs run in the park happily.".to_string());
        assert!(!state2.needs_exploration());
    }
    
    #[test]
    fn detects_similar_outputs() {
        let mut detector = RepetitionDetector::new();
        
        detector.add_output("The cat sat on the mat.".to_string());
        detector.add_output("The cat sat on the mat today.".to_string());
        let state = detector.add_output("The cat sat on the mat yesterday.".to_string());
        
        // Should be at least Similar, possibly Stuck
        assert!(matches!(state, PatternState::Similar | PatternState::Stuck | PatternState::Loop));
    }
    
    #[test]
    fn detects_loop() {
        let mut detector = RepetitionDetector::new();
        
        // Add same output 5 times
        for _ in 0..5 {
            detector.add_output("The cat sat on the mat.".to_string());
        }
        
        let state = detector.current_state();
        assert_eq!(state, PatternState::Loop);
        assert!(state.needs_exploration());
        assert_eq!(state.exploration_level(), 1.0);
    }
    
    #[test]
    fn calculates_similarity() {
        let detector = RepetitionDetector::new();
        
        let sim = detector.jaccard_similarity(
            "the cat sat on the mat",
            "the cat sat on the mat"
        );
        assert!((sim - 1.0).abs() < 0.01);
        
        let sim = detector.jaccard_similarity(
            "the cat sat on the mat",
            "dogs run in the park"
        );
        assert!(sim < 0.3);
    }
    
    #[test]
    fn statistics_computed() {
        let mut detector = RepetitionDetector::new();
        
        detector.add_output("A B C D E".to_string());
        detector.add_output("A B C D F".to_string());
        detector.add_output("A B C D G".to_string());
        
        let stats = detector.statistics();
        assert_eq!(stats.history_size, 3);
        assert!(stats.avg_similarity > 0.5);
        assert!(stats.max_similarity >= stats.min_similarity);
    }
}