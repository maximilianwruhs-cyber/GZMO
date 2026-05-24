## 2024-05-24 - Input Field Visual Cues
**Learning:** In a terminal TUI, users sometimes don't know what state the input field is in or if it has focus. A common UX improvement is to change the border color or add a visual cue when the input field is active or inactive. Furthermore, an empty text field offers no direction.
**Action:** Add placeholder text '(Type a message... Ctrl+P for palette)' with dim styling to the input component when it is empty to instruct the user what to do and how to access additional commands.
