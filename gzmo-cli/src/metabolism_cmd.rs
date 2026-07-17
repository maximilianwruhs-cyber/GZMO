//! `gzmo metabolism` — overnight job board (TUI).

use anyhow::Result;
use gzmo_core::config::GzmoConfig;

use crate::tui::boards::metabolism_board;

pub async fn run(config: &GzmoConfig) -> Result<()> {
    metabolism_board::run(config).await
}
