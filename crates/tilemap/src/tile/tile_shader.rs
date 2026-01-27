use bevy::prelude::*;
use bevy_common_assets::ron::RonAssetPlugin;
use bevy_ecs_tilemap::prelude::MaterialTilemapPlugin;
use bevy_replicon::prelude::*;
use common::common_states::AssetLoading;

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
        
        init_shaders,
        
    ).chain().in_set(TileShaderSystems))
    .add_systems(Update, (
        
        add_image_handle_to_tile_shader,
        update_wavy_time,
        
    ).in_set(TileShaderSystems))
    .add_observer(remove_tile_shader_from_map_on_despawn)
    .add_plugins((
        MaterialTilemapPlugin::<MonoRepeatTextureOverlayMat>::default(),
        MaterialTilemapPlugin::<VoronoiTextureOverlayMat>::default(),
        MaterialTilemapPlugin::<WavyMat>::default(),
        MaterialTilemapPlugin::<RockyTerrainMat>::default(),
        
        RonAssetPlugin::<ShaderRepeatTexSeri>::new(&["rep1shader.ron"]),
        RonAssetPlugin::<ShaderVoronoiShuffleSeri>::new(&["voroshu.ron"]),
        RonAssetPlugin::<ShaderWavySeri>::new(&["wavy.ron"]),
        RonAssetPlugin::<ShaderRockyTerrainSeri>::new(&["rocky.ron"]),
    ))
    .init_resource::<TileShaderEntityMap>()

    .register_type::<MonoRepeatTextureOverlayMat>()
    .register_type::<VoronoiTextureOverlayMat>()
    .register_type::<TwoOverlaysExample>()
    .register_type::<WavyMat>()
    .register_type::<RockyTerrainMat>()
    .register_type::<ShaderRepeatTexSerisHandles>()
    .register_type::<ShaderRepeatTexSeri>()
    .register_type::<ShaderVoroshuSerisHandles>()
    .register_type::<ShaderVoronoiShuffleSeri>()
    .register_type::<ShaderWavySerisHandles>()
    .register_type::<ShaderWavySeri>()
    .register_type::<ShaderRockyTerrainSerisHandles>()
    .register_type::<ShaderRockyTerrainSeri>()
    .register_type::<TileShaderEntityMap>()
    .register_type::<TileShader>()
    .register_type::<TileShaderRef>()
    .register_type::<EguiTileShaderHolder>()
    
    .replicate::<TileShader>()
    .replicate::<TileShaderRef>()
    .replicate::<EguiTileShaderHolder>()
    ;
}

