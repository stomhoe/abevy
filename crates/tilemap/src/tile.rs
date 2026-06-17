use bevy_lit::directional_light::DirectionalLight2d;
use bevy_replicon::prelude::*;
use color_sampler::ColorSampleSystems;
use common::{common_states::AssetLoading };
use common::common_resources::ImageSizeReady;
use bevy_ecs_tilemap::prelude::*;
use bevy::ecs::schedule::common_conditions::{any_with_component, on_message};
use game_common::HostSystems;
use sprite_systems::AcSpriteSystems;
use bevy::prelude::*;
use crate::terrain::terrprobe::{search_suitable_positions, terrprobe_components::AwaitingStartSearch, terrprobe_messages::{SearchFailed, SuitablePosFound}};
use crate::tile::tile_systems::*;
use crate::tile::portal_init_systems::*;
use crate::tile::tile_init_systems::*;
use crate::tile::tile_childrensprite_init_systems::*;
use crate::tile::tile_sampler_init_systems::*;

use tile_despawn_systems::*;
use tile_flip_rotate_systems::*;
use tile_adj_retex_systems::*;

mod tile_systems;
mod tile_adj_retex_systems;
pub mod tile_delete_others_systems;
pub mod tile_despawn_systems;
mod tile_flip_rotate_systems;
mod tile_init_systems;
mod tile_childrensprite_init_systems;
mod portal_init_systems;
mod tile_sampler_init_systems;
mod tile_init_helpers;
pub mod tile_seris;
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
#[allow(unused_imports)] pub use tile_seris::*;
#[allow(unused_imports)] pub use tile_resources::*;
#[allow(unused_imports)] pub use tile_sampler_components::*;
#[allow(unused_imports)] pub use tile_sampler_resources::*;
#[allow(unused_imports)] pub use tile_shader::*;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct TilingSystems;

#[allow(unused_parens, path_statements, )]
pub fn plugin(app: &mut App) {
    app



    .add_systems(Update, (

        start_portals_oe_search
            .run_if(in_state(ClientState::Disconnected))
            .run_if(any_with_component::<AwaitingStartSearch>)
            .before(search_suitable_positions),
        (despawn_other_tiles_in_same_pos_if_not_excepted_from_added_delete_other_tiles, 
            despawn_other_tiles_in_same_pos_if_not_excepted).in_set(PreChunkDespawnSystems),//DON'T TOUCH
        add_projectile_colliders_to_tiles,
        (snap_transform_to_gpos, 
            apply_corpse_pose_after_gpos_change.in_set(HostSystems)
        ).chain(),
        sync_tile_instance_templ_enti_ref_from_map,
        emit_global_tile_pos_change,
        validate_portal_recipes,
        
    ))
    .add_systems(Update, (
        flip_tile_based_on_initial_pos_hash,
        rotate_tile_based_on_initial_pos_hash,
        track_non_default_tile_cardinal_direction_changes,
        sync_sprite_flips_with_tileflip,
        sample_tile_normal_size_variations,
        
    ))
    .add_systems(Update, (
        add_handles,
        (init_childrensprite,
        ApplyDeferred,
        init_templ_childrensprite_light_occluders,).chain()
        
    ))

    .add_systems(Update, (
        handler_portals_search_results
            .run_if(in_state(ClientState::Disconnected))
            .run_if(on_message::<SuitablePosFound>.or(on_message::<SearchFailed>))
            .after(search_suitable_positions),
        sync_cardinal_dir_at_gpos_on_gpos_change
            .after(emit_global_tile_pos_change)
            .run_if(on_message::<GlobalTilePosChanged>),
        add_spawned_tiles_to_gpos_map
            .after(emit_global_tile_pos_change)
            .run_if(on_message::<GlobalTilePosChanged>),
        fix_childrensprite_spritemask_occluders_img_size.run_if(on_message::<ImageSizeReady>),
        safe_despawn_tile_at
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
    .add_observer(on_templ_tile_despawn)
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
    .replicate_once::<BlocksProjectiles>()

    .replicate_once::<AdjRetexConfig>()

    .replicate_once::<SpriteTile>()
    .replicate_once::<TileChildSprite>()
    .replicate_once::<TileStepSfx>()
    .replicate_once::<TileImagePaths>()
    .replicate::<TileColor>()
    .replicate_once::<InitialPos>()
    .replicate::<PrevPos>()
    .replicate_once::<PortalsZeroEguiHolder>()
    .replicate_once::<BlocksProjectiles>()
    .replicate_once::<WalkSpeedMultIfOnTop>()
    .replicate_once::<OffsetForTerrgenPlacement>()
    .replicate_once::<SizeInTiles>()
    .replicate_once::<FlipHorizontallyBasedOnHash>()
    .replicate_once::<FlipVerticallyBasedOnHash>()
    .replicate_once::<FlipDiagonallyBasedOnHash>()
    .replicate_once::<ChangeFacingDirectionBasedOnHash>()
    .replicate_once::<RotateTransform>()
    .replicate_once::<OffsetForTerrgenPlacement>()
    .replicate_once::<TileStepSfx>()
    .replicate::<U16TileIndex>()
    .replicate::<TileU16IndexHashIdMapping>()
    .replicate::<LightOccluderSeri>()


    .replicate_once::<GlobalTilePos>()

    .replicate_bundle::<(TilePos, TileTextureIndex, TileFlip, TileVisible, TileColor, TilePosOld, )>()
    .replicate_filtered::<Transform, (With<Tile>, Without<SnapTransformToGpos>)>()
    .replicate::<SnapTransformToGpos>()
    .replicate_once::<(OplistSize)>()//LO USAN LAS TILE INSTANCES DE TILEMAP, NO BORRAR


    .replicate_filtered::<ChildOf, (With<Tile>, Without<game_common::Templ>, Without<TilemapId>)>()
    .replicate_filtered::<ChildOf, (With<TileChildSprite>, Without<TilemapId>)>()



    //usar feature
    .add_message::<SavedTileHadChunkDespawn>()
    .add_message::<GlobalTilePosChanged>()
    .add_message::<RecheckTileAdjacency>()
    .add_message::<AiNavGridDirtyDim>()
    .add_message::<SafeDespawn>()



    ;
}
