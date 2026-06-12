## 2024-06-12 - Mouse Capture Swallows Native Scroll
**Learning:** Enabling `crossterm::event::EnableMouseCapture` to support terminal mouse events has the side effect of swallowing native terminal scrolling. Any scrollable or navigatable TUI component (like lists or transcripts) must explicitly handle `MouseEventKind::ScrollUp` and `MouseEventKind::ScrollDown` to restore basic scrolling functionality.
**Action:** Added explicit mouse scroll handling to the Command Palette component. Ensure all future scrollable TUI lists explicitly handle `MouseEventKind`.
