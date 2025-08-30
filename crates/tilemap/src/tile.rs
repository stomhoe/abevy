use bevy_common_assets::ron::RonAssetPlugin;
use bevy_replicon::prelude::*;
use common::common_states::{AssetsLoadingState, };
use bevy_ecs_tilemap::prelude::*;
use game_common::ColorSamplersInitSystems;
use tilemap_shared::GlobalTilePos;

#[allow(unused_imports)] use {bevy::prelude::*, superstate::superstate_plugin};

use crate::tile::{
    tile_components::*, tile_init_systems::*, tile_materials::*, tile_resources::*, tile_sampler_init_systems::*, tile_sampler_resources::*, tile_shader_init_systems::*, tile_systems::*
};
mod tile_systems;
mod tile_init_systems;
mod tile_sampler_init_systems;
mod tile_shader_init_systems;
pub mod tile_components;
pub mod tile_resources;
pub mod tile_sampler_resources;
pub mod tile_materials;


#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct TilingSystems;



#[allow(unused_parens, path_statements, )]
pub fn plugin(app: &mut App) {
    app
        .add_systems(Update, (
            flip_tile_along_x,
            (add_tile_weighted_samplers_to_map, ).run_if(not(server_or_singleplayer)),
        ))

        .add_systems(
            OnEnter(AssetsLoadingState::LocalFinished), (
            (init_shaders, add_shaders_to_map, init_tiles, add_tiles_to_map, map_min_dist_tiles).chain()
        ).in_set(TilingSystems))
        .add_systems(
            OnEnter(AssetsLoadingState::ReplicatedFinished), (
                (init_tile_weighted_samplers, add_tile_weighted_samplers_to_map, init_tile_weighted_samplers_weights, )
                .chain().run_if(server_or_singleplayer),
        ).in_set(TilingSystems))

        .configure_sets(OnEnter(AssetsLoadingState::LocalFinished), ColorSamplersInitSystems.before(TilingSystems))

        .add_plugins((
            MaterialTilemapPlugin::<MonoRepeatTextureOverlayMat>::default(),
            MaterialTilemapPlugin::<VoronoiTextureOverlayMat>::default(),
            RonAssetPlugin::<ShaderRepeatTexSeri>::new(&["rep1shader.ron"]),
            RonAssetPlugin::<ShaderVoronoiSeri>::new(&["voro.ron"]),
            RonAssetPlugin::<TileSeri>::new(&["tile.ron"]),
            RonAssetPlugin::<TileWeightedSamplerSeri>::new(&["sampler.ron"]),
        ))
        
        .add_server_trigger::<TileEntitiesMap>(Channel::Unordered)
        .make_trigger_independent::<TileEntitiesMap>()
        .add_server_trigger::<TileWeightedSamplersMap>(Channel::Unordered)
        .make_trigger_independent::<TileWeightedSamplersMap>()

        .register_type::<ShaderRepeatTexSerisHandles>()
        .register_type::<ShaderRepeatTexSeri>()
        .register_type::<ShaderVoronoiSerisHandles>()
        .register_type::<ShaderVoronoiSeri>()
        .register_type::<TileSerisHandles>()
        .register_type::<TileSeri>()
        .register_type::<GlobalTilePos>()
        .register_type::<TileRef>()
        .register_type::<TileWeightedSamplerHandles>()
        .register_type::<TileWeightedSamplerSeri>()
        .register_type::<TileEntitiesMap>()
        .register_type::<TileWeightedSamplersMap>()
        .register_type::<TileShaderEntityMap>()
        .register_type::<TileShader>()
        .register_type::<TileShaderRef>()
        .register_type::<MonoRepeatTextureOverlayMat>()
        .register_type::<VoronoiTextureOverlayMat>()
        .register_type::<TwoOverlaysExample>()
        .register_type::<MinDistancesMap>()
        .register_type::<TileCategories>()

        //usar feature
        .add_observer(client_map_server_tiling)


    ;
}

