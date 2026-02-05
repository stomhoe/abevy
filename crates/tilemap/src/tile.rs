use std::time::Duration;

use bevy::{time::common_conditions::on_timer};
use bevy_common_assets::ron::RonAssetPlugin;
use bevy_replicon::prelude::*;
use color_sampler::ColorSampleSystems;
use common::{common_states::AssetLoading, };
use bevy_ecs_tilemap::prelude::*;
use game_common::{game_common_components::{EntityZero, EntityZeroRef, VisibilityGameState}, game_common_components_samplers::EntityWeightedSampler};
use sprite::AcSpriteSystems;
use tilemap_shared::{GlobalTilePos, OplistSize, PrevGlobalTilePos};
use bevy::prelude::*;

use crate::tile::{
    tile_components::*, tile_init_systems::*, tile_messages::*, tile_resources::*, tile_sampler_init_systems::*, tile_sampler_resources::*, tile_systems::*
} ;
pub mod tile_systems;
pub mod tile_init_systems;
pub mod tile_sampler_init_systems;
pub mod tile_components;
pub mod tile_resources;
pub mod tile_sampler_resources;
pub mod tile_sampler_components;
pub mod tile_messages;
pub mod tile_shader;

use crate::tile::tile_components::TileStrId;




#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct TilingSystems;


#[allow(unused_parens, path_statements, )]
pub fn plugin(app: &mut App) {
    app
    .add_systems(Update, (

        instantiate_portal.run_if(in_state(ClientState::Disconnected)),
        flip_tile_horizontally_based_on_initial_pos_hash,
        despawn_if_not_excepted,//DON'T TOUCH
        (add_spawned_tiles_to_gpos_map, ),
        add_projectile_colliders_to_tiles,
        (spritetile_snap_transform_to_global_pos).chain(),
        make_spritetile_child_of_chunk.run_if(on_timer(Duration::from_millis(500))),//DON   
        add_handles,
        init_childrensprite,
        emit_global_tile_pos_change,
        validate_portal_recipes,
        remove_tile_from_gpos_map_on_despawn,
    ))
    .add_systems(
        OnEnter(AssetLoading::SpawnReplicatedEntities), 
        (   
            init_tiles, map_tile_ezeros_map_id_to_entity, map_min_dist_tiles, map_portal_tiles, init_tile_weighted_samplers, map_tile_weighted_samplers_map_id_to_entity,  init_tile_weighted_samplers_refs, 
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
        RonAssetPlugin::<TileWeightedSamplerSeri>::new(&["tsampler.ron"]),
        plugin_tile_ezeros_map,
        plugin_tile_weighted_samplers_map,
    ))
    
    
    .register_type::<TileSerisHandles>()
    .register_type::<TileSerialization>()
    .register_type::<GlobalTilePos>()
    .register_type::<TileWeightedSamplerHandles>()
    .register_type::<TileWeightedSamplerSeri>()


    .register_type::<MinDistancesMap>()
    .register_type::<TileCategories>()
    .register_type::<KeepDistanceFrom>()
    .register_type::<PortalRecipe>()
    .register_type::<PortalTo>()
    .register_type::<BlocksProjectiles>()
    
    
    .replicate::<Tile>()
    .replicate::<TileChildSprite>()
    .replicate::<TileStrId>()
    .replicate::<TileImagePaths>()
    .replicate::<TileColor>()
    .replicate::<TileSamplerHolder>()
    .replicate::<InitialPos>()
    .replicate::<PrevGlobalTilePos>()
    .replicate::<PortalsZeroEguiHolder>()
    .replicate::<BlocksProjectiles>()

    .replicate_bundle::<(TilePos, TileTextureIndex, TileFlip, TileVisible, TileColor, TilePosOld, )>()
    .replicate_filtered::<Transform, With<Tile>>()
    .replicate_filtered::<EntityZeroRef, With<Tile>>()
    .replicate_filtered::<GlobalTilePos, With<Tile>>()
    .replicate_once::<(OplistSize)>()//LO USAN LAS TILE INSTANCES DE TILEMAP, NO BORRAR

    
    .replicate_filtered::<Transform, With<TilesEguiHolder>>()
    .replicate_filtered::<Transform, With<PortalsZeroEguiHolder>>()
    .replicate_filtered::<ChildOf, Or<(With<Tile>, Without<TilemapId>, )>>()
    
    
    .replicate_filtered::<ChildOf, With<EntityWeightedSampler>>()
    
    //usar feature
    .add_message::<SavedTileHadChunkDespawn>()
    .add_message::<GlobalTilePosChanged>()
    
    
    ;
}

