use std::{hash::{DefaultHasher, Hash, Hasher}, iter::Map};

#[allow(unused_imports)] use bevy::prelude::*;
use bevy_ecs_tilemap::tiles::TileFlip;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
#[allow(unused_imports)] use bevy_asset_loader::prelude::*;
use crate::game::tilemap::{terrain_gen::terrgen_resources::WorldGenSettings, tile::{
    tile_components::*, tile_constants::*, tile_resources::*, 
}};

#[allow(unused_parens)]
pub fn init_shaders(
    mut cmd: Commands, asset_server: Res<AssetServer>, 
    mut repeat_tex_handles: ResMut<ShaderRepeatTexSerisHandles>,
    mut assets: ResMut<Assets<ShaderRepeatTexSeri>>,
    mut map: ResMut<TileShaderEntityMap>,
) {
    for handle in std::mem::take(&mut repeat_tex_handles.handles) {
        info!(target: "tiling_loading", "Loading TileSeri from handle: {:?}", handle);
        map.new_repeat_tex_shader(&mut cmd, &asset_server, handle, &mut assets);
    }
} 

#[allow(unused_parens)]
pub fn init_tiles(
    mut cmd: Commands, 
    asset_server: Res<AssetServer>,
    mut seris_handles: ResMut<TileSerisHandles>,
    mut assets: ResMut<Assets<TileSeri>>,
    mut map: ResMut<TilingEntityMap>,
    shader_map: Res<TileShaderEntityMap>,
) {
    for handle in std::mem::take(&mut seris_handles.handles) {
        info!(target: "tiling_loading", "Loading TileSeri from handle: {:?}", handle);
        map.new_tile_ent_from_seri(&mut cmd, &asset_server, handle, &mut assets, &shader_map);
    }
} 

#[allow(unused_parens)]
pub fn init_tile_weighted_samplers(
    mut cmd: Commands, 
    mut seris_handles: ResMut<TileWeightedSamplerSerisHandles>,
    mut assets: ResMut<Assets<TileWeightedSamplerSeri>>,
    mut map: ResMut<TilingEntityMap>,
) {
    for handle in std::mem::take(&mut seris_handles.handles) {
        info!(target: "tiling_loading", "Loading TileWeightedSamplerSeri from handle: {:?}", handle);
        map.new_weighted_tilesampler_ent_from_seri(&mut cmd, handle, &mut assets);
    }

    //info!(target: "tiling_loading", "TilingEntityMap contents:"); for (id, ent) in map.0.iter() { info!(target: "tiling_loading", "  - id: {}, entity: {:?}", id, ent); }
} 


#[allow(unused_parens)]
pub fn update_tile_hash_value(
    settings: Res<WorldGenSettings>,
    mut query: Query<(&GlobalTilePos, &mut TileposHashRand),(Added<GlobalTilePos>)>) {
    for (pos, mut hash) in query.iter_mut() {
        let mut hasher = DefaultHasher::new();
        pos.hash(&mut hasher);
        settings.seed.hash(&mut hasher);
        hash.0 = (hasher.finish() as f64 / u64::MAX as f64) as f32;
    }
}

#[allow(unused_parens)]
pub fn update_tile_name(mut query: Query<(&mut Name, &GlobalTilePos),(Changed<GlobalTilePos>)>) {
    for (mut name, pos) in query.iter_mut() {
        let prev_name = name.as_str().split(GlobalTilePos::TYPE_DEBUG_NAME).next().unwrap_or("Tile").to_string();
        name.set(format!("{} {:?}", prev_name, pos));
    }
}

#[allow(unused_parens)]
pub fn flip_tile_along_x(mut query: Query<(&mut TileFlip, &TileposHashRand),(With<FlipAlongX>, Changed<TileposHashRand>)>) {
    for (mut flip, &TileposHashRand(val)) in query.iter_mut() {
        if val > 0.5 { flip.x = !flip.x; }
    }
}
