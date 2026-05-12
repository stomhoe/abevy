use bevy::{prelude::*};

#[path = "river_components.rs"]
mod river_components;
#[path = "river_formation.rs"]
mod river_formation;
#[path = "river_sculpting_helpers.rs"]
mod river_sculpting_helpers;
#[path = "river_chunk_claiming_systems.rs"]
mod river_chunk_claiming_systems;

#[path = "river_building_systems.rs"]
mod river_building_systems;



pub use river_components::*;
pub use river_building_systems::*;
pub use river_chunk_claiming_systems::*;

pub fn plugin(app: &mut App) {
    app
        .add_systems(Update, (
            claim_chunks_for_river_structures
                .in_set(crate::regioning::RegioningSystems)
                //para determinismo
                .before(crate::regioning::claim_chunks_for_various_dungeon_types),
            river_structure_building_system.in_set(crate::regioning::StructureBuildingSystems),
        ))
        .init_resource::<RiverDebugData>();
}
