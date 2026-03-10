use std::collections::HashMap;
use std::fs::{OpenOptions, create_dir_all};
use std::io::Write;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

use serde::Deserialize;

struct DriftFileLogger {
    enabled: bool,
    files: HashMap<String, std::fs::File>,
}

static DRIFT_FILE_LOGGER: OnceLock<Mutex<DriftFileLogger>> = OnceLock::new();

#[derive(Deserialize)]
struct DriftLogSettingsSeri {
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
    let Ok(parsed) = ron::from_str::<DriftLogSettingsSeri>(&text) else {
        return false;
    };
    parsed.drift_file_logging
}

fn make_log_path(role: &str) -> Option<PathBuf> {
    let mut dir = std::env::current_dir().ok()?;
    dir.push("logs");
    dir.push("drift");
    create_dir_all(&dir).ok()?;
    let pid = std::process::id();
    dir.push(format!("{role}_pid{pid}.log"));
    Some(dir)
}

pub fn drift_log(role: &str, line: &str) {
    let logger = DRIFT_FILE_LOGGER.get_or_init(|| {
        Mutex::new(DriftFileLogger {
            enabled: settings_flag_enabled(),
            files: HashMap::default(),
        })
    });
    let Ok(mut guard) = logger.lock() else { return; };
    if !guard.enabled {
        return;
    }
    if !guard.files.contains_key(role) {
        let Some(path) = make_log_path(role) else { return; };
        let Ok(file) = OpenOptions::new().create(true).append(true).open(path) else { return; };
        guard.files.insert(role.to_string(), file);
    }
    let Some(file) = guard.files.get_mut(role) else { return; };
    let _ = writeln!(file, "{line}");
}
