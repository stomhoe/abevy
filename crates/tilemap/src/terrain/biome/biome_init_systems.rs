#[allow(unused_imports)] use bevy::prelude::*;
use common::{
    common_components::StrId,
    log_targets::BIOME_INIT,
};
use game_common::game_common_components::EntityZero;

use crate::terrain::biome::{
    biome_components::{Biome, SpawnablesPerBiome},
    biome_resources::*,
};

pub fn init_biomes(
    mut cmd: Commands,
    biome_entity_map: Res<BiomeEntityMap>,
) {
    if !biome_entity_map.0.is_empty() {
        return;
    }

    let mut biome_comps = Vec::new();
    for seri in load_biome_seri_defs() {
        let Ok(str_id) = StrId::new_with_result(&seri.id, 1) else {
            error!(target: BIOME_INIT, "Failed to create StrId for biome '{}'", seri.id);
            continue;
        };
        biome_comps.push((
            cmd.spawn_empty().id(),
            (
                Biome,
                SpawnablesPerBiome::default(),
                EntityZero,
                str_id,
            ),
        ));
    }
    if biome_comps.is_empty() {
        debug!(target: BIOME_INIT, "No biome defs were loaded during biome init");
        return;
    }
    let biome_count = biome_comps.len();
    cmd.insert_batch(biome_comps);
    debug!(target: BIOME_INIT, "Spawned {} biome entities from biome defs", biome_count);
}
