use bevy::prelude::*;

use crate::tile::tile_shader::tile_material::terrbl::TerrBlendMat;

pub fn update_wavy_time(time: Res<Time>, mut mats: ResMut<Assets<TerrBlendMat>>) {
    let t = time.elapsed_secs();
    for (_h, mat) in mats.iter_mut() {
        mat.time = t;
    }
}
