use bevy::prelude::*;
use crate::terrain::{
    terrprobe::{terrprobe_components::TerrProbeTempl, terrprobe_messages::TerrProbeJob},
    terrgen_messages::{PendingOp, PendingOpMatrixSpec},
};
use ::tilemap_shared::*;

pub fn process_chunk_pattern(
    pos_search: TerrProbeJob,
    templ: &TerrProbeTempl,
    root_oplist: DimensionRootOplist,
    _curr_iteration_batch_i: i16,
    new_pending_ops: &mut Vec<PendingOp>,
    _new_pos_searches: &mut Vec<TerrProbeJob>,
    _search_failed: &mut Vec<Entity>,
) {
    let chunk_pos = pos_search.search_start_pos.to_chunkpos();
    let matrix_spec = templ.collect.then(|| PendingOpMatrixSpec {
        min: chunk_pos.to_tilepos(),
        matrix_size: ChunkPos::CHUNK_SIZE,
        spacing: 1,
    });
    for gpos in chunk_pos.get_tilepositions_within_chunk() {
        new_pending_ops.push(PendingOp {
            dimension_ref: pos_search.dimension_ref,
            oplist: root_oplist,
            gpos,
            filtered_op: templ.opfilter_ref.0,
            requester: pos_search.requester,
            max_emitted_results: u32::MAX,
            mark_last_success_in_batch: pos_search.collect_all_successes,
            matrix_spec,
        });
    }
}
