use std::path::PathBuf;

use sem_core::config::Config;
use sem_core::ipc::socket_path;

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    println!("Semaphore doctor\n");

    let config_dir = Config::config_dir();
    println!(
        "[{}] config dir: {}",
        status(config_dir.exists()),
        config_dir.display()
    );

    let socket = socket_path();
    #[cfg(unix)]
    let socket_ok = socket.exists();
    #[cfg(windows)]
    let socket_ok = std::path::Path::new(r"\\.\pipe\semaphore").exists()
        || true; // pipe may not exist until app runs

    println!(
        "[{}] ipc socket: {}",
        if socket_ok { "ok" } else { "waiting" },
        socket.display()
    );

    let bin_dir = Config::bin_dir();
    let sem_hook = bin_dir.join(if cfg!(windows) { "sem-hook.bat" } else { "sem-hook" });
    let semctl_bin = bin_dir.join(if cfg!(windows) { "semctl.exe" } else { "semctl" });
    println!(
        "[{}] sem-hook: {}",
        status(sem_hook.exists()),
        sem_hook.display()
    );
    println!(
        "[{}] semctl: {}",
        status(semctl_bin.exists()),
        semctl_bin.display()
    );

    for (tool, path) in tool_config_paths() {
        let installed = path.exists() && hook_file_contains(&path);
        println!("[{}] hooks ({tool}): {}", status(installed), path.display());
    }

    Ok(())
}

fn tool_config_paths() -> Vec<(&'static str, PathBuf)> {
    let home = dirs_home();
    vec![
        ("cursor", home.join(".cursor/hooks.json")),
        ("claude-code", home.join(".claude/settings.json")),
        ("codex", home.join(".codex/hooks.json")),
        ("gemini-cli", home.join(".gemini/settings.json")),
        ("copilot-cli", home.join(".copilot/hooks.json")),
    ]
}

fn hook_file_contains(path: &std::path::Path) -> bool {
    std::fs::read_to_string(path)
        .map(|s| s.contains("_semaphore"))
        .unwrap_or(false)
}

fn dirs_home() -> PathBuf {
    if let Ok(home) = std::env::var("HOME") {
        return home.into();
    }
    if let Ok(profile) = std::env::var("USERPROFILE") {
        return profile.into();
    }
    ".".into()
}

fn status(ok: bool) -> &'static str {
    if ok { "ok" } else { "missing" }
}
