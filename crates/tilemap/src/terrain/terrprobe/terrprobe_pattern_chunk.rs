use bevy::prelude::*;
use crate::terrain::{
    terrprobe::{terrprobe_components::TerrProbeTempl, terrprobe_messages::TerrProbeJob},
    terrgen_messages::PendingOp,
};
use ::tilemap_shared::*;

pub fn process_chunk_pattern(
    pos_search: TerrProbeJob,
    templ: &TerrProbeTempl,
    root_oplist: DimensionRootOplist,
    chunk_pos: ChunkPos,
    curr_iteration_batch_i: i16,
    new_pending_ops: &mut Vec<PendingOp>,
    search_failed: &mut Vec<Entity>,
) {
    if curr_iteration_batch_i > 0 {
        error!(target: "pos_search", "No more batches to search for {:?}", templ.opfilter_ref);
        search_failed.push(pos_search.requester);
        return;
    }

    for gpos in chunk_pos.get_tilepositions_within_chunk() {
        new_pending_ops.push(PendingOp {
            dimension_ref: pos_search.dimension_ref,
            oplist: root_oplist,
            gpos,
            filtered_op: templ.opfilter_ref.0,
            requester: pos_search.requester,
            max_emitted_results: templ.max_emitted_results,
            mark_last_success_in_batch: pos_search.collect_all_successes,
        });
    }

    search_failed.push(pos_search.requester);
}
