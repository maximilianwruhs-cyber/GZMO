//! OKCP HTTP client for OKForge (`/api/v1/okf/*`).
//!
//! Session → PATCH concept → commit. Token from env (never from git).

use std::time::Duration;

use anyhow::{bail, Context, Result};
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::config::WikiOkforgeConfig;

/// Reachability probe for the local forge UI (no token, no write).
/// Non-5xx (including 401/403) means the process answered.
pub async fn probe_observatory(base_url: &str) -> Result<(u16, String), String> {
    let base = base_url.trim().trim_end_matches('/');
    if base.is_empty() {
        return Err("okforge url empty".into());
    }
    let http = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .redirect(reqwest::redirect::Policy::limited(2))
        .build()
        .map_err(|e| e.to_string())?;
    let mut last_err = format!("unreachable {base}");
    for path in ["/observatory", "/", "/api/v1/version"] {
        match http.get(format!("{base}{path}")).send().await {
            Ok(resp) => {
                let status = resp.status().as_u16();
                if status < 500 {
                    return Ok((status, path.to_string()));
                }
                last_err = format!("HTTP {status} {path}");
            }
            Err(e) => last_err = format!("{path}: {e}"),
        }
    }
    Err(last_err)
}

#[derive(Debug, Clone)]
pub struct OkforgeClient {
    base: String,
    token: String,
    http: reqwest::Client,
}

#[derive(Debug, Serialize)]
struct SessionStartReq<'a> {
    agent_id: &'a str,
    owner: &'a str,
    repo: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    task_slug: Option<&'a str>,
    /// When set (e.g. `"main"`), commit lands on this branch instead of an agent/* task branch.
    #[serde(skip_serializing_if = "Option::is_none")]
    branch: Option<&'a str>,
}

#[derive(Debug, Deserialize)]
pub struct SessionStartResp {
    pub session_id: String,
    #[serde(default)]
    pub branch: String,
}

#[derive(Debug, Serialize)]
struct ConceptWriteReq<'a> {
    content: &'a str,
    session_id: &'a str,
}

#[derive(Debug, Serialize)]
struct SessionCommitReq<'a> {
    message: &'a str,
    owner: &'a str,
    repo: &'a str,
    open_pr: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    branch: Option<&'a str>,
}

#[derive(Debug, Deserialize)]
pub struct SessionCommitResp {
    #[serde(default)]
    pub commit_sha: String,
    #[serde(default)]
    pub files: usize,
    #[serde(default)]
    pub message: String,
    #[serde(default)]
    pub branch: String,
}

impl OkforgeClient {
    pub fn from_config(cfg: &WikiOkforgeConfig) -> Result<Self> {
        let token_env = cfg.token_env.trim();
        let token = std::env::var(token_env).with_context(|| {
            format!("OKForge token env `{token_env}` not set (export PAT with write:repository)")
        })?;
        if token.trim().is_empty() {
            bail!("OKForge token env `{token_env}` is empty");
        }
        let base = cfg.url.trim().trim_end_matches('/').to_string();
        Ok(Self {
            base,
            token,
            http: reqwest::Client::new(),
        })
    }

    fn url(&self, path: &str) -> String {
        format!("{}/api/v1{}", self.base, path)
    }

    async fn check_ok(&self, resp: reqwest::Response, ctx: &str) -> Result<reqwest::Response> {
        let status = resp.status();
        if status.is_success() {
            return Ok(resp);
        }
        let body = resp.text().await.unwrap_or_default();
        bail!("OKCP {ctx} failed ({status}): {body}");
    }

    pub async fn session_start(
        &self,
        cfg: &WikiOkforgeConfig,
        task_slug: Option<&str>,
    ) -> Result<SessionStartResp> {
        // Direct-to-default when open_pr is false (local overnight path).
        let (task_slug, branch) = if cfg.open_pr {
            (task_slug, None)
        } else {
            (None, Some("main"))
        };
        let body = SessionStartReq {
            agent_id: &cfg.agent_id,
            owner: &cfg.owner,
            repo: &cfg.repo,
            task_slug,
            branch,
        };
        let resp = self
            .http
            .post(self.url("/okf/sessions"))
            .header(AUTHORIZATION, format!("token {}", self.token))
            .header(CONTENT_TYPE, "application/json")
            .json(&body)
            .send()
            .await
            .context("OKCP session start request")?;
        let resp = self.check_ok(resp, "session.start").await?;
        let parsed: SessionStartResp = resp.json().await.context("parse session.start")?;
        info!(session_id = %parsed.session_id, branch = %parsed.branch, "OKCP session started");
        Ok(parsed)
    }

    pub async fn concept_write(
        &self,
        owner: &str,
        repo: &str,
        rel_path: &str,
        content: &str,
        session_id: &str,
    ) -> Result<()> {
        let path = rel_path.trim_start_matches('/');
        let body = ConceptWriteReq {
            content,
            session_id,
        };
        let resp = self
            .http
            .patch(self.url(&format!("/okf/concepts/{owner}/{repo}/{path}")))
            .header(AUTHORIZATION, format!("token {}", self.token))
            .header(CONTENT_TYPE, "application/json")
            .json(&body)
            .send()
            .await
            .context("OKCP concept.write request")?;
        self.check_ok(resp, "concept.write").await?;
        Ok(())
    }

    pub async fn session_commit(
        &self,
        cfg: &WikiOkforgeConfig,
        session_id: &str,
        message: &str,
    ) -> Result<SessionCommitResp> {
        let branch = if cfg.open_pr { None } else { Some("main") };
        let body = SessionCommitReq {
            message,
            owner: &cfg.owner,
            repo: &cfg.repo,
            open_pr: cfg.open_pr,
            branch,
        };
        let resp = self
            .http
            .post(self.url(&format!("/okf/sessions/{session_id}/commit")))
            .header(AUTHORIZATION, format!("token {}", self.token))
            .header(CONTENT_TYPE, "application/json")
            .json(&body)
            .send()
            .await
            .context("OKCP session.commit request")?;
        let resp = self.check_ok(resp, "session.commit").await?;
        let parsed: SessionCommitResp = resp.json().await.context("parse session.commit")?;
        info!(
            sha = %parsed.commit_sha,
            files = parsed.files,
            "OKCP session committed"
        );
        Ok(parsed)
    }
}
