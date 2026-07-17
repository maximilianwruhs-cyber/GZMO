use gzmo_chaos::pulse::ChaosSnapshot;

#[derive(Debug, Clone)]
pub enum Action {
    /// Signal to tick/update internal physics or chaos engine
    Tick,
    /// Instructs the TUI to re-render the screen
    Render,
    /// Sent when the window has resized
    Resize(u16, u16),
    /// Halt the TUI application
    Quit,
    /// Clears the screen (like Ctrl+L)
    ClearScreen,
    // --- User Input Actions ---
    TranscriptClear,
    TranscriptRestore(Vec<gzmo_core::types::Message>),

    /// User has finished typing and submitted a prompt
    SubmitInput(String),
    /// Toggles the floating Command Palette overlay
    ToggleCommandPalette,
    /// Absolute open/closed state for the command palette (single source of truth)
    SetCommandPalette(bool),
    /// Toggle the keyboard help overlay
    ToggleHelp,
    /// Absolute open/closed state for the help overlay
    SetHelp(bool),

    // --- Background Agent Actions ---
    /// New chunk of stream text from the LLM
    AgentTokenStream(String),
    /// Complete agent response block
    AgentResponse(String),
    /// Sync mutated conversation history back to the AgentComponent
    AgentMessagesSync(Vec<gzmo_core::types::Message>),

    // --- Chaos & Pulse Actions ---
    /// Periodic snapshot from the PulseLoop
    ChaosSnapshot(ChaosSnapshot),
    /// Lore or trigger event from the background engine
    LoreEvent(String, String, String), // (category, author, text)
    /// Hardware load telemetry
    Telemetry(f32, f32), // (CPU Usage %, MEM Usage %)
    /// LLM engine reachability (status label, latency or empty)
    EngineHealth(String, String),

    // --- Trigger Engine Actions ---
    /// Autonomous trigger notification from the chaos engine
    TriggerNotification(String),
    /// Trigger wants to run a skill (skill_name, args)
    TriggerSkill(String, String),
    /// Trigger wants to inject a prompt into the conversation
    TriggerInject(String),
}
