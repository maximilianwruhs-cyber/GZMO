//! Self-Improving Mode Command — Run GZMO with adaptive parameter optimization

use anyhow::Result;
use std::io::{self, Write};
use std::time::Duration;

use gzmo_core::self_improving::{
    LoopConfig, ModulationParams, SelfImprovingLoop,
};

/// Run interactive self-improving mode
pub async fn run(
    _config: &gzmo_core::config::GzmoConfig,
    detect_repetition: bool,
    adapt_strategy: bool,
) -> Result<()> {
    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║          GZMO Self-Improving Mode                          ║");
    println!("║          Adaptive parameter optimization                   ║");
    println!("╚════════════════════════════════════════════════════════════╝");
    println!();
    
    if !detect_repetition && !adapt_strategy {
        println!("Running with default settings (detection enabled).");
        println!("Use --detect and --adapt flags for more control.");
        println!();
    }
    
    let config = LoopConfig::default();
    let mut loop_sys = SelfImprovingLoop::new(config.clone())
        .with_temperature(0.7);
    
    println!("Configuration:");
    println!("  - History window: {} outputs", config.history_window);
    println!("  - Similarity threshold: {:.2}", config.similarity_threshold);
    println!("  - Min diversity score: {:.2}", config.min_diversity_score);
    println!("  - Strategy update every {} generations", config.strategy_update_interval);
    println!();
    println!("Enter prompts. Type 'quit' to exit.");
    println!("Type 'stats' to see current statistics.");
    println!();
    
    loop {
        print!("prompt> ");
        io::stdout().flush()?;
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();
        
        match input {
            "quit" | "exit" | "q" => {
                println!("Exiting self-improving mode.");
                print_final_stats(&loop_sys);
                break;
            }
            "stats" => {
                print_stats(&loop_sys);
                continue;
            }
            "" => continue,
            _ => {
                // Run the generation cycle
                let start = std::time::Instant::now();
                
                let result = loop_sys.cycle(input, |_prompt, params| {
                    // In a real implementation, this would call the LLM gateway
                    // For now, we simulate with temperature-based variation
                    let output = simulate_generation(input, params);
                    (output, start.elapsed())
                });
                
                println!();
                println!("Output: {}", result.output);
                println!();
                println!("  Temperature: {:.2}", result.params.temperature);
                println!("  Pattern state: {:?}", result.pattern_state);
                println!("  Diversity: {:.2}", result.metrics.lexical_diversity);
                println!("  Latency: {}ms", result.metrics.latency_ms);
                
                if result.was_stuck {
                    println!("  ⚠️  Detected repetition — increased exploration");
                }
                if result.exploration_applied {
                    println!("  🔧 Exploration boost applied");
                }
                
                // Update strategy if needed
                if adapt_strategy && loop_sys.should_update() {
                    loop_sys.apply_optimal();
                    println!("  📝 Strategy updated based on experience");
                }
                
                println!();
            }
        }
    }
    
    Ok(())
}

fn simulate_generation(prompt: &str, params: ModulationParams) -> String {
    // Simple simulation that varies output based on temperature
    // In production, this would call the actual LLM
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    
    let mut hasher = DefaultHasher::new();
    prompt.hash(&mut hasher);
    params.temperature.to_bits().hash(&mut hasher);
    let hash = hasher.finish();
    
    let temp_effect = (params.temperature * 10.0) as u64;
    let variant = (hash + temp_effect) % 3;
    
    match variant {
        0 => format!("Response A (temp={:.2})", params.temperature),
        1 => format!("Response B (temp={:.2})", params.temperature),
        2 => format!("Response C (temp={:.2})", params.temperature),
        _ => unreachable!(),
    }
}

fn print_stats(loop_sys: &SelfImprovingLoop) {
    let stats = loop_sys.stats();
    
    println!();
    println!("┌─────────────────────────────────────────────────────────┐");
    println!("│ Current Statistics                                      │");
    println!("├─────────────────────────────────────────────────────────┤");
    println!("│  Generations:        {:>6}                           │", stats.generation_count);
    println!("│  Stuck detections:   {:>6}                           │", stats.stuck_count);
    println!("│  Escapes:            {:>6}                           │", stats.escape_count);
    if stats.stuck_count > 0 {
        println!("│  Escape rate:        {:>5.1}%                          │", stats.escape_rate * 100.0);
    }
    println!("│  Current exploration: {:>4.1}%                        │", stats.current_exploration * 100.0);
    println!("│  Pattern state:       {:?}                            ", stats.pattern_state);
    println!("└─────────────────────────────────────────────────────────┘");
    println!();
}

fn print_final_stats(loop_sys: &SelfImprovingLoop) {
    let stats = loop_sys.stats();
    
    println!();
    println!("╔═══════════════════════════════════════════════════════════╗");
    println!("║ Final Statistics                                          ║");
    println!("╠═══════════════════════════════════════════════════════════╣");
    println!("║  Total generations:     {:>6}                            ║", stats.generation_count);
    println!("║  Stuck detections:     {:>6}                            ║", stats.stuck_count);
    println!("║  Successful escapes:   {:>6}                            ║", stats.escape_count);
    if stats.stuck_count > 0 {
        println!("║  Escape rate:          {:>5.1}%                           ║", stats.escape_rate * 100.0);
    }
    println!("╚═══════════════════════════════════════════════════════════╝");
    println!();
}
