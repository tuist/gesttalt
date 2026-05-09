use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};

use serde_json::Value;

use crate::detect::detect_with;
use crate::types::{
    AgentCLI, AgentInfo, AgentKind, Error, Model, ReasoningEffort, Result, SessionEvent,
    SessionRequest, SessionResult,
};

/// Adapter for Anthropic's `claude` CLI (Claude Code).
///
/// Claude does not expose a `models` subcommand: the CLI accepts model aliases
/// (`sonnet`, `opus`, `haiku`) and full names (`claude-sonnet-4-6`, etc.). We
/// surface the documented aliases plus the latest known full names; users can
/// pass any string to `--model`.
///
/// Driving model: `claude --print --model <m> --effort <e> --output-format
/// stream-json --verbose --include-partial-messages "prompt"`. The stream-json
/// output is the same wire format the SDK speaks: `system/init`, `stream_event`
/// (with nested `content_block_delta` etc.), and a terminal `result` line.
pub struct Claude;

impl Claude {
    pub fn new() -> Self {
        Self
    }

    const BIN: &'static str = "claude";
}

impl Default for Claude {
    fn default() -> Self {
        Self::new()
    }
}

const KNOWN_MODELS: &[(&str, Option<&str>)] = &[
    ("claude-opus-4-7", Some("opus")),
    ("claude-sonnet-4-6", Some("sonnet")),
    ("claude-haiku-4-5-20251001", Some("haiku")),
];

impl AgentCLI for Claude {
    fn kind(&self) -> AgentKind {
        AgentKind::Claude
    }

    fn detect(&self) -> Result<Option<AgentInfo>> {
        detect_with(AgentKind::Claude, Self::BIN, &["--version"], |s| {
            // `claude --version` prints e.g. "2.1.131 (Claude Code)".
            s.split_whitespace().next().unwrap_or(s.trim()).to_string()
        })
    }

    fn list_models(&self) -> Result<Vec<Model>> {
        Ok(KNOWN_MODELS
            .iter()
            .map(|(id, alias)| Model {
                id: (*id).to_string(),
                alias: alias.map(|a| a.to_string()),
                provider: "anthropic".to_string(),
                supports_reasoning: true,
            })
            .collect())
    }

    fn reasoning_efforts(&self) -> Vec<ReasoningEffort> {
        // From `claude --help`: `--effort <level>` accepts low, medium, high,
        // xhigh, max.
        vec![
            ReasoningEffort::Low,
            ReasoningEffort::Medium,
            ReasoningEffort::High,
            ReasoningEffort::XHigh,
            ReasoningEffort::Max,
        ]
    }

    fn run_session(
        &self,
        req: &SessionRequest,
        on_event: &mut dyn FnMut(&SessionEvent),
    ) -> Result<SessionResult> {
        let mut cmd = Command::new(Self::BIN);
        cmd.arg("--print")
            .arg("--output-format")
            .arg("stream-json")
            .arg("--verbose")
            // Without this, claude only emits one consolidated `assistant`
            // line per turn, no `content_block_delta` events. We want
            // per-token deltas so the UI can stream output as it arrives.
            .arg("--include-partial-messages");
        if let Some(model) = &req.model {
            cmd.arg("--model").arg(model);
        }
        if let Some(effort) = req.effort {
            cmd.arg("--effort").arg(effort.as_str());
        }
        if let Some(cwd) = &req.cwd {
            cmd.current_dir(cwd);
        }
        cmd.arg(&req.prompt);
        cmd.stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let mut child = cmd.spawn().map_err(|source| Error::Spawn {
            cmd: format!("{} --print", Self::BIN),
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
            for event in parse_event(&value) {
                collect(&event, &mut result);
                on_event(&event);
                result.events.push(event);
            }
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
                cmd: format!("{} --print", Self::BIN),
                status: status.code().unwrap_or(-1),
                stderr,
            });
        }
        Ok(result)
    }
}

fn parse_event(value: &Value) -> Vec<SessionEvent> {
    let ty = value.get("type").and_then(Value::as_str).unwrap_or("");
    match ty {
        "system" => {
            if value.get("subtype").and_then(Value::as_str) == Some("init") {
                if let Some(session_id) = value.get("session_id").and_then(Value::as_str) {
                    return vec![SessionEvent::SessionStarted {
                        session_id: session_id.to_string(),
                    }];
                }
            }
            Vec::new()
        }
        "stream_event" => {
            let inner = match value.get("event") {
                Some(v) => v,
                None => return Vec::new(),
            };
            parse_stream_inner(inner)
        }
        "result" => {
            let text = value
                .get("result")
                .and_then(Value::as_str)
                .map(str::to_string);
            let usage = value.get("usage").cloned();
            vec![SessionEvent::Done { text, usage }]
        }
        // The non-streaming `assistant` summary line duplicates content we
        // already emitted via stream_event deltas; ignore it.
        _ => Vec::new(),
    }
}

fn parse_stream_inner(event: &Value) -> Vec<SessionEvent> {
    let ty = event.get("type").and_then(Value::as_str).unwrap_or("");
    match ty {
        "content_block_delta" => {
            let delta = match event.get("delta") {
                Some(v) => v,
                None => return Vec::new(),
            };
            let dty = delta.get("type").and_then(Value::as_str).unwrap_or("");
            match dty {
                "text_delta" => {
                    let text = delta
                        .get("text")
                        .and_then(Value::as_str)
                        .unwrap_or("")
                        .to_string();
                    vec![SessionEvent::AssistantText { text }]
                }
                "thinking_delta" => {
                    let text = delta
                        .get("thinking")
                        .and_then(Value::as_str)
                        .unwrap_or("")
                        .to_string();
                    vec![SessionEvent::Reasoning { text }]
                }
                _ => Vec::new(),
            }
        }
        "content_block_start" => {
            let block = match event.get("content_block") {
                Some(v) => v,
                None => return Vec::new(),
            };
            let bty = block.get("type").and_then(Value::as_str).unwrap_or("");
            if bty == "tool_use" {
                let name = block
                    .get("name")
                    .and_then(Value::as_str)
                    .unwrap_or("tool")
                    .to_string();
                let input = block.get("input").cloned().unwrap_or(Value::Null);
                return vec![SessionEvent::ToolUse { name, input }];
            }
            Vec::new()
        }
        _ => Vec::new(),
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
        SessionEvent::Done { text: Some(t), .. } => {
            // The terminal `result` line carries the consolidated final text;
            // prefer it over our delta accumulation if we have it.
            into.final_text = Some(t.clone());
        }
        _ => {}
    }
}
