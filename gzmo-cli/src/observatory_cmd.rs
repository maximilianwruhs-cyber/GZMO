//! `gzmo observatory` — ecosystem health LED board (TUI).

use anyhow::Result;
use gzmo_core::config::GzmoConfig;

use crate::tui::boards::health_board;

pub async fn run(config: &GzmoConfig) -> Result<()> {
    health_board::run(config).await
}
