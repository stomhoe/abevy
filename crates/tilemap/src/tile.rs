use bevy::ecs::entity_disabling::Disabled;
use bevy_common_assets::ron::RonAssetPlugin;
use bevy_replicon::prelude::*;
use color_sample::ColorSampleSystems;
use common::{common_components::AnyDisabling, common_states::AssetLoading};
use bevy_ecs_tilemap::prelude::*;
use game_common::{game_common_components::{EntityZeroRef, VisibilityGameState}, game_common_components_samplers::EntityWeightedSampler};
use sprite::AcSpriteSystems;
use tilemap_shared::{GlobalTilePos, OplistSize};
use bevy::prelude::*;

use crate::{terrain_gen::terrgen_systems::process_pending_ops_and_collect_tiles, tile::{
    tile_components::*, tile_init_systems::*, tile_messages::*, tile_resources::*, tile_sampler_init_systems::*, tile_sampler_resources::*, tile_systems::*
}, tilemap_systems::{process_tiles_pre, tile_assign_child_of} };
pub mod tile_systems;
mod tile_init_systems;
mod tile_sampler_init_systems;
pub mod tile_components;
pub mod tile_resources;
pub mod tile_sampler_resources;
pub mod tile_sampler_components;
pub mod tile_messages;
pub mod tile_shader;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct TilingSystems;


#[allow(unused_parens, path_statements, )]
pub fn plugin(app: &mut App) {
    app
    .add_systems(Update, (
        instantiate_portal.run_if(in_state(ClientState::Disconnected)),
        (add_tiles_to_map, client_sync_tile, ).run_if(in_state(ClientState::Connected)),
        flip_tile_horizontally_based_on_initial_pos_hash,
        despawn_if_not_excepted.after(process_tiles_pre),//DON'T TOUCH
        (add_spawned_tiles_to_gpos_map, ),
        (spritetile_readjust_transform_to_match_globalpos).chain(),
        make_child_of_chunk,
        add_handles,
        init_tile_sprite,
        emit_global_tile_pos_change,
        validate_portal_recipes,
        remove_tile_from_gpos_map_on_despawn,
    ))
    .add_observer(remove_ezero_tile_from_map_on_despawn)
    .add_observer(remove_tws_from_map_on_despawn)
    
    /* .add_systems(map_portal_tiles
    OnEnter(AssetsLoadingState::LocalFinished), (
    ().chain()
    ).in_set(TilingSystems))*/
    .add_systems(
        OnEnter(AssetLoading::SpawnReplicatedEntities), 
        (   
            init_tiles, add_tiles_to_map, map_min_dist_tiles, map_portal_tiles, init_tile_weighted_samplers, init_tile_weighted_samplers_refs, 
        )
        .chain().in_set(TilingSystems))
        
    .configure_sets(OnEnter(AssetLoading::SpawnReplicatedEntities), 
        (ColorSampleSystems.before(TilingSystems), 
        AcSpriteSystems.before(TilingSystems),
        TilingSystems.run_if(in_state(ClientState::Disconnected)),
        
    ))
    
    .add_plugins((
        tile_shader::plugin,
        
        RonAssetPlugin::<TileSerialization>::new(&["tile.ron"]),
        RonAssetPlugin::<TileWeightedSamplerSeri>::new(&["sampler.ron"]),
    ))
    .init_resource::<TilesAtGpos>()
    .init_resource::<TileAsyncTasks>()
    .init_resource::<TileEzerosMap>()
    .init_resource::<TileWeightedSamplersMap>()
    
    .register_type::<TileSerisHandles>()
    .register_type::<TileSerialization>()
    .register_type::<GlobalTilePos>()
    .register_type::<TileWeightedSamplerHandles>()
    .register_type::<TileWeightedSamplerSeri>()
    .register_type::<TileEzerosMap>()
    .register_type::<TileWeightedSamplersMap>()


    .register_type::<MinDistancesMap>()
    .register_type::<TileCategories>()
    .register_type::<KeepDistanceFrom>()
    .register_type::<PortalRecipe>()
    .register_type::<PortalTo>()
    
    
    .replicate::<Tile>()
    .replicate::<TileStrId>()
    .replicate::<TileImagePaths>()
    .replicate::<TileColor>()
    .replicate::<TileSamplerHolder>()
    .replicate::<InitialPos>()
    .replicate::<PortalsZeroEguiHolder>()
    .replicate::<TileInstancesHolder>()

    .replicate_bundle::<(TilePos, TileTextureIndex, TileFlip, TileVisible, TileColor, TilePosOld, )>()
    .replicate_filtered::<Transform, With<Tile>>()
    .replicate_filtered::<EntityZeroRef, With<Tile>>()
    .replicate_filtered::<GlobalTilePos, With<Tile>>()
    
    .replicate_filtered::<Transform, With<TilesEguiHolder>>()
    .replicate_filtered::<Transform, With<PortalsZeroEguiHolder>>()
    .replicate_filtered::<ChildOf, Or<(With<Tile>, Without<TilemapId>, AnyDisabling)>>()
    
    
    .replicate_filtered::<ChildOf, With<EntityWeightedSampler>>()
    
    //usar feature
    .add_message::<SavedTileHadChunkDespawn>()
    .add_message::<GlobalTilePosChanged>()
    
    
    ;
}

