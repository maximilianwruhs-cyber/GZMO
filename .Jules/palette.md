## 2024-05-24 - Context over code
**Learning:** Evaluated the assumption that `[Ctrl+P] Palette` was hallucinated. However, tracing through `gzmo-cli/src/tui/app.rs` shows that `Ctrl+P` is indeed a registered global shortcut mapped to `ToggleCommandPalette`.
**Action:** The hint is correct and reflects the actual application behavior.
