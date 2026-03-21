use bevy::prelude::*;
use being_shared::SimulatedBeingsWithin;
use tilemap::chunking::chunking_components::MacroChunk;

#[allow(unused_parens, )]
pub fn iterate_simulated_beings_within_macrochunks(
    macrochunk_query: Query<(Entity, &SimulatedBeingsWithin), (With<MacroChunk>, )>,
) {
    for (_macrochunk_ent, simulated_beings_within) in macrochunk_query.iter() {
        for _being_ent in simulated_beings_within.iter() {
        }
    }
}
