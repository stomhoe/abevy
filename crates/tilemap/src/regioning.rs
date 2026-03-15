use bevy::prelude::*;
use bevy_replicon::prelude::AppRuleExt;
use common::common_states::*;
use game_common::game_common_timers::TimedOut;
use crate::{ regioning::{dungeoning::*, natural::*, regioning_components::*, regioning_messages::*, regioning_resources::*, regioning_sgc_components::{StructuredGenConfig, *}, regioning_sgc_init_systems::*, regioning_systems::*}, terrain::terrprobe::terrprobe_messages::*, };


#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct RegioningSystems;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct StructureBuildingSystems;

#[allow(unused_parens, )]
pub fn plugin(app: &mut App) {
    app
    .add_plugins((
        plugin_structured_gen_config,
    ))
    .add_systems(Update, (
        (
            (
                drunkwalk_dungeon_building_system
                    .after(read_chunk_claims_for_region_and_emit_build_orders_to_dungeoning_systems)
                    .run_if(on_message::<SgcPrepareTilesOrder>),
                corridor_dungeon_building_system
                    .after(read_chunk_claims_for_region_and_emit_build_orders_to_dungeoning_systems)
                    .run_if(on_message::<SgcPrepareTilesOrder>),
                spiral_dungeon_building_system
                    .after(read_chunk_claims_for_region_and_emit_build_orders_to_dungeoning_systems)
                    .run_if(on_message::<SgcPrepareTilesOrder>),
                archimedes_spiral_building_system
                    .after(read_chunk_claims_for_region_and_emit_build_orders_to_dungeoning_systems)
                    .run_if(on_message::<SgcPrepareTilesOrder>),
                maze_dungeon_building_system
                    .after(read_chunk_claims_for_region_and_emit_build_orders_to_dungeoning_systems)
                    .run_if(on_message::<SgcPrepareTilesOrder>),
                river_structure_building_system
                    .after(read_chunk_claims_for_region_and_emit_build_orders_to_dungeoning_systems)
                    .run_if(on_message::<SgcPrepareTilesOrder>),
            )
            .in_set(StructureBuildingSystems),
            offer_chunks_of_new_regions_to_dungeoning_systems,
            (
                claim_chunks_for_various_dungeon_types
                    .after(offer_chunks_of_new_regions_to_dungeoning_systems)
                    .run_if(on_message::<OfferChunk>),
                claim_chunks_for_river_structures
                    .after(offer_chunks_of_new_regions_to_dungeoning_systems)
                    .run_if(on_message::<OfferChunk>.or(on_message::<SampledValuesCollected>)),
            ),
            read_chunk_claims_for_region_and_emit_build_orders_to_dungeoning_systems
                .after(claim_chunks_for_various_dungeon_types)
                .after(claim_chunks_for_river_structures)
                .run_if(on_message::<ChunksClaim>.or(on_message::<RecheckRegion>)),
            failsafe_timeout_pending_chunks,
            add_planned_tiles_to_region,
            mark_as_building_started_timed_out.run_if(on_message::<TimedOut>),
            advance_i_on_claimlist_timeout,
            clonespawn_tiles_on_chunk_spawn
            .before(crate::tilemap_systems::process_tiles_pre)//removing this breaks it
            ,
        ).in_set(RegioningSystems),
        despawn_empty_regions,
    ))
    .add_systems(
        OnEnter(AssetLoading::SpawnReplicatedEntities), (
            init_structured_gen_configs, map_structured_gen_config_id_to_entity
        ).in_set(RegioningSystems))
    .add_observer(on_region_despawn_remove_from_loaded_regions)

    .init_resource::<LoadedRegions>()
    .init_resource::<Prioritized>()
    .init_resource::<PrioritizedPerRegion>()
    .init_resource::<RiverPlans>()
    .init_resource::<RiverDebugData>()

    .replicate::<WhitelistedFilterOf>()
    .replicate::<StructuredGenConfig>()
    .replicate::<SgcsWeightedSampler>()
    .replicate_once::<Region>()

    .add_message::<OfferChunk>()
    .add_message::<ChunksClaim>()
    .add_message::<SgcPrepareTilesOrder>()
    .add_message::<StructureBuildCompliance>()
    .add_message::<RecheckRegion>()



;
}

pub mod regioning_components;
pub mod regioning_resources;
pub mod regioning_seris;
pub mod regioning_messages;
pub mod regioning_sgc_components;
pub mod dungeoning;
pub mod natural;
mod regioning_systems;
mod regioning_sgc_init_systems;

#[allow(unused_imports, ambiguous_glob_reexports)]
pub mod prelude {
    pub use super::{
        regioning_components::*,
        regioning_resources::*,
        regioning_seris::*,
        regioning_messages::*,
        regioning_sgc_components::*,
        dungeoning::*,
        natural::*,
    };
}
