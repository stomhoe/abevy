use bevy_replicon::prelude::*;
use color_sampler::ColorSampleSystems;
use common::{common_states::AssetLoading };
use bevy_ecs_tilemap::prelude::*;
use bevy::ecs::schedule::common_conditions::any_with_component;
use game_common::{game_common_samplers::EntityWeightedSampler};
use sprite_systems::AcSpriteSystems;
use bevy::prelude::*;
use crate::terrain::terrprobe::{search_suitable_positions, terrprobe_components::AwaitingStartSearch, terrprobe_messages::{SearchFailed, SuitablePosFound}};
use crate::tile::tile_systems::*;
use crate::tile::portal_init_systems::*;
use crate::tile::tile_init_systems::*;
use crate::tile::tile_sampler_init_systems::*;
use tile_despawn_systems::*;
use tile_flip_rotate_systems::*;
use tile_adj_retex_systems::*;

mod tile_systems;
mod tile_adj_retex_systems;
pub mod tile_delete_others_helpers;
pub mod tile_despawn_systems;
mod tile_flip_rotate_systems;
mod tile_init_systems;
mod portal_init_systems;
mod tile_sampler_init_systems;
pub mod tile_components;
pub mod tile_resources;
pub mod tile_sampler_resources;
pub mod tile_sampler_components;
pub mod tile_messages;
pub mod tile_bundles;
pub mod tile_shader;
#[allow(unused_imports)] pub use tile_bundles::*;
#[allow(unused_imports)] pub use tile_components::*;
#[allow(unused_imports)] pub use tile_messages::*;
#[allow(unused_imports)] pub use tile_resources::*;
#[allow(unused_imports)] pub use tile_sampler_components::*;
#[allow(unused_imports)] pub use tile_sampler_resources::*;
#[allow(unused_imports)] pub use tile_shader::*;
use ::tilemap_shared::*;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct TilingSystems;

#[allow(unused_parens, path_statements, )]
pub fn plugin(app: &mut App) {
    app
    .add_systems(Update, (

        start_portal_search
            .run_if(in_state(ClientState::Disconnected))
            .run_if(any_with_component::<AwaitingStartSearch>)
            .before(search_suitable_positions),
        resolve_portal_search_results
            .run_if(in_state(ClientState::Disconnected))
            .run_if(on_message::<SuitablePosFound>.or(on_message::<SearchFailed>))
            .after(search_suitable_positions),
        flip_tile_based_on_initial_pos_hash,
        rotate_tile_based_on_initial_pos_hash,
        sync_sprite_flips_with_tileflip,
        despawn_other_tiles_in_same_pos_if_not_excepted_from_added_delete_other_tiles.in_set(PreChunkDespawnSystems),
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
    .replicate::<WalkSpeedMultIfOnTop>()
    .replicate::<OffsetForTerrgenPlacement>()
    .replicate::<SizeInTiles>()
    .replicate::<FlipHorizontallyBasedOnHash>()
    .replicate::<FlipVerticallyBasedOnHash>()
    .replicate::<FlipDiagonallyBasedOnHash>()
    .replicate::<RotateCardinallyBasedOnHash>()
    .replicate::<TransformBasedCardRotation>()
    .replicate::<OffsetForTerrgenPlacement>()
    .replicate::<TileStepSfx>()


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
