use std::fs;
use std::path::{Path, PathBuf};

use sem_core::config::Config;

const MARKER: &str = "_semaphore";

pub fn run_install(all: bool, tool: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
    ensure_binaries()?;

    let tools: Vec<&str> = if all {
        vec!["cursor", "claude-code"]
    } else {
        vec![tool.ok_or("specify a tool or use --all")?]
    };

    for tool in tools {
        match tool {
            "cursor" => install_cursor()?,
            "claude-code" => install_claude_code()?,
            other => return Err(format!("unknown tool: {other}").into()),
        }
        println!("installed hooks for {tool}");
    }
    Ok(())
}

pub fn run_uninstall(all: bool, tool: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
    let tools: Vec<&str> = if all {
        vec!["cursor", "claude-code"]
    } else {
        vec![tool.ok_or("specify a tool or use --all")?]
    };

    for tool in tools {
        match tool {
            "cursor" => uninstall_cursor()?,
            "claude-code" => uninstall_claude_code()?,
            other => return Err(format!("unknown tool: {other}").into()),
        }
        println!("removed hooks for {tool}");
    }
    Ok(())
}

fn ensure_binaries() -> Result<(), Box<dyn std::error::Error>> {
    let bin_dir = Config::bin_dir();
    fs::create_dir_all(&bin_dir)?;

    let sem_hook_path = bin_dir.join(if cfg!(windows) { "sem-hook.bat" } else { "sem-hook" });
    if !sem_hook_path.exists() {
        write_sem_hook(&sem_hook_path)?;
    }
    Ok(())
}

fn write_sem_hook(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(unix)]
    {
        let script = r#"#!/usr/bin/env bash
set -euo pipefail
STATE="${1:-yellow}"
REASON="${2:-event}"
SOURCE="${3:-hook}"
SESSION="default"
if [ ! -t 0 ]; then
  INPUT="$(cat)"
  PARSED="$(printf '%s' "$INPUT" | python3 -c 'import json,sys
data=json.load(sys.stdin)
print(data.get("session_id") or data.get("conversation_id") or data.get("sessionId") or "default")' 2>/dev/null || echo default)"
  SESSION="$PARSED"
fi
SEMCTL="${SEMAPHORE_BIN:-$HOME/.semaphore/bin/semctl}"
if [ -x "$SEMCTL" ]; then
  "$SEMCTL" set "$STATE" --session "$SESSION" --source "$SOURCE" --reason "$REASON" >/dev/null 2>&1 || true
fi
exit 0
"#;
        fs::write(path, script)?;
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(path)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(path, perms)?;
    }

    #[cfg(windows)]
    {
        let script = r#"@echo off
setlocal
set STATE=%~1
if "%STATE%"=="" set STATE=yellow
set REASON=%~2
if "%REASON%"=="" set REASON=event
set SOURCE=%~3
if "%SOURCE%"=="" set SOURCE=hook
set SESSION=default
set SEMCTL=%USERPROFILE%\.semaphore\bin\semctl.exe
if exist "%SEMCTL%" (
  "%SEMCTL%" set %STATE% --session %SESSION% --source %SOURCE% --reason %REASON% >nul 2>&1
)
exit /b 0
"#;
        fs::write(path, script)?;
    }
    Ok(())
}

fn hook_command(state: &str, reason: &str) -> String {
    let hook = Config::bin_dir().join(if cfg!(windows) {
        "sem-hook.bat"
    } else {
        "sem-hook"
    });
    format!("{} {} {}", hook.display(), state, reason)
}

fn install_cursor() -> Result<(), Box<dyn std::error::Error>> {
    let path = home_dir().join(".cursor/hooks.json");
    merge_cursor_hooks(&path)
}

fn uninstall_cursor() -> Result<(), Box<dyn std::error::Error>> {
    let path = home_dir().join(".cursor/hooks.json");
    remove_marked_hooks(&path, "hooks")
}

fn install_claude_code() -> Result<(), Box<dyn std::error::Error>> {
    let path = home_dir().join(".claude/settings.json");
    merge_claude_hooks(&path)
}

fn uninstall_claude_code() -> Result<(), Box<dyn std::error::Error>> {
    let path = home_dir().join(".claude/settings.json");
    remove_marked_hooks(&path, "hooks")
}

fn merge_cursor_hooks(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir_all(path.parent().unwrap())?;
    let mut root: serde_json::Value = if path.exists() {
        serde_json::from_str(&fs::read_to_string(path)?)?
    } else {
        serde_json::json!({ "version": 1, "hooks": {} })
    };

    let hooks = root
        .as_object_mut()
        .and_then(|o| o.get_mut("hooks"))
        .and_then(|v| v.as_object_mut())
        .ok_or("invalid cursor hooks.json structure")?;

    insert_hook(hooks, "beforeSubmitPrompt", "yellow", "thinking", None);
    insert_hook(hooks, "afterAgentThought", "yellow", "thinking", None);
    insert_hook(
        hooks,
        "preToolUse",
        "red",
        "writing",
        Some("Write|Edit|Shell"),
    );
    insert_hook(hooks, "afterFileEdit", "red", "writing", None);
    insert_hook(hooks, "stop", "green", "idle", None);
    insert_hook(hooks, "sessionEnd", "green", "idle", None);

    fs::write(path, serde_json::to_string_pretty(&root)?)?;
    Ok(())
}

fn merge_claude_hooks(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir_all(path.parent().unwrap())?;
    let mut root: serde_json::Value = if path.exists() {
        serde_json::from_str(&fs::read_to_string(path)?)?
    } else {
        serde_json::json!({ "hooks": {} })
    };

    let hooks = root
        .as_object_mut()
        .and_then(|o| o.get_mut("hooks"))
        .and_then(|v| v.as_object_mut())
        .ok_or("invalid claude settings.json structure")?;

    insert_claude_hook(hooks, "UserPromptSubmit", "yellow", "thinking", "");
    insert_claude_hook(hooks, "PreToolUse", "red", "writing", "Write|Edit|Bash");
    insert_claude_hook(hooks, "PostToolUse", "yellow", "thinking", "");
    insert_claude_hook(hooks, "Stop", "green", "idle", "");
    insert_claude_hook(hooks, "SessionEnd", "green", "idle", "");

    fs::write(path, serde_json::to_string_pretty(&root)?)?;
    Ok(())
}

fn insert_hook(
    hooks: &mut serde_json::Map<String, serde_json::Value>,
    event: &str,
    state: &str,
    reason: &str,
    matcher: Option<&str>,
) {
    let mut entry = serde_json::json!({
        "command": hook_command(state, reason),
        "_semaphore": true
    });
    if let Some(m) = matcher {
        entry["matcher"] = serde_json::Value::String(m.to_string());
    }
    let list = hooks.entry(event.to_string()).or_insert_with(|| serde_json::json!([]));
    if let Some(arr) = list.as_array_mut() {
        if !arr.iter().any(|v| v.get(MARKER) == Some(&serde_json::Value::Bool(true))) {
            arr.push(entry);
        }
    }
}

fn insert_claude_hook(
    hooks: &mut serde_json::Map<String, serde_json::Value>,
    event: &str,
    state: &str,
    reason: &str,
    matcher: &str,
) {
    let hook_entry = serde_json::json!({
        "type": "command",
        "command": hook_command(state, reason),
        "_semaphore": true
    });

    let event_list = hooks
        .entry(event.to_string())
        .or_insert_with(|| serde_json::json!([]));

    if matcher.is_empty() {
        if let Some(arr) = event_list.as_array_mut() {
            if !arr.iter().any(|v| {
                v.get("hooks")
                    .and_then(|h| h.as_array())
                    .map(|hooks| {
                        hooks.iter().any(|h| {
                            h.get(MARKER) == Some(&serde_json::Value::Bool(true))
                        })
                    })
                    .unwrap_or(false)
            }) {
                arr.push(serde_json::json!({
                    "hooks": [hook_entry]
                }));
            }
        }
        return;
    }

    if let Some(arr) = event_list.as_array_mut() {
        let exists = arr.iter().any(|block| {
            block.get(MARKER) == Some(&serde_json::Value::Bool(true))
                || block
                    .get("hooks")
                    .and_then(|h| h.as_array())
                    .map(|hooks| {
                        hooks.iter().any(|h| {
                            h.get(MARKER) == Some(&serde_json::Value::Bool(true))
                        })
                    })
                    .unwrap_or(false)
        });
        if !exists {
            arr.push(serde_json::json!({
                "matcher": matcher,
                "hooks": [hook_entry],
                "_semaphore": true
            }));
        }
    }
}

fn remove_marked_hooks(path: &Path, key: &str) -> Result<(), Box<dyn std::error::Error>> {
    if !path.exists() {
        return Ok(());
    }
    let mut root: serde_json::Value = serde_json::from_str(&fs::read_to_string(path)?)?;
    let Some(hooks) = root.get_mut(key).and_then(|v| v.as_object_mut()) else {
        return Ok(());
    };

    for (_event, value) in hooks.clone().iter() {
        // handled per event below
        let _ = value;
    }

    let events: Vec<String> = hooks.keys().cloned().collect();
    for event in events {
        let Some(value) = hooks.get_mut(&event) else { continue };
        if let Some(arr) = value.as_array_mut() {
            arr.retain(|entry| entry.get(MARKER) != Some(&serde_json::Value::Bool(true)));
            arr.retain(|entry| {
                !entry
                    .get("hooks")
                    .and_then(|h| h.as_array())
                    .map(|hooks| {
                        hooks.iter().any(|h| {
                            h.get(MARKER) == Some(&serde_json::Value::Bool(true))
                        })
                    })
                    .unwrap_or(false)
            });
            for entry in arr.iter_mut() {
                if let Some(inner) = entry.get_mut("hooks").and_then(|v| v.as_array_mut()) {
                    inner.retain(|h| h.get(MARKER) != Some(&serde_json::Value::Bool(true)));
                }
            }
            if arr.is_empty() {
                hooks.remove(&event);
            }
        }
    }

    fs::write(path, serde_json::to_string_pretty(&root)?)?;
    Ok(())
}

fn home_dir() -> PathBuf {
    if let Ok(home) = std::env::var("HOME") {
        return PathBuf::from(home);
    }
    if let Ok(profile) = std::env::var("USERPROFILE") {
        return PathBuf::from(profile);
    }
    PathBuf::from(".")
}
