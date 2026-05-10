use std::fmt;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("agent `{0}` is not installed on PATH")]
    NotInstalled(&'static str),
    #[error("failed to spawn `{cmd}`: {source}")]
    Spawn {
        cmd: String,
        #[source]
        source: std::io::Error,
    },
    #[error("`{cmd}` exited with status {status}: {stderr}")]
    NonZeroExit {
        cmd: String,
        status: i32,
        stderr: String,
    },
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("could not parse output of `{cmd}`: {detail}")]
    Parse { cmd: String, detail: String },
}

pub type Result<T> = std::result::Result<T, Error>;

/// The set of CLI agents we know how to drive. New agents append to this enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AgentKind {
    OpenCode,
    Codex,
    Claude,
}

impl AgentKind {
    pub fn as_str(self) -> &'static str {
        match self {
            AgentKind::OpenCode => "opencode",
            AgentKind::Codex => "codex",
            AgentKind::Claude => "claude",
        }
    }
}

impl fmt::Display for AgentKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentInfo {
    pub kind: AgentKind,
    pub executable: PathBuf,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Model {
    /// The identifier passed to `--model`.
    pub id: String,
    /// Optional short label (e.g. `sonnet`, `opus`) the CLI also accepts.
    pub alias: Option<String>,
    /// Provider portion when the agent groups models by provider, otherwise
    /// the agent's own name.
    pub provider: String,
    /// True if the model accepts a reasoning-effort knob.
    pub supports_reasoning: bool,
}

/// Reasoning levels the various CLIs support. Each adapter returns the subset
/// it actually accepts; we deliberately use a wider enum here because the CLIs
/// don't agree on the same set (claude has `max`, codex has `none`, etc.) and
/// it's useful to display the full picture in the UI.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ReasoningEffort {
    None,
    Minimal,
    Low,
    Medium,
    High,
    XHigh,
    Max,
}

impl ReasoningEffort {
    pub fn as_str(self) -> &'static str {
        match self {
            ReasoningEffort::None => "none",
            ReasoningEffort::Minimal => "minimal",
            ReasoningEffort::Low => "low",
            ReasoningEffort::Medium => "medium",
            ReasoningEffort::High => "high",
            ReasoningEffort::XHigh => "xhigh",
            ReasoningEffort::Max => "max",
        }
    }
}

impl fmt::Display for ReasoningEffort {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone)]
pub struct SessionRequest {
    pub prompt: String,
    pub model: Option<String>,
    pub effort: Option<ReasoningEffort>,
    pub cwd: Option<PathBuf>,
}

impl SessionRequest {
    pub fn new(prompt: impl Into<String>) -> Self {
        Self {
            prompt: prompt.into(),
            model: None,
            effort: None,
            cwd: None,
        }
    }
    pub fn model(mut self, m: impl Into<String>) -> Self {
        self.model = Some(m.into());
        self
    }
    pub fn effort(mut self, e: ReasoningEffort) -> Self {
        self.effort = Some(e);
        self
    }
    pub fn cwd(mut self, p: impl Into<PathBuf>) -> Self {
        self.cwd = Some(p.into());
        self
    }
}

/// Normalised event stream every adapter emits while a session is running. We
/// pick a small common vocabulary; the original payload is preserved in
/// [`SessionEvent::Raw::value`] for callers that want full fidelity.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SessionEvent {
    /// The session has been assigned an id by the agent.
    SessionStarted { session_id: String },
    /// The model produced visible text (assistant turn output).
    AssistantText { text: String },
    /// The model produced a chunk of reasoning ("thinking"). Some agents only
    /// emit this when explicitly asked for it.
    Reasoning { text: String },
    /// The agent invoked a tool / executed a shell command.
    ToolUse {
        name: String,
        input: serde_json::Value,
    },
    /// The session finished. `text` is the final assistant message if the
    /// adapter could pull one out.
    Done {
        text: Option<String>,
        usage: Option<serde_json::Value>,
    },
    /// Anything we recognised but didn't normalise.
    Raw { value: serde_json::Value },
}

#[derive(Debug, Clone, Default)]
pub struct SessionResult {
    pub session_id: Option<String>,
    pub final_text: Option<String>,
    pub events: Vec<SessionEvent>,
}

pub trait AgentCLI: Send + Sync {
    fn kind(&self) -> AgentKind;

    /// Locate the binary on PATH and read its version. Returns `None` if the
    /// binary is not installed.
    fn detect(&self) -> Result<Option<AgentInfo>>;

    /// Models exposed by this CLI. May hit the network or read local config.
    fn list_models(&self) -> Result<Vec<Model>>;

    /// Reasoning levels this CLI accepts. The set is intrinsic to the CLI,
    /// not per-model — the same enum applies to whatever `--model` the caller
    /// picks, even if the underlying provider silently ignores it.
    fn reasoning_efforts(&self) -> Vec<ReasoningEffort>;

    /// Run a one-shot session. Events are streamed via `on_event`; the result
    /// also collects them so callers that don't want streaming can ignore the
    /// callback. Pass `|_| {}` if you don't care.
    fn run_session(
        &self,
        req: &SessionRequest,
        on_event: &mut dyn FnMut(&SessionEvent),
    ) -> Result<SessionResult>;
}
