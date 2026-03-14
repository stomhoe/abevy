
use bevy_replicon::prelude::*;
use color_sampler::ColorSampleSystems;
use common::{common_states::AssetLoading };
use bevy_ecs_tilemap::prelude::*;
use game_common::{game_common_samplers::EntityWeightedSampler};
use sprite_systems::AcSpriteSystems;
use bevy::prelude::*;

use crate::tile::{
    portal_init_systems::*, tile_adj_retex_systems::*, tile_components::*, tile_despawn_systems::*, tile_flip_rotate_systems::*, tile_init_systems::*, tile_messages::*, tile_resources::*, tile_sampler_init_systems::*, tile_sampler_resources::*, tile_systems::*
} ;
pub mod tile_systems;
pub mod tile_adj_retex_systems;
pub mod tile_despawn_systems;
pub mod tile_flip_rotate_systems;
pub mod tile_init_systems;
pub mod portal_init_systems;
pub mod tile_sampler_init_systems;
pub mod tile_components;
pub mod tile_resources;
pub mod tile_sampler_resources;
pub mod tile_sampler_components;
pub mod tile_messages;
pub mod tile_bundles;
pub mod tile_shader;
use ::tilemap_shared::*;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct TilingSystems;

#[allow(unused_parens, path_statements, )]
pub fn plugin(app: &mut App) {
    app
    .add_systems(Update, (

        instantiate_portal.run_if(in_state(ClientState::Disconnected)),
        flip_tile_based_on_initial_pos_hash,
        rotate_tile_based_on_initial_pos_hash,
        sync_sprite_flips_with_tileflip,
        despawn_other_tiles_in_same_pos_if_not_excepted.in_set(PreChunkDespawnSystems),//DON'T TOUCH
        add_spawned_tiles_to_gpos_map
            .after(emit_global_tile_pos_change)
            .run_if(on_message::<GlobalTilePosChanged>),
        add_projectile_colliders_to_tiles,
        (snap_transform_to_gpos).chain(),
        add_handles,
        init_childrensprite,
        emit_global_tile_pos_change,
        validate_portal_recipes,
        safe_despawn_tile_at
            .after(emit_global_tile_pos_change)
            .run_if(on_message::<SafeDespawn>),
        reckeck_adjacency_for
            .after(emit_global_tile_pos_change)
            .run_if(on_message::<GlobalTilePosChanged>),
        tile_adjacency_dependent_retexturing_system
            .after(reckeck_adjacency_for)
            .after(safe_despawn_tile_at)
            .run_if(on_message::<RecheckTileAdjacency>),//.in_set(PreChunkDespawnSystems),

    ))
    .add_observer(on_spritetile_despawn)
    .add_observer(on_ezero_tile_despawn)
    .add_systems(
        OnEnter(AssetLoading::SpawnReplicatedEntities),
        (
            init_tiles, map_tile_id_to_entity, map_min_dist_tiles, map_portal_tiles, init_tile_weighted_samplers, map_tile_weighted_sampler_id_to_entity,  init_tile_weighted_samplers_part_two,
        )
        .chain().in_set(TilingSystems))

    .configure_sets(OnEnter(AssetLoading::SpawnReplicatedEntities),
        (ColorSampleSystems.before(TilingSystems),
        AcSpriteSystems.before(TilingSystems),
        TilingSystems.run_if(in_state(ClientState::Disconnected)),

    ))
    .add_plugins((
        tile_shader::plugin,
        plugin_tile,
        plugin_tile_weighted_sampler,
    ))
    //.replicate::<MinDistancesMap>()
    .replicate::<BlocksProjectiles>()

    .replicate::<AdjRetexConfig>()

    .replicate::<SpriteTile>()
    .replicate::<TileChildSprite>()
    .replicate::<TileStepSfx>()
    .replicate::<TileImagePaths>()
    .replicate::<TileColor>()
    .replicate::<InitialPos>()
    .replicate::<PrevPos>()
    .replicate::<PortalsZeroEguiHolder>()
    .replicate::<BlocksProjectiles>()
    .replicate::<TiledCollisionMask>()
    .replicate::<WalkSpeedMultIfOnTop>()
    .replicate::<OffsetForTerrgenPlacement>()
    .replicate::<SizeInTiles>()
    .replicate::<FlipHorizontallyBasedOnHash>()
    .replicate::<FlipVerticallyBasedOnHash>()
    .replicate::<FlipDiagonallyBasedOnHash>()
    .replicate::<RotateCardinallyBasedOnHash>()
    .replicate::<TransformBasedCardRotation>()
    .replicate::<OffsetForTerrgenPlacement>()


    .replicate_once::<GlobalTilePos>()

    .replicate_bundle::<(TilePos, TileTextureIndex, TileFlip, TileVisible, TileColor, TilePosOld, )>()
    .replicate_filtered::<Transform, With<Tile>>()
    .replicate_once::<(OplistSize)>()//LO USAN LAS TILE INSTANCES DE TILEMAP, NO BORRAR


    .replicate_filtered::<ChildOf, (Without<TilemapId>)>()


    .replicate_filtered::<ChildOf, With<EntityWeightedSampler>>()

    //usar feature
    .add_message::<SavedTileHadChunkDespawn>()
    .add_message::<GlobalTilePosChanged>()
    .add_message::<RecheckTileAdjacency>()
    .add_message::<SafeDespawn>()



    ;
}

#[allow(unused_imports, ambiguous_glob_reexports)]
pub mod prelude {
    pub use super::{
        tile_systems::*,
        tile_adj_retex_systems::*,
        tile_despawn_systems::*,
        tile_flip_rotate_systems::*,
        tile_init_systems::*,
        portal_init_systems::*,
        tile_sampler_init_systems::*,
        tile_components::*,
        tile_resources::*,
        tile_sampler_resources::*,
        tile_sampler_components::*,
        tile_messages::*,
        tile_bundles::*,
        tile_shader::*,
    };
}
