use std::{fs, path::Path};

use bevy::prelude::*;
use common::log_targets::SPRITE_INIT;
use tilemap_shared::ZSettings;

pub fn load_y_sort_settings(mut settings: ResMut<ZSettings>) {
    let path = Path::new("assets/ron/settings/z.settings.ron");
    let Ok(contents) = fs::read_to_string(path) else {
        return;
    };
    let Ok(loaded) = ron::from_str::<ZSettings>(&contents) else {
        error!(target: SPRITE_INIT, "Failed parsing '{}'", path.display());
        return;
    };
    *settings = loaded;
}