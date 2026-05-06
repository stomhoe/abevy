use std::{fs, path::PathBuf};

pub fn trim_world_texture_sprite_id(path: &str) -> String {
    let trimmed = path.strip_prefix("texture/world/").unwrap_or(path).trim_matches('/');
    if trimmed.is_empty() {
        return String::new();
    }

    let mut segments = trimmed.rsplit('/');
    let file_name = segments.next().unwrap_or(trimmed);
    let containing_dir = segments.next();
    if let Some(containing_dir) = containing_dir {
        return format!("{containing_dir}/{file_name}");
    }
    file_name.to_string()
}

pub fn gather_step_sfx_paths_from_dir(directory: &str) -> Vec<String> {
    let directory = directory.trim().trim_matches('/');
    if directory.is_empty() {
        return Vec::new();
    }

    let mut paths = Vec::new();
    let mut stack = vec![PathBuf::from("assets").join(directory)];
    while let Some(curr) = stack.pop() {
        let Ok(entries) = fs::read_dir(&curr) else { continue };
        for entry in entries.flatten() {
            let Ok(file_type) = entry.file_type() else { continue };
            let path = entry.path();
            if file_type.is_dir() {
                stack.push(path);
                continue;
            }
            if !file_type.is_file() {
                continue;
            }
            let Some(ext) = path.extension().and_then(|ext| ext.to_str()) else { continue };
            if !matches!(ext.to_ascii_lowercase().as_str(), "wav" | "ogg" | "mp3" | "flac") {
                continue;
            }
            let Ok(asset_rel) = path.strip_prefix("assets") else { continue };
            let Some(asset_rel) = asset_rel.to_str() else { continue };
            paths.push(asset_rel.replace('\\', "/"));
        }
    }
    paths.sort();
    paths
}
