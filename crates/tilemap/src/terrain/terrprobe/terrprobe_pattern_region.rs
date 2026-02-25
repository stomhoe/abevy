use bevy::prelude::*;

use crate::terrain::{
    terrprobe::{terrprobe_components::TerrProbeTempl, terrprobe_messages::TerrProbeJob},
    terrgen_messages::PendingOp,
};
use ::tilemap_shared::*;

pub fn process_region_pattern(
    pos_search: TerrProbeJob,
    templ: &TerrProbeTempl,
    root_oplist: DimensionRootOplist,
    spacing: u16,
    curr_iteration_batch_i: i16,
    new_pending_ops: &mut Vec<PendingOp>,
    search_failed: &mut Vec<Entity>,
) {
    if curr_iteration_batch_i > 0 {
        error!(target: "pos_search", "No more batches to search for {:?}", templ.opfilter);
        search_failed.push(pos_search.requester);
        return;
    }

    let spacing = spacing.max(1) as usize;
    let region_pos = pos_search.search_start_pos.to_chunkpos().to_region_pos();
    let (min_chunk, max_chunk_excl) = region_pos.chunk_bounds();
    let min_tile = min_chunk.to_tilepos();
    let max_tile_excl = max_chunk_excl.to_tilepos();

    for y in (min_tile.0.y..max_tile_excl.0.y).step_by(spacing) {
        for x in (min_tile.0.x..max_tile_excl.0.x).step_by(spacing) {
            new_pending_ops.push(PendingOp {
                dimension_ref: pos_search.dimension_ref,
                oplist: root_oplist,
                gpos: GlobalTilePos(IVec2::new(x, y)),
                filtered_op: pos_search.templ_ent,
                requester: pos_search.requester,
                max_emitted_results: templ.max_emitted_results,
            });
        }
    }

    search_failed.push(pos_search.requester);
}
