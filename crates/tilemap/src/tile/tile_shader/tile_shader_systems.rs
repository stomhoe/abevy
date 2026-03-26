use bevy::prelude::*;
use common::AnyDisabling;

use ::tilemap_shared::*;

pub fn update_wavy_time(time: Res<Time>, mut mats: ResMut<Assets<TerrBlendMat>>) {
    let t = time.elapsed_secs();
    for (_h, mat) in mats.iter_mut() {
        mat.time = t;
    }
}

pub fn resolve_terrbl_texture_handles(
    asset_server: Res<AssetServer>,
    mut terrbl_query: Query<&mut TerrBlendParams, (Changed<TerrBlendParams>, AnyDisabling)>,
) {
    for mut params in terrbl_query.iter_mut() {
        if params.texture_path.to_string().is_empty() {
            if params.texture_handle != Handle::default() {
                params.texture_handle = Handle::default();
            }
            continue;
        }
        let next_handle: Handle<Image> = asset_server.load(&params.texture_path);
        if params.texture_handle.id() != next_handle.id() {
            params.texture_handle = next_handle;
        }
    }
}
