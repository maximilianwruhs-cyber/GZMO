//! Obolus analytics CLI — token ledger reports on Prime (:8000).

use std::path::PathBuf;

use anyhow::{bail, Result};
use chrono::{DateTime, Duration, Utc};
use gzmo_core::config::GzmoConfig;
use gzmo_core::obolus::{aggregate_by_process, compute_from_sources, synapse_bus_path, ObolusLedger};

struct ReportOpts {
    since: DateTime<Utc>,
    json: bool,
    show_gaps: bool,
    sort_by_context: bool,
}

fn parse_since(args: &[String]) -> DateTime<Utc> {
    let mut since = Utc::now() - Duration::hours(24);
    let mut i = 0;
    while i < args.len() {
        if args[i] == "--since" {
            if let Some(val) = args.get(i + 1) {
                if let Ok(h) = val.trim_end_matches('h').parse::<i64>() {
                    if val.ends_with('h') {
                        since = Utc::now() - Duration::hours(h);
                    } else if val.ends_with('d') {
                        if let Ok(d) = val.trim_end_matches('d').parse::<i64>() {
                            since = Utc::now() - Duration::days(d);
                        }
                    }
                }
                i += 2;
                continue;
            }
        }
        i += 1;
    }
    since
}

fn parse_opts(args: &[String], default_context_sort: bool) -> ReportOpts {
    ReportOpts {
        since: parse_since(args),
        json: args.iter().any(|a| a == "--json"),
        show_gaps: args.iter().any(|a| a == "--gaps"),
        sort_by_context: default_context_sort || args.iter().any(|a| a == "--by" && false),
    }
}

fn ledger_path(config: &GzmoConfig) -> PathBuf {
    PathBuf::from(&config.obolus_analytics.ledger_path)
}

fn load_entries(config: &GzmoConfig, since: DateTime<Utc>) -> Result<Vec<gzmo_core::obolus::LedgerEntry>> {
    ObolusLedger::read_since(since, &ledger_path(config))
}

fn print_table(
    rollups: &[gzmo_core::obolus::ProcessRollup],
    prime_ctx: u64,
    show_gaps: bool,
) {
    println!(
        "Prime context window: {} tokens ({}K)",
        prime_ctx,
        prime_ctx / 1024
    );
    println!(
        "{:<28} {:>6} {:>10} {:>10} {:>10} {:>8} {:>10}",
        "PROCESS", "CALLS", "INPUT", "OUTPUT", "TOTAL", "CTX_%", "PEAK_IN"
    );
    for r in rollups {
        if show_gaps && r.gaps > 0 {
            println!(
                "  ({} calls missing usage tokens)",
                r.gaps
            );
        }
        println!(
            "{:<28} {:>6} {:>10} {:>10} {:>10} {:>7.1}% {:>10}",
            truncate_process(&r.process, 28),
            r.call_count,
            r.sum_input,
            r.sum_output,
            r.sum_total,
            r.context_share_pct,
            r.max_input_single_call,
        );
    }
}

fn truncate_process(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}…", &s[..max.saturating_sub(1)])
    }
}

fn print_efficiency_table(rollups: &[gzmo_core::obolus::EfficiencyRollup]) {
    println!(
        "{:<24} {:>10} {:>6} {:>6} {:>12} {:>8} {:>8}",
        "PROCESS", "E_TOTAL", "Q", "I", "η/Mtok", "OUTCOMES", "CALLS"
    );
    for r in rollups {
        println!(
            "{:<24} {:>10} {:>6.3} {:>6.3} {:>12.6} {:>8} {:>8}",
            truncate_process(&r.process, 24),
            r.e_total,
            r.q,
            r.i,
            r.eta_per_million_tokens,
            r.outcome_samples,
            r.ledger_calls,
        );
    }
    println!("\nη = (Q·I)/E_total  |  η/Mtok = η × 10⁶ (readable scale)");
}

pub async fn run(args: &[String], config: &GzmoConfig) -> Result<()> {
    if !config.obolus_analytics.enabled {
        bail!("obolus analytics disabled — set [obolus_analytics] enabled = true in gzmo.toml");
    }

    let sub = args.first().map(|s| s.as_str()).unwrap_or("status");

    match sub {
        "status" => {
            let since = Utc::now() - Duration::days(1);
            let entries = load_entries(config, since)?;
            let rollups = aggregate_by_process(
                &entries,
                config.obolus_analytics.prime_context_tokens,
                config.obolus_analytics.tokens_per_obl,
            );
            println!("Obolus token ledger — today (top by E_total)");
            print_table(
                &rollups.iter().take(10).cloned().collect::<Vec<_>>(),
                config.obolus_analytics.prime_context_tokens,
                false,
            );
            if !rollups.is_empty() {
                println!("\nTop by context pressure (ctx_%):");
                let mut by_ctx = rollups.clone();
                by_ctx.sort_by(|a, b| {
                    b.context_share_pct
                        .partial_cmp(&a.context_share_pct)
                        .unwrap_or(std::cmp::Ordering::Equal)
                });
                print_table(
                    &by_ctx.iter().take(5).cloned().collect::<Vec<_>>(),
                    config.obolus_analytics.prime_context_tokens,
                    false,
                );
            }
            println!("\nLedger: {}", ledger_path(config).display());
            println!("Entries (24h): {}", entries.len());
        }
        "report" => {
            let opts = parse_opts(&args[1..], false);
            let entries = load_entries(config, opts.since)?;
            let rollups = aggregate_by_process(
                &entries,
                config.obolus_analytics.prime_context_tokens,
                config.obolus_analytics.tokens_per_obl,
            );
            if opts.json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&serde_json::json!({
                        "since": opts.since,
                        "entries": entries.len(),
                        "rollups": rollups,
                    }))?
                );
            } else {
                print_table(
                    &rollups,
                    config.obolus_analytics.prime_context_tokens,
                    opts.show_gaps,
                );
            }
        }
        "context" => {
            let opts = parse_opts(&args[1..], true);
            let entries = load_entries(config, opts.since)?;
            let mut rollups = aggregate_by_process(
                &entries,
                config.obolus_analytics.prime_context_tokens,
                config.obolus_analytics.tokens_per_obl,
            );
            rollups.sort_by(|a, b| {
                b.context_share_pct
                    .partial_cmp(&a.context_share_pct)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
            if opts.json {
                println!("{}", serde_json::to_string_pretty(&rollups)?);
            } else {
                println!("Obolus context pressure by process (sorted by ctx_%):");
                print_table(
                    &rollups,
                    config.obolus_analytics.prime_context_tokens,
                    opts.show_gaps,
                );
            }
        }
        "balance" => {
            let balance = gzmo_core::obolus::gate::load_balance_since(
                config,
                chrono::Utc::now() - chrono::Duration::hours(1),
            )?;
            if args.iter().any(|a| a == "--json") {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&serde_json::json!({
                        "balance_1h": balance,
                        "limits": {
                            "max_e_total_per_hour": config.obolus_governance.max_e_total_per_hour,
                            "max_ctx_pressure_pct": config.obolus_governance.max_ctx_pressure_pct,
                        },
                    }))?
                );
            } else {
                println!("Obolus system balance (rolling 1h)");
                println!("  E_total:      {}", balance.e_total);
                println!("  ctx_% (max process): {:.1}%", balance.ctx_pressure_pct);
                println!("  peak call ctx_%:     {:.1}%", balance.peak_call_ctx_pct);
                println!("  ledger entries: {}", balance.entry_count);
                println!(
                    "  limits: E_total <= {} | ctx_% <= {}",
                    config.obolus_governance.max_e_total_per_hour,
                    config.obolus_governance.max_ctx_pressure_pct,
                );
            }
        }
        "preflight" => {
            let action = args.get(1).map(|s| s.as_str()).unwrap_or("");
            let (obolus_action, tier) = match action {
                "discovery_cycle" => (
                    gzmo_core::obolus::ObolusAction::DiscoveryCycle,
                    gzmo_core::obolus::ObolusTier::SemiAutonomous,
                ),
                "spawn_discovery_fix" => (
                    gzmo_core::obolus::ObolusAction::SpawnDiscoveryFix,
                    gzmo_core::obolus::ObolusTier::Autonomous,
                ),
                "dice_loop" => (
                    gzmo_core::obolus::ObolusAction::DiceLoop,
                    gzmo_core::obolus::ObolusTier::Autonomous,
                ),
                "dream_tick" => (
                    gzmo_core::obolus::ObolusAction::DreamTick,
                    gzmo_core::obolus::ObolusTier::Autonomous,
                ),
                "spark_tick" => (
                    gzmo_core::obolus::ObolusAction::SparkTick,
                    gzmo_core::obolus::ObolusTier::Autonomous,
                ),
                "spawn_session_triage" => (
                    gzmo_core::obolus::ObolusAction::SpawnSessionTriage,
                    gzmo_core::obolus::ObolusTier::Autonomous,
                ),
                "operator_chat" => (
                    gzmo_core::obolus::ObolusAction::OperatorChat,
                    gzmo_core::obolus::ObolusTier::Operator,
                ),
                other => bail!("unknown preflight action '{other}'"),
            };
            let verdict =
                gzmo_core::obolus::gate::evaluate_from_config(config, obolus_action, tier)?;
            let balance = gzmo_core::obolus::gate::load_balance_since(
                config,
                chrono::Utc::now() - chrono::Duration::hours(1),
            )?;
            let exit_code = match &verdict {
                gzmo_core::obolus::ObolusVerdict::Allow => 0,
                gzmo_core::obolus::ObolusVerdict::Warn { .. } => 0,
                gzmo_core::obolus::ObolusVerdict::Defer { .. } => 2,
                gzmo_core::obolus::ObolusVerdict::Deny { .. } => 1,
            };
            if args.iter().any(|a| a == "--json") {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&serde_json::json!({
                        "action": action,
                        "tier": format!("{:?}", tier),
                        "verdict": format!("{:?}", verdict),
                        "balance_1h": balance,
                        "limits": {
                            "max_e_total_per_hour": config.obolus_governance.max_e_total_per_hour,
                            "max_ctx_pressure_pct": config.obolus_governance.max_ctx_pressure_pct,
                        },
                    }))?
                );
            } else {
                println!(
                    "obolus preflight {action}: {verdict:?}\n  1h E_total={} ctx_%={:.1} peak_call={:.1}%",
                    balance.e_total,
                    balance.ctx_pressure_pct,
                    balance.peak_call_ctx_pct,
                );
            }
            if exit_code != 0 {
                std::process::exit(exit_code);
            }
        }
        "efficiency" => {
            let opts = parse_opts(&args[1..], false);
            let entries = load_entries(config, opts.since)?;
            let bus = synapse_bus_path(config);
            let rollups = compute_from_sources(
                &entries,
                &bus,
                opts.since,
                config.obolus_analytics.prime_context_tokens,
                config.obolus_analytics.tokens_per_obl,
            )?;
            if opts.json {
                println!("{}", serde_json::to_string_pretty(&rollups)?);
            } else {
                println!("Obolus Wirkungsgrad η (since {}):", opts.since.to_rfc3339());
                print_efficiency_table(&rollups);
            }
        }
        other => {
            bail!(
                "unknown obolus subcommand '{other}' — use: status | report | context | balance | efficiency | preflight <action> [--json] [--since 24h|7d] [--gaps]"
            );
        }
    }

    Ok(())
}
