use std::collections::HashMap;
use std::fs::{OpenOptions, create_dir_all};
use std::io::Write;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

use serde::Deserialize;

struct MoveFileLogger {
    enabled: bool,
    files: HashMap<String, std::fs::File>,
}

static MOVE_FILE_LOGGER: OnceLock<Mutex<MoveFileLogger>> = OnceLock::new();

#[derive(Deserialize)]
struct MoveLogSettingsSeri {
    #[serde(default)]
    drift_file_logging: bool,
}

fn settings_flag_enabled() -> bool {
    let Ok(cwd) = std::env::current_dir() else {
        return false;
    };
    let path = cwd
        .join("assets")
        .join("ron")
        .join("settings")
        .join("debug_ui.settings.ron");
    let Ok(text) = std::fs::read_to_string(path) else {
        return false;
    };
    let Ok(parsed) = ron::from_str::<MoveLogSettingsSeri>(&text) else {
        return false;
    };
    parsed.drift_file_logging
}

fn make_log_path(role: &str) -> Option<PathBuf> {
    let mut dir = std::env::current_dir().ok()?;
    dir.push("logs");
    dir.push("move");
    create_dir_all(&dir).ok()?;
    let pid = std::process::id();
    dir.push(format!("{role}_pid{pid}.log"));
    Some(dir)
}

fn clear_old_role_logs(role: &str) {
    let Ok(mut dir) = std::env::current_dir() else {
        return;
    };
    dir.push("logs");
    dir.push("move");
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    let prefix = format!("{role}_pid");
    for entry in entries.flatten() {
        let path = entry.path();
        let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        if name.starts_with(&prefix) && name.ends_with(".log") {
            let _ = std::fs::remove_file(path);
        }
    }
}

pub fn movement_log(role: &str, line: &str) {
    let logger = MOVE_FILE_LOGGER.get_or_init(|| {
        Mutex::new(MoveFileLogger {
            enabled: settings_flag_enabled(),
            files: HashMap::default(),
        })
    });
    let Ok(mut guard) = logger.lock() else {
        return;
    };
    if !guard.enabled {
        return;
    }
    if !guard.files.contains_key(role) {
        clear_old_role_logs(role);
        let Some(path) = make_log_path(role) else {
            return;
        };
        let Ok(file) = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(path)
        else {
            return;
        };
        guard.files.insert(role.to_string(), file);
    }
    let Some(file) = guard.files.get_mut(role) else {
        return;
    };
    let _ = writeln!(file, "{line}");
}
