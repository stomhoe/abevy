use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::MaterialTilemapPlugin;
use bevy_replicon::prelude::*;
use common::{common_states::AssetLoading};

use crate::tile::tile_shader::{tile_material::prelude::*, tile_shader_components::*, tile_shader_init_systems::*, tile_shader_resources::*, tile_shader_systems::*};


// Bring material types into scope for this file

pub mod tile_material;
mod tile_shader_init_systems;
mod tile_shader_systems;
pub mod tile_shader_components;
pub mod tile_shader_resources;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct TileShaderSystems;


pub fn plugin(app: &mut App) {
    app
    .add_systems(OnEnter(AssetLoading::SpawnReplicatedEntities), (
        (init_shaders, map_tile_shader_id_to_entity).chain()
    ).in_set(TileShaderSystems))
    .add_systems(Update, (

        add_image_handle_to_tile_shader,
        update_wavy_time,

    ).in_set(TileShaderSystems))
    .add_plugins((
        plugin_tile_shader,
        MaterialTilemapPlugin::<MonoRepeatTextureOverlayMat>::default(),
        MaterialTilemapPlugin::<VoronoiTextureOverlayMat>::default(),
        MaterialTilemapPlugin::<WavyMat>::default(),
        MaterialTilemapPlugin::<RockyTerrainMat>::default(),
    ))

    .register_type::<MonoRepeatTextureOverlayMat>()
    .register_type::<VoronoiTextureOverlayMat>()
    .register_type::<TwoOverlaysExample>()
    .register_type::<WavyMat>()
    .register_type::<RockyTerrainMat>()
    ;
}
