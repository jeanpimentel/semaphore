use std::path::Path;

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
    let socket_ok = true;

    println!(
        "[{}] ipc socket: {}",
        status(socket_ok),
        socket.display()
    );

    let bin_dir = Config::bin_dir();
    let sem_hook = bin_dir.join(if cfg!(windows) { "sem-hook.exe" } else { "sem-hook" });
    println!(
        "[{}] sem-hook: {}",
        status(sem_hook.exists()),
        sem_hook.display()
    );

    for tool in ["cursor", "claude-code"] {
        let installed = check_tool_hooks(tool);
        println!("[{}] hooks: {tool}", status(installed));
    }

    Ok(())
}

fn check_tool_hooks(tool: &str) -> bool {
    match tool {
        "cursor" => hook_file_contains(&dirs_home().join(".cursor/hooks.json")),
        "claude-code" => hook_file_contains(&dirs_home().join(".claude/settings.json")),
        _ => false,
    }
}

fn hook_file_contains(path: &Path) -> bool {
    path.exists()
        && std::fs::read_to_string(path)
            .map(|s| s.contains("_semaphore"))
            .unwrap_or(false)
}

fn dirs_home() -> std::path::PathBuf {
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
