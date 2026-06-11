//! `gzmo mentor` — headless mentor client (daemon socket or local fallback).

use std::fs;
use std::io::{self, IsTerminal, Read};
use anyhow::{Context, Result, bail};

use gzmo_core::config::{GzmoConfig, TaskKind};
use gzmo_core::gateway::GatewayRouter;
use gzmo_core::types::{Message, Role};

use crate::mentor_ipc::{self, MentorRequest, MentorResponse, MentorTurn};
use crate::pedagogy_bridge::PedagogyRuntime;

pub async fn run(config: &GzmoConfig, args: &[String]) -> Result<()> {
    let sub = args.first().map(|s| s.as_str()).unwrap_or("teach");
    match sub {
        "ping" => run_ping(config).await,
        "status" => run_status(config).await,
        "reload" => run_reload(config).await,
        "teach" => {
            let req = parse_teach_request(&args[1..])?;
            run_teach_request(config, req).await
        }
        _ => bail!("Usage: gzmo mentor <ping|status|reload|teach [message]|teach --json-file path>"),
    }
}

fn parse_teach_request(args: &[String]) -> Result<MentorRequest> {
    if let Some(idx) = args.iter().position(|a| a == "--json-file") {
        let path = args
            .get(idx + 1)
            .context("--json-file requires a path argument")?;
        let raw = fs::read_to_string(path).with_context(|| format!("read {path}"))?;
        return serde_json::from_str(&raw).context("parse mentor JSON request file");
    }

    if let Some(msg) = args.first().filter(|s| !s.is_empty() && !s.starts_with('-')) {
        return Ok(MentorRequest {
            method: "teach".into(),
            message: msg.to_string(),
            conversation: vec![],
        });
    }

    if !io::stdin().is_terminal() {
        let mut buf = String::new();
        io::stdin().read_to_string(&mut buf)?;
        let trimmed = buf.trim();
        if trimmed.is_empty() {
            bail!("teach requires a message argument, --json-file, or stdin JSON");
        }
        if trimmed.starts_with('{') {
            let req: MentorRequest = serde_json::from_str(trimmed)?;
            if req.message.is_empty() && req.method == "teach" {
                bail!("JSON teach request missing message");
            }
            return Ok(MentorRequest {
                method: "teach".into(),
                message: req.message,
                conversation: req.conversation,
            });
        }
        return Ok(MentorRequest {
            method: "teach".into(),
            message: trimmed.to_string(),
            conversation: vec![],
        });
    }

    bail!("Usage: gzmo mentor teach <message> | gzmo mentor teach --json-file req.json")
}

async fn run_ping(config: &GzmoConfig) -> Result<()> {
    let resp = call_or_local(
        config,
        MentorRequest {
            method: "ping".into(),
            message: String::new(),
            conversation: vec![],
        },
    )
    .await?;
    if resp.ok {
        println!("{}", resp.response.unwrap_or_else(|| "pong".into()));
        Ok(())
    } else {
        bail!(resp.error.unwrap_or_else(|| "ping failed".into()))
    }
}

async fn run_status(config: &GzmoConfig) -> Result<()> {
    let resp = call_or_local(
        config,
        MentorRequest {
            method: "status".into(),
            message: String::new(),
            conversation: vec![],
        },
    )
    .await?;
    println!(
        "learner={} mentor={} ops_mode={}",
        resp.learner_id.unwrap_or_default(),
        resp.mentor.map(|m| m.to_string()).unwrap_or_else(|| "?".into()),
        resp.ops_mode.map(|m| m.to_string()).unwrap_or_else(|| "?".into()),
    );
    Ok(())
}

async fn run_reload(config: &GzmoConfig) -> Result<()> {
    let resp = call_or_local(
        config,
        MentorRequest {
            method: "reload".into(),
            message: String::new(),
            conversation: vec![],
        },
    )
    .await?;
    if resp.ok {
        println!("{}", resp.response.unwrap_or_else(|| "reloaded".into()));
        Ok(())
    } else {
        bail!(resp.error.unwrap_or_else(|| "reload failed".into()))
    }
}

async fn run_teach_request(config: &GzmoConfig, req: MentorRequest) -> Result<()> {
    let resp = call_or_local(config, req).await?;
    if !resp.ok {
        bail!(resp.error.unwrap_or_else(|| "teach failed".into()));
    }
    if let Some(text) = resp.response {
        println!("{text}");
    } else {
        eprintln!(
            "mentor path skipped: {}",
            resp.error.unwrap_or_else(|| "no response".into())
        );
    }
    Ok(())
}

async fn call_or_local(config: &GzmoConfig, req: MentorRequest) -> Result<MentorResponse> {
    let socket = mentor_ipc::socket_path(config);
    if config.pedagogy.mentor_api_enabled && mentor_ipc::daemon_running() && socket.exists() {
        return mentor_ipc::client_request(&socket, &req)
            .await
            .context("daemon mentor API");
    }
    local_dispatch(config, req).await
}

async fn local_dispatch(config: &GzmoConfig, req: MentorRequest) -> Result<MentorResponse> {
    match req.method.as_str() {
        "ping" => Ok(MentorResponse {
            ok: true,
            response: Some("pong (local)".into()),
            mentor: None,
            ops_mode: None,
            learner_id: Some(config.pedagogy.learner_id().to_string()),
            error: None,
        }),
        "reload" => {
            let mut runtime = PedagogyRuntime::boot(config).await?;
            runtime.reload_from_disk().await?;
            Ok(MentorResponse {
                ok: true,
                response: Some("reloaded (local)".into()),
                mentor: Some(!runtime.session.ops_mode),
                ops_mode: Some(runtime.session.ops_mode),
                learner_id: Some(config.pedagogy.learner_id().to_string()),
                error: None,
            })
        }
        "status" => {
            let mut runtime = PedagogyRuntime::boot(config).await?;
            runtime.reload_from_disk().await?;
            Ok(MentorResponse {
                ok: true,
                response: None,
                mentor: Some(!runtime.session.ops_mode),
                ops_mode: Some(runtime.session.ops_mode),
                learner_id: Some(config.pedagogy.learner_id().to_string()),
                error: None,
            })
        }
        "teach" => local_teach(config, &req).await,
        other => bail!("unknown method: {other}"),
    }
}

async fn local_teach(config: &GzmoConfig, req: &MentorRequest) -> Result<MentorResponse> {
    if !config.pedagogy.enabled {
        bail!("pedagogy disabled");
    }
    let message = req.message.trim();
    if message.is_empty() {
        bail!("message required");
    }
    let router = GatewayRouter::new(config);
    let tutor = router.gateway(TaskKind::Chat);
    let mut runtime = PedagogyRuntime::boot(config).await?;
    runtime.reload_from_disk().await?;
    if runtime.session.ops_mode {
        return Ok(MentorResponse {
            ok: false,
            response: None,
            mentor: Some(false),
            ops_mode: Some(true),
            learner_id: Some(config.pedagogy.learner_id().to_string()),
            error: Some("ops mode active".into()),
        });
    }
    let messages = build_messages(&req.conversation, message);
    let text = runtime
        .maybe_teach(config, &router, tutor.as_ref(), message, &messages)
        .await?;
    Ok(MentorResponse {
        ok: true,
        response: text.clone(),
        mentor: Some(text.is_some()),
        ops_mode: Some(false),
        learner_id: Some(config.pedagogy.learner_id().to_string()),
        error: if text.is_none() {
            Some("not a mentor turn".into())
        } else {
            None
        },
    })
}

fn build_messages(conversation: &[MentorTurn], user_message: &str) -> Vec<Message> {
    let mut messages: Vec<Message> = conversation
        .iter()
        .map(|t| Message {
            role: if t.role.eq_ignore_ascii_case("assistant")
                || t.role.eq_ignore_ascii_case("gzmo")
                || t.role.eq_ignore_ascii_case("mentor")
            {
                Role::Assistant
            } else {
                Role::User
            },
            content: t.content.clone(),
            is_meta: false,
            tool_calls: None,
            tool_call_id: None,
        })
        .collect();
    messages.push(Message {
        role: Role::User,
        content: user_message.to_string(),
        is_meta: false,
        tool_calls: None,
        tool_call_id: None,
    });
    messages
}
