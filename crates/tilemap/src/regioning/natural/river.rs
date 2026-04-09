#[path = "river_components.rs"]
mod river_components;
#[path = "river_systems.rs"]
mod river_systems;

pub use river_components::{
    RiverDebugData,
    RiverPlans,
    RiverProbeRequest,
    RiverRegionDebugInfo,
};
pub use river_systems::{
    claim_chunks_for_river_structures,
    river_structure_building_system,
};
