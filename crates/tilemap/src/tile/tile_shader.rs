use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::MaterialTilemapPlugin;
use bevy_replicon::prelude::*;
use common::{common_states::AssetLoading};
use ::tilemap_shared::*;

#[allow(unused_imports, )]
use crate::tile::tile_shader::{tile_material::*, tile_shader_components::*, tile_shader_init_systems::*, tile_shader_resources::*, tile_shader_systems::*};


// Bring material types into scope for this file

pub mod tile_material;
mod tile_shader_init_systems;
mod tile_shader_systems;
pub mod tile_shader_components;
pub mod tile_shader_resources;
pub mod tile_shader_seris;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct TileShaderSystems;


pub fn plugin(app: &mut App) {
    app
    .add_systems(OnEnter(AssetLoading::SpawnReplicatedEntities), (
        (init_shaders, map_tile_shader_id_to_entity).chain()
    ).in_set(TileShaderSystems))
    .add_systems(Update, (
        resolve_terrbl_texture_handles,
        update_wavy_time,

    ).in_set(TileShaderSystems))
    .add_plugins((
        plugin_tile_shader,
        MaterialTilemapPlugin::<TerrBlendMat>::default(),
    ))
    .replicate::<TerrBlendParams>()
    ;
}
