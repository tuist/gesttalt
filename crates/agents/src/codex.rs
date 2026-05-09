use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};

use serde_json::Value;

use crate::detect::detect_with;
use crate::types::{
    AgentCLI, AgentInfo, AgentKind, Error, Model, ReasoningEffort, Result, SessionEvent,
    SessionRequest, SessionResult,
};

/// Adapter for OpenAI's `codex` CLI.
///
/// Codex does not expose a `models` subcommand: the model is configured in
/// `~/.codex/config.toml` and selected via `-m`. We therefore surface a small
/// curated set of well-known models plus whatever the user has configured as
/// their default. Users can pass any model id at runtime — the curated list
/// is just for UX.
///
/// Driving model: `codex exec --json -m <model> -c model_reasoning_effort=<effort>
/// --skip-git-repo-check "prompt"`. JSONL events on stdout look like
/// `{"type":"thread.started",...}`, `{"type":"item.completed", "item":{...}}`,
/// `{"type":"turn.completed", "usage":{...}}`.
pub struct Codex;

impl Codex {
    pub fn new() -> Self {
        Self
    }

    const BIN: &'static str = "codex";
}

impl Default for Codex {
    fn default() -> Self {
        Self::new()
    }
}

/// Curated list of models that codex is known to ship support for as of the
/// 0.128.x line. We keep this small — codex itself accepts any string and the
/// user's `config.toml` may name something else.
const CURATED_MODELS: &[&str] = &[
    "gpt-5.4",
    "gpt-5.4-mini",
    "gpt-5.3-codex",
    "gpt-5.2-codex",
    "gpt-5.1-codex",
    "gpt-5.1-codex-mini",
    "gpt-5-codex",
];

impl AgentCLI for Codex {
    fn kind(&self) -> AgentKind {
        AgentKind::Codex
    }

    fn detect(&self) -> Result<Option<AgentInfo>> {
        detect_with(AgentKind::Codex, Self::BIN, &["--version"], |s| {
            // `codex --version` prints e.g. "codex-cli 0.128.0".
            s.split_whitespace()
                .last()
                .unwrap_or(s.trim())
                .to_string()
        })
    }

    fn list_models(&self) -> Result<Vec<Model>> {
        let mut out: Vec<Model> = CURATED_MODELS
            .iter()
            .map(|id| Model {
                id: (*id).to_string(),
                alias: None,
                provider: "openai".to_string(),
                supports_reasoning: true,
            })
            .collect();

        // Best-effort: include the user's configured default model if it is
        // not already in the curated list. We parse a minimal subset of TOML
        // (top-level `model = "..."`) by hand to avoid a TOML dep.
        if let Some(home) = std::env::var_os("HOME") {
            let path = std::path::Path::new(&home).join(".codex/config.toml");
            if let Ok(contents) = std::fs::read_to_string(&path) {
                if let Some(model) = extract_top_level_string(&contents, "model") {
                    if !out.iter().any(|m| m.id == model) {
                        out.insert(
                            0,
                            Model {
                                id: model,
                                alias: Some("configured-default".into()),
                                provider: "openai".into(),
                                supports_reasoning: true,
                            },
                        );
                    }
                }
            }
        }
        Ok(out)
    }

    fn reasoning_efforts(&self) -> Vec<ReasoningEffort> {
        // Discovered empirically from codex's error message:
        // "expected one of `none`, `minimal`, `low`, `medium`, `high`, `xhigh`".
        vec![
            ReasoningEffort::None,
            ReasoningEffort::Minimal,
            ReasoningEffort::Low,
            ReasoningEffort::Medium,
            ReasoningEffort::High,
            ReasoningEffort::XHigh,
        ]
    }

    fn run_session(
        &self,
        req: &SessionRequest,
        on_event: &mut dyn FnMut(&SessionEvent),
    ) -> Result<SessionResult> {
        let mut cmd = Command::new(Self::BIN);
        cmd.arg("exec").arg("--json").arg("--skip-git-repo-check");
        if let Some(model) = &req.model {
            cmd.arg("-m").arg(model);
        }
        if let Some(effort) = req.effort {
            cmd.arg("-c")
                .arg(format!("model_reasoning_effort=\"{}\"", effort.as_str()));
        }
        if let Some(cwd) = &req.cwd {
            cmd.arg("-C").arg(cwd);
        }
        cmd.arg(&req.prompt);
        // Detach stdin so codex doesn't try to read additional input from a
        // piped parent process.
        cmd.stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let mut child = cmd.spawn().map_err(|source| Error::Spawn {
            cmd: format!("{} exec", Self::BIN),
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
                cmd: format!("{} exec", Self::BIN),
                status: status.code().unwrap_or(-1),
                stderr,
            });
        }
        Ok(result)
    }
}

fn parse_event(value: &Value) -> SessionEvent {
    let ty = value.get("type").and_then(Value::as_str).unwrap_or("");
    match ty {
        "thread.started" => {
            let session_id = value
                .get("thread_id")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string();
            SessionEvent::SessionStarted { session_id }
        }
        "item.completed" | "item.updated" => {
            let item = value.get("item");
            let item_type = item
                .and_then(|i| i.get("type"))
                .and_then(Value::as_str)
                .unwrap_or("");
            match item_type {
                "agent_message" => {
                    let text = item
                        .and_then(|i| i.get("text"))
                        .and_then(Value::as_str)
                        .unwrap_or("")
                        .to_string();
                    SessionEvent::AssistantText { text }
                }
                "reasoning" => {
                    let text = item
                        .and_then(|i| i.get("text"))
                        .and_then(Value::as_str)
                        .unwrap_or("")
                        .to_string();
                    SessionEvent::Reasoning { text }
                }
                "command_execution" | "tool_call" | "function_call" => {
                    let name = item
                        .and_then(|i| i.get("name"))
                        .and_then(Value::as_str)
                        .unwrap_or(item_type)
                        .to_string();
                    let input = item
                        .and_then(|i| i.get("arguments"))
                        .cloned()
                        .or_else(|| item.and_then(|i| i.get("command")).cloned())
                        .unwrap_or(Value::Null);
                    SessionEvent::ToolUse { name, input }
                }
                _ => SessionEvent::Raw {
                    value: value.clone(),
                },
            }
        }
        "turn.completed" => SessionEvent::Done {
            text: None,
            usage: value.get("usage").cloned(),
        },
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

/// Pull a top-level `key = "value"` out of a TOML file. We only need the very
/// first occurrence and intentionally do not handle tables or arrays. Avoids
/// adding a TOML parser dependency for one read.
fn extract_top_level_string(contents: &str, key: &str) -> Option<String> {
    for raw in contents.lines() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if line.starts_with('[') {
            // We stop at the first table header — keys after that are nested.
            return None;
        }
        if let Some((k, v)) = line.split_once('=') {
            if k.trim() == key {
                let v = v.trim().trim_matches('"');
                return Some(v.to_string());
            }
        }
    }
    None
}
