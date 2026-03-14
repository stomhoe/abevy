#[allow(unused_imports)] use bevy::prelude::*;
use common::define_entity_map_systems;

use crate::terrain::biome::biome_components::Biome;
pub use crate::terrain::biome::biome_seris::*;

define_entity_map_systems!(
    Biome,
    BiomeSeri, "seri.tilemap.terrain.biome", "biome.ron",
);
