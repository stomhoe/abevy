use std::{fs, path::Path};

use bevy::prelude::*;
use bevy_replicon_renet::{RenetClient, RenetServer};
use common::{log_targets::SPRITE_INIT, common_components::SettingsEntity};
use tilemap_shared::ZSettings;

pub fn load_y_sort_settings(
    mut commands: Commands,
    settings_query: Query<Entity, With<SettingsEntity>>,
    server: Option<Res<RenetServer>>,
    client: Option<Res<RenetClient>>,
) {
    let path = Path::new("assets/ron/settings/z.settings.ron");
    let Ok(contents) = fs::read_to_string(path) else {
        return;
    };
    let Ok(loaded) = ron::from_str::<ZSettings>(&contents) else {
        error!(target: SPRITE_INIT, "Failed parsing '{}'", path.display());
        return;
    };

    if let Ok(settings_entity) = settings_query.single() {
        commands.entity(settings_entity).insert(loaded);
    } else if server.is_some() || client.is_none() {
        commands.spawn((SettingsEntity, loaded,));
    }
}