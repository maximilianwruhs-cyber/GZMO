//! `gzmo session close` — end-of-session takeaway ritual → distill queue / one-shot distill.

use anyhow::{bail, Context, Result};
use tracing::info;

use gzmo_core::config::GzmoConfig;
use gzmo_core::memory::scratch::{
    messages_to_transcript, DistillJob, DistillSource, ScratchService,
};
use gzmo_core::session::SessionManager;
use gzmo_core::types::{Message, Role};

use crate::distill_cmd;

fn usage() -> ! {
    eprintln!(
        "Usage:\n  \
         gzmo session close [session-id] --takeaway \"durable fact…\" [--now]\n  \
         gzmo session close [session-id] --takeaway-file PATH [--now]\n\n  \
         Appends durable takeaways to the session, then enqueues distill (default)\n  \
         or runs `gzmo distill` immediately with `--now`.\n  \
         Omit session-id to use the most recently active session."
    );
    std::process::exit(2);
}

pub async fn run(config: &GzmoConfig, args: &[String]) -> Result<()> {
    if args.first().map(|s| s.as_str()) != Some("close") {
        usage();
    }
    let rest = &args[1..];

    let mut session_id: Option<String> = None;
    let mut takeaways: Vec<String> = Vec::new();
    let mut now = false;
    let mut i = 0;
    while i < rest.len() {
        match rest[i].as_str() {
            "--now" => {
                now = true;
                i += 1;
            }
            "--takeaway" => {
                let Some(t) = rest.get(i + 1) else {
                    bail!("--takeaway requires a string");
                };
                takeaways.push(t.clone());
                i += 2;
            }
            "--takeaway-file" => {
                let Some(path) = rest.get(i + 1) else {
                    bail!("--takeaway-file requires a path");
                };
                let raw = std::fs::read_to_string(path)
                    .with_context(|| format!("read takeaway file {path}"))?;
                for line in raw.lines() {
                    let line = line.trim();
                    if !line.is_empty() && !line.starts_with('#') {
                        takeaways.push(line.to_string());
                    }
                }
                i += 2;
            }
            "-h" | "--help" => usage(),
            other if !other.starts_with('-') && session_id.is_none() => {
                session_id = Some(other.to_string());
                i += 1;
            }
            other => bail!("unknown argument: {other}"),
        }
    }

    if takeaways.is_empty() {
        bail!("at least one --takeaway or --takeaway-file entry is required");
    }

    let mgr = SessionManager::new(&config.session_distill.sessions_dir);
    let id = if let Some(id) = session_id {
        id
    } else {
        let metas = mgr.list().await?;
        let Some(meta) = metas.first() else {
            bail!(
                "no sessions in {}",
                config.session_distill.sessions_dir.display()
            );
        };
        meta.id.clone()
    };

    let mut session = mgr.load(&id).await?;
    for t in &takeaways {
        session.messages.push(Message {
            role: Role::User,
            content: format!("[TAKEAWAY] {t}"),
            is_meta: false,
            tool_calls: None,
            tool_call_id: None,
        });
        session.messages.push(Message {
            role: Role::Assistant,
            content: format!("Recorded durable takeaway for distill: {t}"),
            is_meta: false,
            tool_calls: None,
            tool_call_id: None,
        });
    }

    mgr.save(
        &session.id,
        session.name.as_deref(),
        &session.messages,
        session.created_at,
    )
    .await?;
    info!(
        session_id = %session.id,
        takeaways = takeaways.len(),
        "Session close: takeaways appended"
    );

    if now {
        println!(
            "Session `{}` closed with {} takeaway(s); running distill --now…",
            session.id,
            takeaways.len()
        );
        distill_cmd::run(config, Some(session.id.clone())).await?;
        return Ok(());
    }

    let scratch = ScratchService::from_config(&config.redis, &config.context_memory).await;
    let transcript = messages_to_transcript(&session.messages);
    scratch
        .enqueue_distill(DistillJob {
            session_id: session.id.clone(),
            transcript,
            source: DistillSource::MainArchive,
        })
        .await?;

    println!(
        "Session `{}` closed with {} takeaway(s); distill job enqueued.\n\
         Start `gzmo serve` (or `gzmo distill {}`) to metabolize.",
        session.id,
        takeaways.len(),
        session.id
    );
    Ok(())
}
