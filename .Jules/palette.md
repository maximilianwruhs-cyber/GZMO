## 2026-06-11 - Restoring Mouse Scroll When Mouse Capture is Enabled
**Learning:** When `crossterm::event::EnableMouseCapture` is active, native terminal scrolling is swallowed. Components like scrollable lists will break for mouse users unless explicitly handled.
**Action:** Any scrollable TUI components must explicitly capture and map `MouseEventKind::ScrollUp` and `MouseEventKind::ScrollDown` to their navigation methods to restore accessibility.
