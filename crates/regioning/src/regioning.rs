use bevy::prelude::*;
use bevy::ecs::schedule::common_conditions::any_with_component;
use bevy_replicon::prelude::AppRuleExt;
use common::common_states::*;
use game_common::{GameplaySystems, game_common_timers::TimedOut};
use tilemap_shared::*;
use tilemap::{process_tiles_pre, terrain::{TerrainGenSystems, terrprobe::TerrainProbeSystems}};

use crate::regioning::{regioning_sgc_init_systems::*, regioning_systems::*};

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct RegioningSystems;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct StructureBuildingSystems;

#[allow(unused_parens, )]
pub fn plugin(app: &mut App) {
    app
    .add_plugins((
        plugin_structured_gen_config,
        natural::river::plugin,
    ))
    .add_systems(Update, (
        (
            (
                drunkwalk_dungeon_building_system,
                corridor_dungeon_building_system,
                spiral_dungeon_building_system,
                archimedes_spiral_building_system,
                maze_dungeon_building_system,
            )
            .in_set(StructureBuildingSystems),
            offer_chunks_of_new_regions_to_dungeoning_systems,
            (
                claim_chunks_for_various_dungeon_types
                    .after(offer_chunks_of_new_regions_to_dungeoning_systems)
                    .run_if(on_message::<OfferChunk>),
            ),
            read_chunk_claims_for_region_and_emit_build_orders_to_dungeoning_systems
                .after(claim_chunks_for_various_dungeon_types)
                .after(claim_chunks_for_river_structures)
                .run_if(on_message::<ChunksClaim>.or(on_message::<RecheckRegion>)),
            failsafe_timeout_pending_chunks,
            add_planned_tiles_to_region,
            mark_as_building_started_timed_out.run_if(on_message::<TimedOut>),
            advance_i_on_claimlist_timeout,

            clonespawn_tiles_on_chunk_spawn.before(process_tiles_pre)//removing this breaks it
            ,
        ).in_set(RegioningSystems),
        despawn_empty_regions,
    ))
    .configure_sets(Update,(
        StructureBuildingSystems.after(read_chunk_claims_for_region_and_emit_build_orders_to_dungeoning_systems).run_if(on_message::<SgcPrepareTilesOrder>),
        RegioningSystems.in_set(GameplaySystems)
    ))
    .add_systems(
        OnEnter(AssetLoading::SpawnReplicatedEntities), (
            init_structure_generation_settings,
            init_structured_gen_configs, map_structured_gen_config_id_to_entity,
        ).chain().in_set(RegioningSystems)
    )
    .configure_sets(OnEnter(AssetLoading::SpawnReplicatedEntities),
        (RegioningSystems.after(TerrainGenSystems))
    )
    .add_observer(on_region_despawn_remove_from_loaded_regions)

    .init_resource::<LoadedRegions>()
    .init_resource::<PrioritizedPerRegion>()
    .insert_resource(SgcCommandRegistry::with_builtins())

    .replicate::<WhitelistedFilterOf>()
    .replicate::<StructuredGenConfig>()
    .replicate::<StructureGenerationSettings>()
    .replicate::<PrioritizedSgs>()
    .replicate::<SgcsWeightedSampler>()

    .add_message::<OfferChunk>()
    .add_message::<ChunksClaim>()
    .add_message::<SgcPrepareTilesOrder>()
    .add_message::<StructureBuildCompliance>()
    .add_message::<RecheckRegion>()



;
}

pub mod regioning_resources;
pub mod regioning_sgc_seris;
pub mod dungeoning;
pub mod natural;
mod regioning_systems;
mod regioning_sgc_init_systems;
#[allow(unused_imports)] pub use dungeoning::*;
#[allow(unused_imports)] pub use natural::*;
#[allow(unused_imports)] pub use regioning_messages::*;
#[allow(unused_imports)] pub use regioning_sgc_seris::*;
