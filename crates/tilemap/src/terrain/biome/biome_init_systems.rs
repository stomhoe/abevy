#[allow(unused_imports)] use bevy::prelude::*;
use common::{
    common_components::StrId,
    log_targets::BIOME_INIT,
};
use game_common::game_common_components::Templ;

use crate::terrain::biome::{
    biome_components::{Biome, CreatureSampler},
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
                CreatureSampler::default(),
                Templ,
                str_id,
            ),
        ));
    }
    if biome_comps.is_empty() {
        error!(target: BIOME_INIT, "No biome defs were loaded during biome init");
        return;
    }
    cmd.insert_batch(biome_comps);
}
