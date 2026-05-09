//! End-to-end probe for the CLI agent adapters.
//!
//! For each agent we know about: detect it, list its models, list its
//! reasoning levels, and send a tiny prompt to verify the JSON event stream
//! parses. Output is plain text, formatted for a terminal.
//!
//! Usage:
//!     cargo run -p agents --bin agents-probe
//!     cargo run -p agents --bin agents-probe -- --no-session
//!     cargo run -p agents --bin agents-probe -- --only opencode
//!     cargo run -p agents --bin agents-probe -- --prompt "what is 2+2?"

use std::io::Write;
use std::process::ExitCode;

use agents::{AgentCLI, SessionEvent, SessionRequest};

struct Args {
    no_session: bool,
    only: Option<String>,
    prompt: String,
}

fn parse_args() -> Args {
    let mut args = Args {
        no_session: false,
        only: None,
        prompt: "Reply with exactly the words: probe ok.".to_string(),
    };
    let mut iter = std::env::args().skip(1);
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--no-session" => args.no_session = true,
            "--only" => args.only = iter.next(),
            "--prompt" => {
                if let Some(p) = iter.next() {
                    args.prompt = p;
                }
            }
            "-h" | "--help" => {
                println!("agents-probe [--no-session] [--only NAME] [--prompt TEXT]");
                std::process::exit(0);
            }
            _ => {
                eprintln!("unknown argument: {arg}");
                std::process::exit(2);
            }
        }
    }
    args
}

fn main() -> ExitCode {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_target(false)
        .init();
    let args = parse_args();

    let mut any_failed = false;
    for adapter in agents::all() {
        let kind = adapter.kind();
        if let Some(only) = &args.only {
            if only != kind.as_str() {
                continue;
            }
        }
        section(&format!("== {} ==", kind));

        let info = match adapter.detect() {
            Ok(Some(info)) => info,
            Ok(None) => {
                println!("  (not installed on PATH; skipping)");
                continue;
            }
            Err(err) => {
                println!("  detect failed: {err}");
                any_failed = true;
                continue;
            }
        };
        println!("  binary:  {}", info.executable.display());
        println!("  version: {}", info.version);

        match adapter.list_models() {
            Ok(models) => {
                println!("  models ({}):", models.len());
                for m in models.iter().take(10) {
                    let alias = m
                        .alias
                        .as_deref()
                        .map(|a| format!(" (alias: {a})"))
                        .unwrap_or_default();
                    println!("    - {} [{}]{alias}", m.id, m.provider);
                }
                if models.len() > 10 {
                    println!("    ... {} more", models.len() - 10);
                }
            }
            Err(err) => {
                println!("  list_models failed: {err}");
                any_failed = true;
            }
        }

        let efforts = adapter.reasoning_efforts();
        let pretty: Vec<_> = efforts.iter().map(|e| e.to_string()).collect();
        println!("  thinking levels: {}", pretty.join(", "));

        if args.no_session {
            continue;
        }

        if let Err(err) = run_probe_session(&*adapter, &args.prompt) {
            println!("  session failed: {err}");
            any_failed = true;
        }
    }

    if any_failed {
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    }
}

fn section(title: &str) {
    println!();
    println!("{title}");
}

fn run_probe_session(adapter: &dyn AgentCLI, prompt: &str) -> agents::Result<()> {
    println!("  session: sending probe prompt …");
    let req = SessionRequest::new(prompt);
    let mut text_chars = 0usize;
    let mut reasoning_chars = 0usize;
    let mut tool_calls = 0usize;
    let mut session_id = None;
    let stdout = std::io::stdout();
    let mut handle = stdout.lock();

    let mut on_event = |event: &SessionEvent| match event {
        SessionEvent::SessionStarted { session_id: id } => {
            session_id = Some(id.clone());
            let _ = writeln!(handle, "    [session] {id}");
        }
        SessionEvent::AssistantText { text } => {
            text_chars += text.len();
            let _ = write!(handle, "{text}");
            let _ = handle.flush();
        }
        SessionEvent::Reasoning { text } => {
            reasoning_chars += text.len();
        }
        SessionEvent::ToolUse { name, .. } => {
            tool_calls += 1;
            let _ = writeln!(handle, "\n    [tool] {name}");
        }
        SessionEvent::Done { .. } | SessionEvent::Raw { .. } => {}
    };

    let result = adapter.run_session(&req, &mut on_event)?;
    println!();
    println!(
        "  done: text={}B reasoning={}B tool_calls={} session_id={}",
        text_chars,
        reasoning_chars,
        tool_calls,
        session_id
            .as_deref()
            .or(result.session_id.as_deref())
            .unwrap_or("<none>")
    );
    if let Some(text) = result.final_text.as_deref() {
        let preview = text.chars().take(120).collect::<String>();
        println!("  final:  {preview:?}");
    }
    Ok(())
}
