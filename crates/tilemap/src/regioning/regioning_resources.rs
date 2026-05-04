use bevy::{platform::collections::*, prelude::*};
use bevy_replicon::prelude::*;

use ::tilemap_shared::*;
use crate::regioning::regioning_sgc_components::StructuredGenConfig;
use serde::{Deserialize, Serialize};
use common::common_components::*;
pub use crate::regioning::regioning_sgc_seris::*;

#[derive(Component, Debug, Deserialize, Serialize, Clone)]
#[require(Replicated, Prefix::trunc("StructureGenerationSettings"), AssetScoped, SelectedForHotReload)]
pub struct StructureGenerationSettings {
    /// Timeout in seconds to wait for StructureBuildCompliance before giving up
    pub structure_build_timeout_secs: f64,
    /// Timeout in seconds to wait before advancing claim processing when the current claim is still pending
    pub claimlist_advance_timeout_secs: f32,
    /// Timeout in seconds to wait for new region structure offers before building immediately
    pub region_offer_timeout_secs: f32,
    /// Fraction of region chunks allowed to be occupied by generated structures
    pub max_used_chunks_per_region_ratio: f32,
}

impl Default for StructureGenerationSettings {
    fn default() -> Self {
        Self {
            structure_build_timeout_secs: 4.0,
            claimlist_advance_timeout_secs: 0.1,
            region_offer_timeout_secs: 2.0,
            max_used_chunks_per_region_ratio: 0.07,
        }
    }
}

common::define_entity_map_systems!(
    main_component: StructuredGenConfig,
    with_filters: (),
    abbreviation: Sgc,
    target: "sgc",
    entity_prefix: "SGC",
    despawn_trigger: StructuredGenConfig,
    id_type: common::common_components::StrId,
);

#[derive(Debug, Clone, Default)]
pub struct SgcCommandSchema {
    pub room_spawn_shapes: HashSet<String>,
}

#[derive(Resource, Debug, Clone, Default)]
pub struct SgcCommandRegistry(pub HashMap<String, SgcCommandSchema>);

impl SgcCommandRegistry {
    pub fn with_builtins() -> Self {
        let mut registry = Self::default();
        registry.register_room_spawn_shapes(
            "chamberscorridors",
            [
                "rectangle",
                "circle",
                "triangle",
                "regular_polygon",
                "pentacle",
            ],
        );
        registry.register_room_spawn_shapes(
            "maze",
            [
                "square_room",
                "circle_room",
                "island_circle",
                "island_triangle",
                "island_hexagon",
                "island_square",
            ],
        );
        registry.register_room_spawn_shapes("drunkwalk", ["chamber_circle"]);
        registry.register_room_spawn_shapes("spiral", ["center_circle", "arm_inner", "arm_outer"]);
        registry.register_room_spawn_shapes("archi", ["center_spiral"]);
        registry
    }

    pub fn register_room_spawn_shapes<S, I>(&mut self, structure_id: &str, room_shapes: I)
    where
        S: AsRef<str>,
        I: IntoIterator<Item = S>,
    {
        let schema = self
            .0
            .entry(structure_id.to_string())
            .or_default();
        for room_shape in room_shapes {
            schema
                .room_spawn_shapes
                .insert(room_shape.as_ref().to_string());
        }
    }

    pub fn allowed_room_spawn_shapes_for(&self, structure_id: &str) -> Option<&HashSet<String>> {
        self.0.get(structure_id).map(|schema| &schema.room_spawn_shapes)
    }
}
#[derive(Resource, Default)]
pub struct LoadedRegions(pub HashMap<(DimensionRef, RegionPos), Entity>);

#[derive(Component, Debug, Clone, Default, Deserialize, Serialize)]
pub struct PrioritizedSgs(pub Vec<HashId>);

#[derive(Resource, Default)]
pub struct PrioritizedPerRegion(pub HashMap<(DimensionRef, RegionPos), Vec<HashId>>);
