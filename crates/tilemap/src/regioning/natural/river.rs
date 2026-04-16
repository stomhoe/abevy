use bevy::{ecs::schedule::ApplyDeferred, prelude::*};

#[path = "river_components.rs"]
mod river_components;
#[path = "river_formation.rs"]
mod river_formation;
#[path = "river_sculpting_helpers.rs"]
mod river_sculpting_helpers;
#[path = "river_systems.rs"]
mod river_systems;

pub use river_components::{
    RiverDebugData,
    RiverMouthRejectReason,
    RiverRegionDebugInfo,
    RiverProbeKind,
    RiverProbeRequest,
    RiverRegionPlan,
};
pub use river_systems::{
    claim_chunks_for_river_structures,
    river_structure_building_system,
};

pub fn plugin(app: &mut App) {
    app
        .add_systems(Update, (
            claim_chunks_for_river_structures
                .in_set(crate::regioning::RegioningSystems)
                .after(crate::regioning::claim_chunks_for_various_dungeon_types)
                .run_if(on_message::<crate::regioning::regioning_messages::OfferChunk>.or(on_message::<crate::terrain::terrprobe::terrprobe_messages::SampledValuesCollected>).or(on_message::<crate::terrain::terrprobe::terrprobe_messages::SearchFailed>)),
            ApplyDeferred,
            river_structure_building_system.in_set(crate::regioning::StructureBuildingSystems),
        ).chain())
        .init_resource::<RiverDebugData>();
}
