//! # GZMO Chaos Engine
//!
//! Deterministic chaos engine for the GZMO sovereign agent.
//! Provides:
//! - **Lorenz Attractor** — 3D strange attractor driving unpredictable but bounded behavior
//! - **Logistic Map** — fast secondary chaos source coupled to the Lorenz attractor
//! - **Thought Cabinet** — Disco Elysium-inspired thought internalization and crystallization
//! - **Engine State** — energy, phase, death/rebirth lifecycle
//! - **Feedback Channel** — bidirectional link: skill outputs modify the chaos system
//! - **PulseLoop** — unified 174 BPM heartbeat with snapshot broadcasting
//!
//! ## Usage
//!
//! ```rust,no_run
//! use gzmo_chaos::pulse::{PulseLoop, ChaosConfig};
//! use gzmo_chaos::feedback::ChaosEvent;
//!
//! #[tokio::main]
//! async fn main() {
//!     let handle = PulseLoop::start(ChaosConfig::default());
//!
//!     // Read latest chaos state
//!     let snapshot = handle.snapshot_rx.borrow().clone();
//!     println!("Temperature: {}", snapshot.llm_temperature);
//!
//!     // Send feedback from a skill
//!     handle.feedback_tx.send(ChaosEvent::DiceRoll { value: 20, max: 20 }).await.unwrap();
//! }
//! ```

pub mod chaos;
pub mod engine;
pub mod thoughts;
pub mod feedback;
pub mod pulse;
pub mod triggers;
