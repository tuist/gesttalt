//! Adapters for CLI coding agents (claude, codex, opencode).
//!
//! Each agent is a separate binary on `PATH`. We do not link any of them in
//! process; we drive them as subprocesses, parse their JSON event streams, and
//! surface a uniform [`AgentCLI`] trait so the rest of `gesttalt` can stay
//! agnostic to which agent the user picked.

mod claude;
mod codex;
mod detect;
mod opencode;
mod types;

pub use claude::Claude;
pub use codex::Codex;
pub use detect::detect_all;
pub use opencode::OpenCode;
pub use types::{
    AgentCLI, AgentInfo, AgentKind, Error, Model, ReasoningEffort, Result, SessionEvent,
    SessionRequest, SessionResult,
};

/// Iterate every agent adapter the crate knows about. Order is stable so
/// callers can rely on it for display.
pub fn all() -> Vec<Box<dyn AgentCLI>> {
    vec![
        Box::new(OpenCode::new()),
        Box::new(Codex::new()),
        Box::new(Claude::new()),
    ]
}
