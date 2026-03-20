#[allow(unused_imports)] use bevy::prelude::*;
use common::{define_entity_map_systems, };

use crate::terrain::biome::biome_components::Biome;
pub use crate::terrain::biome::biome_seris::*;


define_entity_map_systems!(
    main_component: Biome,
    with_filters: (),
    abbreviation: Biome,
    target: common::log_targets::ENTITY_MAP_SYSTEM,
    entity_prefix: "Biome",
    despawn_trigger: Biome,
    id_type: common::common_components::StrId,
    assets: [(BiomeSeri, "seri.tilemap.terrain.biome", "biome.ron")],
);
