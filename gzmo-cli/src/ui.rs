//! Splash screen and ANSI formatting utilities.

#[allow(dead_code)]
use std::io::Write;

/// Print the GZMO splash screen.
pub fn splash(model: &str, vault_count: usize, engine_ms: u128) {
    let banner = format!(
        r#"
  ○ ● ○ ● ○ ● ○ ● ○ ● ○ ● ○ ● ○ ● ○ ● ○ ● ○ ● ○ ● ○ ●
  ●                                                 ○
  ○         ★  S T E P   R I G H T   U P  ★         ●
  ●                                                 ○
  ○       ██████╗ ███████╗███╗   ███╗ ██████╗       ●
  ●      ██╔════╝ ╚══███╔╝████╗ ████║██╔═══██╗      ○
  ○      ██║  ███╗  ███╔╝ ██╔████╔██║██║   ██║      ●
  ●      ██║   ██║ ███╔╝  ██║╚██╔╝██║██║   ██║      ○
  ○      ╚██████╔╝███████╗██║ ╚═╝ ██║╚██████╔╝      ●
  ●       ╚═════╝ ╚══════╝╚═╝     ╚═╝ ╚═════╝       ○
  ○                                                 ●
  ●     ⚙  The Incredible Mechanical Marvel  ⚙      ○
  ○         100% Local · Air-Gapped · Rust          ●
  ●              Engine: ONLINE ({engine_ms}ms)               ○
  ○ Host: {model} | Vault: {vault_count} records ●
  ●                                                 ○
  ○   /quit exit · /clear reset · /vault memory · /remember store   ●
  ●                                                 ○
  ● ○ ● ○ ● ○ ● ○ ● ○ ● ○ ● ○ ● ○ ● ○ ● ○ ● ○ ● ○ ● ○
"#
    );
    eprintln!("{banner}");
}

/// Print the user input prompt.
pub fn prompt() {
    eprint!("\n  ★ you › ");
    let _ = std::io::stderr().flush();
}

/// Print agent text to stderr with formatting.
pub fn agent_text(text: &str) {
    eprint!("{text}");
    let _ = std::io::stderr().flush();
}

/// Newline after agent response.
pub fn agent_end() {
    eprintln!();
}
