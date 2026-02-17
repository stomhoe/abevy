use bevy::prelude::*;
use common::common_components::{ImagePathHolder, MultipleImagePathHolder, };

use crate::tile::tile_shader::{tile_material::wavy::WavyMat, tile_shader_components::*, };

#[allow(unused_parens)]
pub fn add_image_handle_to_tile_shader(
    asset_server: Res<AssetServer>,
    mut query: Query<(&mut TileShader, AnyOf<(&ImagePathHolder, &MultipleImagePathHolder)>),(Or<(Changed<ImagePathHolder>, Changed<MultipleImagePathHolder>)>,)>,
) {
    query.iter_mut().for_each(|(mut tile_shader, (img_path, multiple_img_path))| {
        if let Some(img_path) = img_path {
            let image_handle = asset_server.load(img_path.path());
            tile_shader.set_image_handle(image_handle);
        } else if let Some(multiple_img_path) = multiple_img_path {
            let paths = multiple_img_path.paths();
            let _handles: Vec<Handle<Image>> = paths.iter().map(|path| asset_server.load(path)).collect();
            todo!("Implement multiple image handling");
            //tile_shader.set_multiple_image_handles(handles);

        }
    });
}

pub fn update_wavy_time(time: Res<Time>, mut mats: ResMut<Assets<WavyMat>>) {
    let t = time.elapsed_secs();
    for (_h, mat) in mats.iter_mut() {
        mat.time = t as f32;
    }
}
