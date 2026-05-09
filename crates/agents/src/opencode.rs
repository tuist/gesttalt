use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};

use serde_json::Value;

use crate::detect::{detect_with, run_capture};
use crate::types::{
    AgentCLI, AgentInfo, AgentKind, Error, Model, ReasoningEffort, Result, SessionEvent,
    SessionRequest, SessionResult,
};

/// Adapter for the `opencode` CLI (https://opencode.ai).
///
/// Driving model: `opencode run --format json -m provider/model --variant <effort> "prompt"`.
/// `--format json` makes opencode emit one JSON event per line on stdout, with
/// shapes like `{"type":"text","part":{"text":...}}` and a final
/// `{"type":"step_finish","part":{"tokens":...,"cost":...}}`.
pub struct OpenCode;

impl OpenCode {
    pub fn new() -> Self {
        Self
    }

    const BIN: &'static str = "opencode";
}

impl Default for OpenCode {
    fn default() -> Self {
        Self::new()
    }
}

impl AgentCLI for OpenCode {
    fn kind(&self) -> AgentKind {
        AgentKind::OpenCode
    }

    fn detect(&self) -> Result<Option<AgentInfo>> {
        detect_with(AgentKind::OpenCode, Self::BIN, &["--version"], |s| {
            // `opencode --version` prints just the version, e.g. "1.3.15".
            s.trim().to_string()
        })
    }

    fn list_models(&self) -> Result<Vec<Model>> {
        let stdout = run_capture(Self::BIN, &["models"])?;
        let mut out = Vec::new();
        for line in stdout.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            // opencode prints `provider/model` per line.
            let (provider, _model) = line.split_once('/').unwrap_or(("opencode", line));
            out.push(Model {
                id: line.to_string(),
                alias: None,
                provider: provider.to_string(),
                supports_reasoning: true,
            });
        }
        if out.is_empty() {
            return Err(Error::Parse {
                cmd: "opencode models".into(),
                detail: "no models returned".into(),
            });
        }
        Ok(out)
    }

    fn reasoning_efforts(&self) -> Vec<ReasoningEffort> {
        // From `opencode run --help`: --variant accepts provider-specific
        // reasoning effort, "e.g. high, max, minimal". We expose the three
        // they document.
        vec![
            ReasoningEffort::Minimal,
            ReasoningEffort::High,
            ReasoningEffort::Max,
        ]
    }

    fn run_session(
        &self,
        req: &SessionRequest,
        on_event: &mut dyn FnMut(&SessionEvent),
    ) -> Result<SessionResult> {
        let mut cmd = Command::new(Self::BIN);
        cmd.arg("run").arg("--format").arg("json");
        if let Some(model) = &req.model {
            cmd.arg("-m").arg(model);
        }
        if let Some(effort) = req.effort {
            cmd.arg("--variant").arg(effort.as_str());
        }
        if let Some(cwd) = &req.cwd {
            cmd.arg("--dir").arg(cwd);
        }
        // The prompt is positional: `opencode run [message..]`. Pass via -- to
        // be safe with leading dashes in the user prompt.
        cmd.arg("--").arg(&req.prompt);
        cmd.stdout(Stdio::piped()).stderr(Stdio::piped());

        let mut child = cmd.spawn().map_err(|source| Error::Spawn {
            cmd: format!("{} run", Self::BIN),
            source,
        })?;
        let stdout = child.stdout.take().expect("stdout was piped");
        let reader = BufReader::new(stdout);

        let mut result = SessionResult::default();
        for line in reader.lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }
            let value: Value = match serde_json::from_str(&line) {
                Ok(v) => v,
                Err(_) => continue,
            };
            let event = parse_event(&value);
            collect(&event, &mut result);
            on_event(&event);
            result.events.push(event);
        }

        let status = child.wait()?;
        if !status.success() {
            let stderr = child
                .stderr
                .take()
                .map(|mut s| {
                    let mut buf = String::new();
                    let _ = std::io::Read::read_to_string(&mut s, &mut buf);
                    buf
                })
                .unwrap_or_default();
            return Err(Error::NonZeroExit {
                cmd: format!("{} run", Self::BIN),
                status: status.code().unwrap_or(-1),
                stderr,
            });
        }
        Ok(result)
    }
}

fn parse_event(value: &Value) -> SessionEvent {
    let ty = value.get("type").and_then(Value::as_str).unwrap_or("");
    let part = value.get("part");
    match ty {
        "step_start" => {
            let session_id = value
                .get("sessionID")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string();
            SessionEvent::SessionStarted { session_id }
        }
        "text" => {
            let text = part
                .and_then(|p| p.get("text"))
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string();
            SessionEvent::AssistantText { text }
        }
        "reasoning" => {
            let text = part
                .and_then(|p| p.get("text"))
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string();
            SessionEvent::Reasoning { text }
        }
        "tool" | "tool_call" | "tool_use" => {
            let name = part
                .and_then(|p| p.get("name"))
                .and_then(Value::as_str)
                .unwrap_or("tool")
                .to_string();
            let input = part
                .and_then(|p| p.get("input"))
                .cloned()
                .unwrap_or(Value::Null);
            SessionEvent::ToolUse { name, input }
        }
        "step_finish" => {
            let usage = part.and_then(|p| p.get("tokens")).cloned();
            SessionEvent::Done { text: None, usage }
        }
        _ => SessionEvent::Raw {
            value: value.clone(),
        },
    }
}

fn collect(event: &SessionEvent, into: &mut SessionResult) {
    match event {
        SessionEvent::SessionStarted { session_id } => {
            into.session_id = Some(session_id.clone());
        }
        SessionEvent::AssistantText { text } => match &mut into.final_text {
            Some(existing) => existing.push_str(text),
            None => into.final_text = Some(text.clone()),
        },
        _ => {}
    }
}
