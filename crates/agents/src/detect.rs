use std::path::PathBuf;
use std::process::Command;

use crate::types::{AgentInfo, AgentKind, Error, Result};

/// Run `binary <args...>` capturing stdout. Returns the trimmed stdout on
/// success. Stderr is folded into the error so callers can debug failed
/// invocations.
pub(crate) fn run_capture(binary: &str, args: &[&str]) -> Result<String> {
    let output = Command::new(binary).args(args).output().map_err(|source| {
        if source.kind() == std::io::ErrorKind::NotFound {
            Error::Spawn {
                cmd: binary.to_string(),
                source,
            }
        } else {
            Error::Spawn {
                cmd: binary.to_string(),
                source,
            }
        }
    })?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
        return Err(Error::NonZeroExit {
            cmd: format!("{binary} {}", args.join(" ")),
            status: output.status.code().unwrap_or(-1),
            stderr,
        });
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

/// Locate `binary` on PATH. Returns `None` if it can't be found.
pub(crate) fn locate(binary: &str) -> Option<PathBuf> {
    which::which(binary).ok()
}

/// Quick helper for adapters: detect a binary, run a `--version`-like command,
/// and produce an [`AgentInfo`]. The caller hands us the args used to print
/// the version string and a closure that extracts the version from stdout.
pub(crate) fn detect_with(
    kind: AgentKind,
    binary: &str,
    version_args: &[&str],
    parse_version: impl FnOnce(&str) -> String,
) -> Result<Option<AgentInfo>> {
    let Some(executable) = locate(binary) else {
        return Ok(None);
    };
    let stdout = run_capture(binary, version_args)?;
    Ok(Some(AgentInfo {
        kind,
        executable,
        version: parse_version(&stdout),
    }))
}

/// Detect every known agent in one shot, dropping the ones that aren't
/// installed.
pub fn detect_all() -> Vec<AgentInfo> {
    let mut out = Vec::new();
    for adapter in crate::all() {
        match adapter.detect() {
            Ok(Some(info)) => out.push(info),
            Ok(None) => {}
            Err(err) => log::warn!("detect {} failed: {err}", adapter.kind()),
        }
    }
    out
}
