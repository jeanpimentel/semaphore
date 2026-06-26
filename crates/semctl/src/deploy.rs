use std::fs;
use std::path::{Path, PathBuf};

use sem_core::config::Config;

/// Copy semctl into ~/.semaphore/bin when available (current exe, bundle, or build output).
pub fn deploy_semctl() -> Result<(), Box<dyn std::error::Error>> {
    let dest = Config::bin_dir().join(if cfg!(windows) {
        "semctl.exe"
    } else {
        "semctl"
    });
    fs::create_dir_all(Config::bin_dir())?;

    if dest.exists() {
        return Ok(());
    }

    if let Some(source) = locate_semctl_binary() {
        fs::copy(&source, &dest)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&dest)?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&dest, perms)?;
        }
    }

    Ok(())
}

fn locate_semctl_binary() -> Option<PathBuf> {
    let current = std::env::current_exe().ok()?;
    if is_semctl_name(current.file_name()?.to_str()?) {
        return Some(current);
    }

    if let Ok(path) = std::env::var("SEMAPHORE_SEMCTL") {
        let p = PathBuf::from(path);
        if p.exists() {
            return Some(p);
        }
    }

    for candidate in candidate_paths(&current) {
        if candidate.exists() {
            return Some(candidate);
        }
    }

    None
}

fn is_semctl_name(name: &str) -> bool {
    name == "semctl" || name == "semctl.exe"
}

fn candidate_paths(current_exe: &Path) -> Vec<PathBuf> {
    let mut paths = Vec::new();

    if let Some(parent) = current_exe.parent() {
        paths.push(parent.join("semctl"));
        paths.push(parent.join("semctl.exe"));
        paths.push(parent.join("bin").join("semctl"));
        paths.push(parent.join("bin").join("semctl.exe"));
        // macOS .app bundle Resources
        paths.push(parent.join("../Resources/semctl"));
    }

    paths.push(PathBuf::from("target/debug/semctl"));
    paths.push(PathBuf::from("target/release/semctl"));
    paths.push(PathBuf::from("target/debug/semctl.exe"));
    paths.push(PathBuf::from("target/release/semctl.exe"));

    paths
}

pub fn deploy_from_resource(resource_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let name = if cfg!(windows) { "semctl.exe" } else { "semctl" };
    let source = resource_dir.join(name);
    if !source.exists() {
        return deploy_semctl();
    }
    let dest = Config::bin_dir().join(name);
    fs::create_dir_all(Config::bin_dir())?;
    fs::copy(&source, &dest)?;
    Ok(())
}
