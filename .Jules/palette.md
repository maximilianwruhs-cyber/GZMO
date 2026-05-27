## 2024-05-24 - Empty States for Hotkey Discoverability
**Learning:** TUIs often suffer from poor discoverability for global hotkeys (like opening palettes or quitting) because there's no visual real estate for persistent toolbars. Users frequently get stuck trying to exit or access menus.
**Action:** Always leverage the empty state of primary input fields as a non-intrusive billboard for the most critical global hotkeys (e.g., Ctrl+P, Ctrl+C).

## 2024-05-27 - TUI Mouse Capture and Scrolling
**Learning:** Enabling `crossterm::event::EnableMouseCapture` swallows native terminal scrolling.
**Action:** Any scrollable component (like the transcript) MUST explicitly implement `MouseEventKind::ScrollUp` and `MouseEventKind::ScrollDown` in its event handler, otherwise users lose expected mouse scrolling functionality entirely.
