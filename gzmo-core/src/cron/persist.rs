//! Surgical TOML persistence for cron wizard edits (preserves comments where possible).

use std::path::Path;

use anyhow::{bail, Context, Result};

use crate::config::CustomCronJob;

/// Update a builtin daily hour/minute pair in the live config file.
pub fn persist_builtin_schedule(
    config_path: &Path,
    job_id: &str,
    hour: u32,
    minute: u32,
) -> Result<()> {
    if hour > 23 || minute > 59 {
        bail!("Invalid time {hour:02}:{minute:02}");
    }
    let content = std::fs::read_to_string(config_path)
        .with_context(|| format!("read {}", config_path.display()))?;
    let updated = match job_id {
        "dream" => patch_section_keys(
            &content,
            "[dreams]",
            &[
                ("cron_hour", &hour.to_string()),
                ("cron_minute", &minute.to_string()),
            ],
        )?,
        "distill" => patch_section_keys(
            &content,
            "[session_distill]",
            &[
                ("cron_hour", &hour.to_string()),
                ("cron_minute", &minute.to_string()),
            ],
        )?,
        "promote" => patch_section_keys(
            &content,
            "[metabolism]",
            &[
                ("promote_cron_hour", &hour.to_string()),
                ("promote_cron_minute", &minute.to_string()),
            ],
        )?,
        "embed" => patch_section_keys(
            &content,
            "[metabolism]",
            &[
                ("embed_cron_hour", &hour.to_string()),
                ("embed_cron_minute", &minute.to_string()),
            ],
        )?,
        "wiki_push" => patch_section_keys(
            &content,
            "[wiki]",
            &[
                ("push_cron_hour", &hour.to_string()),
                ("push_cron_minute", &minute.to_string()),
            ],
        )?,
        "spark" => bail!("Use persist for spark hours separately (multi-slot)"),
        other => bail!("Unknown builtin job: {other}"),
    };
    std::fs::write(config_path, updated)
        .with_context(|| format!("write {}", config_path.display()))?;
    Ok(())
}

/// Flip enabled flags for builtins.
pub fn persist_builtin_enabled(config_path: &Path, job_id: &str, enabled: bool) -> Result<()> {
    let content = std::fs::read_to_string(config_path)
        .with_context(|| format!("read {}", config_path.display()))?;
    let flag = if enabled { "true" } else { "false" };
    let updated = match job_id {
        "dream" => patch_section_keys(&content, "[dreams]", &[("enabled", flag)])?,
        "distill" => patch_section_keys(
            &content,
            "[session_distill]",
            &[("enabled", flag), ("daemon_scheduled", flag)],
        )?,
        "promote" | "embed" => {
            patch_section_keys(&content, "[metabolism]", &[("enabled", flag)])?
        }
        "spark" => patch_section_keys(&content, "[spark]", &[("enabled", flag)])?,
        "wiki_push" => {
            // Enabling wiki_push requires wiki.enabled; backend stays as configured.
            patch_section_keys(&content, "[wiki]", &[("enabled", flag)])?
        }
        other => bail!("Unknown builtin job: {other}"),
    };
    std::fs::write(config_path, updated)
        .with_context(|| format!("write {}", config_path.display()))?;
    Ok(())
}

/// Upsert `[cron.jobs.<id>]` block (rewrites that section; appends if missing).
pub fn persist_custom_job(config_path: &Path, id: &str, job: &CustomCronJob) -> Result<()> {
    validate_id(id)?;
    let content = std::fs::read_to_string(config_path)
        .with_context(|| format!("read {}", config_path.display()))?;
    let header = format!("[cron.jobs.{id}]");
    let block = format_custom_block(id, job);
    let updated = if let Some(range) = find_table_range(&content, &header) {
        let mut out = String::new();
        out.push_str(&content[..range.0]);
        out.push_str(&block);
        if !block.ends_with('\n') {
            out.push('\n');
        }
        out.push_str(&content[range.1..]);
        out
    } else {
        let mut out = content;
        if !out.ends_with('\n') {
            out.push('\n');
        }
        out.push('\n');
        out.push_str(&block);
        if !block.ends_with('\n') {
            out.push('\n');
        }
        out
    };
    std::fs::write(config_path, updated)
        .with_context(|| format!("write {}", config_path.display()))?;
    Ok(())
}

/// Remove a `[cron.jobs.<id>]` table.
pub fn remove_custom_job(config_path: &Path, id: &str) -> Result<()> {
    validate_id(id)?;
    let content = std::fs::read_to_string(config_path)
        .with_context(|| format!("read {}", config_path.display()))?;
    let header = format!("[cron.jobs.{id}]");
    let Some(range) = find_table_range(&content, &header) else {
        bail!("Custom job '{id}' not found in {}", config_path.display());
    };
    let mut out = String::new();
    out.push_str(&content[..range.0]);
    out.push_str(&content[range.1..]);
    std::fs::write(config_path, out).with_context(|| format!("write {}", config_path.display()))?;
    Ok(())
}

fn validate_id(id: &str) -> Result<()> {
    if id.is_empty()
        || !id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
    {
        bail!("Invalid job id '{id}' (use letters, digits, _ or -)");
    }
    Ok(())
}

fn format_custom_block(id: &str, job: &CustomCronJob) -> String {
    let kind = match job.kind {
        crate::config::CustomCronKind::Shell => "shell",
        crate::config::CustomCronKind::Prompt => "prompt",
    };
    let mut lines = vec![
        format!("[cron.jobs.{id}]"),
        format!("enabled = {}", job.enabled),
        format!("schedule = \"{}\"", escape_toml_str(&job.schedule)),
        format!("kind = \"{kind}\""),
    ];
    if !job.description.is_empty() {
        lines.push(format!(
            "description = \"{}\"",
            escape_toml_str(&job.description)
        ));
    }
    match job.kind {
        crate::config::CustomCronKind::Shell => {
            lines.push(format!("command = \"{}\"", escape_toml_str(&job.command)));
        }
        crate::config::CustomCronKind::Prompt => {
            lines.push(format!("prompt = \"{}\"", escape_toml_str(&job.prompt)));
        }
    }
    lines.push(String::new());
    lines.join("\n")
}

fn escape_toml_str(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

/// Find byte range of a TOML table starting at `header` through the next `[` section or EOF.
fn find_table_range(content: &str, header: &str) -> Option<(usize, usize)> {
    let start = content.find(header)?;
    if start > 0 && !content.is_char_boundary(start) {
        return None;
    }
    if start > 0 && content.as_bytes().get(start - 1) != Some(&b'\n') {
        // Require section header at line start (avoid matching inside comments/strings).
        return None;
    }
    let after_header_line = content[start..]
        .find('\n')
        .map(|i| start + i + 1)
        .unwrap_or(content.len());
    let mut end = content.len();
    let mut pos = after_header_line;
    while pos < content.len() {
        let rest = &content[pos..];
        let line_end = rest.find('\n').unwrap_or(rest.len());
        let line = &rest[..line_end];
        let t = line.trim();
        if t.starts_with('[') {
            end = pos;
            break;
        }
        pos += line_end + usize::from(line_end < rest.len());
    }
    Some((start, end))
}

fn patch_section_keys(content: &str, section: &str, keys: &[(&str, &str)]) -> Result<String> {
    let Some((sec_start, sec_end)) = find_table_range(content, section) else {
        bail!("Section {section} not found in config");
    };
    let before = &content[..sec_start];
    let section_body = &content[sec_start..sec_end];
    let after = &content[sec_end..];

    let mut lines: Vec<String> = section_body.lines().map(|l| l.to_string()).collect();
    for (key, value) in keys {
        let mut found = false;
        for line in lines.iter_mut() {
            let trimmed = line.trim();
            if trimmed.starts_with('#') || !trimmed.contains('=') {
                continue;
            }
            let name = trimmed.split('=').next().unwrap_or("").trim();
            if name == *key {
                let indent = line.len() - line.trim_start().len();
                *line = format!("{}{} = {}", " ".repeat(indent), key, value);
                found = true;
                break;
            }
        }
        if !found {
            // Insert after section header
            if lines.is_empty() {
                lines.push(section.to_string());
            }
            lines.insert(1, format!("{key} = {value}"));
        }
    }
    let mut out = String::new();
    out.push_str(before);
    out.push_str(&lines.join("\n"));
    if !section_body.ends_with('\n') && after.is_empty() {
        // keep
    } else if !out.ends_with('\n') {
        out.push('\n');
    }
    // Preserve whether original section ended with newline before `after`
    if section_body.ends_with('\n') && !out.ends_with('\n') {
        out.push('\n');
    }
    out.push_str(after);
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::CustomCronKind;
    use std::io::Write;

    #[test]
    fn upsert_and_remove_custom_job() {
        let dir = std::env::temp_dir().join(format!("gzmo-cron-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("gzmo.toml");
        let mut f = std::fs::File::create(&path).unwrap();
        writeln!(f, "[dreams]\nenabled = true\ncron_hour = 1\ncron_minute = 0\n").unwrap();

        let job = CustomCronJob {
            enabled: true,
            schedule: "0 6 * * *".into(),
            kind: CustomCronKind::Shell,
            command: "echo hi".into(),
            prompt: String::new(),
            description: "test".into(),
        };
        persist_custom_job(&path, "morning", &job).unwrap();
        let text = std::fs::read_to_string(&path).unwrap();
        assert!(text.contains("[cron.jobs.morning]"));
        assert!(text.contains("command = \"echo hi\""));

        persist_builtin_schedule(&path, "dream", 2, 15).unwrap();
        let text = std::fs::read_to_string(&path).unwrap();
        assert!(text.contains("cron_hour = 2"));
        assert!(text.contains("cron_minute = 15"));

        remove_custom_job(&path, "morning").unwrap();
        let text = std::fs::read_to_string(&path).unwrap();
        assert!(!text.contains("[cron.jobs.morning]"));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
