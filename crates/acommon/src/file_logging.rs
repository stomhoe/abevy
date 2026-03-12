use std::collections::HashMap;
use std::fs::{OpenOptions, create_dir_all};
use std::io::Write;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

use serde::Deserialize;

struct FileLogger {
    enabled: bool,
    files: HashMap<(String, String), std::fs::File>,
}

static FILE_LOGGER: OnceLock<Mutex<FileLogger>> = OnceLock::new();

#[derive(Deserialize)]
struct FileLogSettingsSeri {
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
    let Ok(parsed) = ron::from_str::<FileLogSettingsSeri>(&text) else {
        return false;
    };
    parsed.drift_file_logging
}

fn make_log_path(category: &str, role: &str) -> Option<PathBuf> {
    let mut dir = std::env::current_dir().ok()?;
    dir.push("logs");
    dir.push(category);
    create_dir_all(&dir).ok()?;
    let pid = std::process::id();
    dir.push(format!("{role}_pid{pid}.log"));
    Some(dir)
}

fn clear_old_role_logs(category: &str, role: &str) {
    let Ok(mut dir) = std::env::current_dir() else {
        return;
    };
    dir.push("logs");
    dir.push(category);
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

pub fn file_log(category: &str, role: &str, line: &str) {
    let logger = FILE_LOGGER.get_or_init(|| {
        Mutex::new(FileLogger {
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
    let key = (category.to_string(), role.to_string());
    if !guard.files.contains_key(&key) {
        clear_old_role_logs(category, role);
        let Some(path) = make_log_path(category, role) else {
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
        guard.files.insert(key.clone(), file);
    }
    let Some(file) = guard.files.get_mut(&key) else {
        return;
    };
    let _ = writeln!(file, "{line}");
}
