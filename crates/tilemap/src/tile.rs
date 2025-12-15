use bevy::ecs::entity_disabling::Disabled;
use bevy_common_assets::ron::RonAssetPlugin;
use bevy_replicon::prelude::*;
use common::common_states::{AssetsLoadingState, };
use bevy_ecs_tilemap::prelude::*;
use game_common::{ColorSamplersInitSystems, game_common_components::EntityZeroRef, game_common_components_samplers::EntityWeightedSampler};
use sprite::SpriteSystems;
use tilemap_shared::{GlobalTilePos, OplistSize};

#[allow(unused_imports)] use {bevy::prelude::*, superstate::superstate_plugin};

use crate::tile::{
    tile_components::*, tile_events::*, tile_init_systems::*, tile_materials::*, tile_resources::*, tile_sampler_init_systems::*, tile_sampler_resources::*, tile_shader_components::*, tile_shader_init_systems::*, tile_systems::*
};
mod tile_systems;
mod tile_init_systems;
mod tile_sampler_init_systems;
mod tile_shader_init_systems;
pub mod tile_components;
pub mod tile_shader_components;
pub mod tile_resources;
pub mod tile_sampler_resources;
pub mod tile_materials;
pub mod tile_events;


#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct TilingSystems;


#[allow(unused_parens, path_statements, )]
pub fn plugin(app: &mut App) {
    app
    .add_systems(Update, (
        flip_tile_along_x,
        (add_tile_weighted_samplers_to_map, client_sync_tile, ).run_if(in_state(ClientState::Connected)),
        sprite_tile_readjust_transform,
        instantiate_portal.run_if(in_state(ClientState::Disconnected)),
        make_child_of_chunk,
        add_handles,
    ))

    .add_systems(
        OnEnter(AssetsLoadingState::LocalFinished), (
        (init_shaders, add_shaders_to_map, ).chain()
    ).in_set(TilingSystems))
    .add_systems(
        OnEnter(AssetsLoadingState::ReplicatedFinished), (
            (init_tiles, add_tiles_to_map, map_min_dist_tiles, map_portal_tiles, init_tile_weighted_samplers, add_tile_weighted_samplers_to_map, init_tile_weighted_samplers_refs, )
            .chain().run_if(in_state(ClientState::Disconnected)),
    ).in_set(TilingSystems))

    .configure_sets(OnEnter(AssetsLoadingState::ReplicatedFinished), (ColorSamplersInitSystems.before(TilingSystems), SpriteSystems.before(TilingSystems)))

    .add_plugins((
        MaterialTilemapPlugin::<MonoRepeatTextureOverlayMat>::default(),
        MaterialTilemapPlugin::<VoronoiTextureOverlayMat>::default(),
        RonAssetPlugin::<ShaderRepeatTexSeri>::new(&["rep1shader.ron"]),
        RonAssetPlugin::<ShaderVoronoiSeri>::new(&["voro.ron"]),
        RonAssetPlugin::<TileSerialization>::new(&["tile.ron"]),
        RonAssetPlugin::<TileWeightedSamplerSeri>::new(&["sampler.ron"]),
    ))


    .register_type::<ShaderRepeatTexSerisHandles>()
    .register_type::<ShaderRepeatTexSeri>()
    .register_type::<ShaderVoronoiSerisHandles>()
    .register_type::<ShaderVoronoiSeri>()
    .register_type::<TileSerisHandles>()
    .register_type::<TileSerialization>()
    .register_type::<GlobalTilePos>()
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
    .register_type::<KeepDistanceFrom>()
    .register_type::<PortalRecipe>()
    .register_type::<PortalInstance>()


    .replicate::<Tile>()
    .replicate::<TileStrId>()
    .replicate::<TileImagePaths>()
    .replicate::<TileColor>()
    .replicate::<TileSamplerHolder>()
    .replicate::<InitialPos>()
    .replicate::<PortalsZeroEguiHolder>()
    .replicate::<TileInstancesHolder>()
    .replicate::<TileShaderRef>()
    .replicate_bundle::<(TilePos, TileTextureIndex, TileFlip, TileVisible, TileColor, TilePosOld, )>()
    .replicate_filtered::<Transform, With<Tile>>()
    .replicate_bundle::<(GlobalTilePos, EntityZeroRef)>()

    .replicate_filtered::<Transform, With<TilesEguiHolder>>()
    .replicate_filtered::<Transform, With<PortalsZeroEguiHolder>>()
    .replicate_filtered::<ChildOf, Or<(With<Tile>, Without<TilePos>, With<Disabled>)>>()


    .replicate_filtered::<ChildOf, With<EntityWeightedSampler>>()

    //usar feature
    .add_message::<SavedTileHadChunkDespawn>()


    ;
}

