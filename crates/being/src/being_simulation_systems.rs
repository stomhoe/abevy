use bevy::prelude::*;
use being_shared::SimulatedBeingsWithin;
use common::log_targets::BEING_SYSTEM;
use tilemap_shared::*;

use crate::being_simulation_resources::MacroChunkNavIslands;

#[allow(unused_parens, )]
pub fn _____________iterate_simulated_beings_within_macrochunks(
    macrochunk_query: Query<(Entity, &SimulatedBeingsWithin), (With<MacroChunkNavIslands>, )>,
) {
    for (_macrochunk_ent, simulated_beings_within) in macrochunk_query.iter() {
        for _being_ent in simulated_beings_within.iter() {
        }
    }
}

#[allow(unused_parens, )]
pub fn insert_macrochunk_nav_islands(
    mut cmd: Commands,
    query: Query<Entity, (With<MacroChunk>, Added<MacrochunkPos>)>,
) {
    for macro_chunk_ent in query.iter() {
        debug!(
            target: BEING_SYSTEM,
            "Initializing macrochunk nav islands on macrochunk {:?}",
            macro_chunk_ent
        );
        cmd.entity(macro_chunk_ent).try_insert(MacroChunkNavIslands::default());
    }
}
