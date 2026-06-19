//! HSP (:8001) hardware snapshot client for GPU power telemetry.

use anyhow::Result;
use serde_json::Value;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct HspSnapshot {
    pub gpu_power_w: Option<f64>,
    pub gpu_power_w_per_device: Vec<(String, f64)>,
    pub cpu_power_w: Option<f64>,
    pub energy_j_total: Option<f64>,
    pub metrics_source: String,
    pub warming_up: bool,
}

fn value_as_f64(v: &Value) -> Option<f64> {
    match v {
        Value::Number(n) => n.as_f64(),
        Value::String(s) => s.parse().ok(),
        _ => None,
    }
}

fn parse_snapshot(body: &Value) -> HspSnapshot {
    if body.get("status").and_then(|v| v.as_str()) == Some("warming_up") {
        return HspSnapshot {
            warming_up: true,
            metrics_source: body
                .get("metrics_source")
                .and_then(|v| v.as_str())
                .unwrap_or("hsp")
                .to_string(),
            ..Default::default()
        };
    }

    let mut per_device = Vec::new();
    if let Value::Object(map) = body {
        for (key, val) in map {
            if key.ends_with("_power_w") && key.starts_with("gpu") && key != "gpu_power_w" {
                if let Some(p) = value_as_f64(val) {
                    per_device.push((key.clone(), p));
                }
            }
        }
    }

    let explicit_gpu = body.get("gpu_power_w").and_then(value_as_f64);
    let gpu_power_w = explicit_gpu.or_else(|| {
        if per_device.is_empty() {
            None
        } else {
            Some(per_device.iter().map(|(_, p)| *p).sum())
        }
    });

    HspSnapshot {
        gpu_power_w,
        gpu_power_w_per_device: per_device,
        cpu_power_w: body.get("power_w").and_then(value_as_f64),
        energy_j_total: body.get("energy_j_total").and_then(value_as_f64),
        metrics_source: body
            .get("metrics_source")
            .and_then(|v| v.as_str())
            .unwrap_or("hsp")
            .to_string(),
        warming_up: false,
    }
}

pub async fn fetch_hsp_state(base_url: &str) -> Result<Option<HspSnapshot>> {
    let url = base_url.trim();
    if url.is_empty() {
        return Ok(None);
    }

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(2))
        .build()?;

    let resp = match client.get(url).send().await {
        Ok(r) if r.status().is_success() => r,
        _ => return Ok(None),
    };

    let body: Value = match resp.json().await {
        Ok(v) => v,
        Err(_) => return Ok(None),
    };

    Ok(Some(parse_snapshot(&body)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn parse_hsp_gpu_fields() {
        let body = json!({
            "gpu0_power_w": 100.5,
            "gpu1_power_w": 80.0,
            "power_w": 12.3,
            "metrics_source": "local"
        });
        let snap = parse_snapshot(&body);
        assert!((snap.gpu_power_w.unwrap() - 180.5).abs() < 0.01);
        assert!((snap.cpu_power_w.unwrap() - 12.3).abs() < 0.01);
    }

    #[test]
    fn parse_warming_up() {
        let body = json!({"status": "warming_up"});
        let snap = parse_snapshot(&body);
        assert!(snap.warming_up);
    }
}
