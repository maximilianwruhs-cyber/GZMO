//! Obolus analytics CLI — token ledger reports on Prime (:8000).

use std::path::PathBuf;

use anyhow::{bail, Result};
use chrono::{DateTime, Duration, Utc};
use gzmo_core::config::GzmoConfig;
use gzmo_core::obolus::{
    aggregate_by_process, compute_energy_correlation, compute_from_sources, synapse_bus_path,
    ObolusLedger, PowerLedger,
};

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

fn power_ledger_path(config: &GzmoConfig) -> PathBuf {
    PathBuf::from(&config.obolus_analytics.power_ledger_path)
}

fn load_power_entries(
    config: &GzmoConfig,
    since: DateTime<Utc>,
) -> Result<Vec<gzmo_core::obolus::PowerLedgerEntry>> {
    PowerLedger::read_since(since, &power_ledger_path(config))
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

fn print_balance_human(
    balance: &gzmo_core::obolus::SystemBalance,
    config: &GzmoConfig,
) {
    println!("Obolus system balance (rolling 1h)");
    println!("  E_total (tokens):    {}", balance.e_total);
    println!(
        "  ctx_% (max process): {:.1}%",
        balance.ctx_pressure_pct
    );
    println!(
        "  peak call ctx_%:     {:.1}%",
        balance.peak_call_ctx_pct
    );
    println!("  token ledger rows:   {}", balance.entry_count);
    if config.obolus_analytics.energy_sampler_enabled {
        println!("  CPU joules (RAPL):   {:.1} J  ({:.4} Wh)", balance.joules_cpu_1h, balance.joules_wh_cpu_1h);
        println!(
            "  GPU joules (est):    {:.1} J  ({:.4} Wh)",
            balance.joules_gpu_est_1h,
            balance.joules_gpu_est_1h / 3600.0
        );
        println!(
            "  Total Wh (est):      {:.4}",
            balance.joules_wh_total_est_1h
        );
        if let Some(tpw) = balance.tokens_per_wh {
            println!("  tokens/Wh:           {:.1}", tpw);
        }
        println!("  power samples:       {}", balance.power_sample_count);
    }
    println!(
        "  limits (token gate): max_e_total={} | max_ctx={}%",
        config.obolus_governance.max_e_total_per_hour,
        config.obolus_governance.max_ctx_pressure_pct,
    );
}

fn print_energy_table(entries: &[gzmo_core::obolus::PowerLedgerEntry]) {
    println!(
        "{:<20} {:>10} {:>10} {:>10} {:>8} {:>8} {:>12}",
        "TIMESTAMP", "CPU_J", "GPU_Jest", "CPU_W", "GPU_W", "CPU_SRC", "GPU_SRC"
    );
    for e in entries {
        println!(
            "{:<20} {:>10.2} {:>10.2} {:>10.1} {:>8.1} {:>8?} {:>12?}",
            e.ts.format("%Y-%m-%d %H:%M"),
            e.cpu_joules,
            e.gpu_joules_est,
            e.cpu_watts_avg,
            e.gpu_power_w.unwrap_or(0.0),
            e.cpu_energy_source,
            e.gpu_energy_source,
        );
    }
}

fn print_correlation_table(report: &gzmo_core::obolus::EnergyCorrelationReport) {
    println!(
        "{:<14} {:>12} {:>12} {:>12} {:>12} {:>8} {:>8}",
        "HOUR", "E_TOTAL", "CPU_J", "GPU_Jest", "Wh_est", "tok/Wh", "samples"
    );
    for b in &report.buckets {
        println!(
            "{:<14} {:>12} {:>12.1} {:>12.1} {:>12.4} {:>12} {:>8}",
            b.hour_bucket,
            b.e_total,
            b.joules_cpu,
            b.joules_gpu_est,
            b.joules_wh_total_est,
            b.tokens_per_wh
                .map(|v| format!("{v:.1}"))
                .unwrap_or_else(|| "-".into()),
            b.power_samples,
        );
    }
    if let Some(r) = report.pearson_tokens_wh {
        println!("\nPearson r(E_total, Wh_est) = {r:.4}");
    } else {
        println!("\nPearson r: insufficient paired hourly buckets");
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
                print_balance_human(&balance, config);
            }
        }
        "energy" => {
            let opts = parse_opts(&args[1..], false);
            let entries = load_power_entries(config, opts.since)?;
            if opts.json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&serde_json::json!({
                        "since": opts.since,
                        "power_ledger": power_ledger_path(config),
                        "samples": entries.len(),
                        "entries": entries,
                    }))?
                );
            } else {
                println!(
                    "Obolus hardware energy samples (since {}) — {}",
                    opts.since.to_rfc3339(),
                    power_ledger_path(config).display()
                );
                print_energy_table(&entries);
                println!("\nSamples: {}", entries.len());
            }
        }
        "sample" => {
            if !config.obolus_analytics.energy_sampler_enabled {
                bail!("energy sampler disabled — set energy_sampler_enabled = true in [obolus_analytics]");
            }
            gzmo_core::obolus::energy_reconcile::sample_and_record_energy(config).await;
            std::thread::sleep(std::time::Duration::from_millis(400));
            let entries = load_power_entries(
                config,
                Utc::now() - Duration::minutes(5),
            )?;
            println!(
                "energy sample recorded — {} recent row(s) in {}",
                entries.len(),
                power_ledger_path(config).display()
            );
        }
        "correlate" => {
            let opts = parse_opts(&args[1..], false);
            let token_entries = load_entries(config, opts.since)?;
            let power_entries = load_power_entries(config, opts.since)?;
            let report = compute_energy_correlation(opts.since, &token_entries, &power_entries);
            if opts.json {
                println!("{}", serde_json::to_string_pretty(&report)?);
            } else {
                println!(
                    "Obolus token ↔ joule correlation (since {})",
                    opts.since.to_rfc3339()
                );
                print_correlation_table(&report);
            }
        }
        "preflight" => {
            let action = args.get(1).map(|s| s.as_str()).unwrap_or("");
            let (obolus_action, tier) = match action {
                "discovery_cycle" => (
                    gzmo_core::obolus::ObolusAction::DiscoveryCycle,
                    gzmo_core::obolus::ObolusTier::SemiAutonomous,
                ),
                "discovery_plan" => (
                    gzmo_core::obolus::ObolusAction::DiscoveryPlan,
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
                "unknown obolus subcommand '{other}' — use: status | report | context | balance | energy | correlate | sample | efficiency | preflight <action> [--json] [--since 24h|7d] [--gaps]"
            );
        }
    }

    Ok(())
}
