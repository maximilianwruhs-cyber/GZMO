## 2024-05-24 - Empty States for Hotkey Discoverability
**Learning:** TUIs often suffer from poor discoverability for global hotkeys (like opening palettes or quitting) because there's no visual real estate for persistent toolbars. Users frequently get stuck trying to exit or access menus.
**Action:** Always leverage the empty state of primary input fields as a non-intrusive billboard for the most critical global hotkeys (e.g., Ctrl+P, Ctrl+C).

## 2025-02-12 - Mouse Scrolling with Mouse Capture
**Learning:** When `crossterm::event::EnableMouseCapture` is active in a terminal app, native terminal scrolling is swallowed.
**Action:** Always explicitly handle `MouseEventKind::ScrollUp` and `MouseEventKind::ScrollDown` in scrollable TUI components to restore native scroll functionality for better usability.
