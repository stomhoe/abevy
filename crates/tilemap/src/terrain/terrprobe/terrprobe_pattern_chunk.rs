use bevy::prelude::*;
use common::common_components::HashId;
use crate::terrain::{
    terrprobe::{terrprobe_components::TerrProbeTempl, terrprobe_messages::TerrProbeJob},
    terrgen_messages::{PendingOp, PendingOpInput, PendingOpMatrixSpec, PendingOpPurpose, PendingOpValueProbe},
};
use ::tilemap_shared::*;

pub fn process_chunk_pattern(
    pos_search: TerrProbeJob,
    templ: &TerrProbeTempl,
    filtered_op: HashId,
    root_oplist: DimensionRootOplist,
    new_pending_ops: &mut Vec<PendingOp>,
) {
    let chunk_pos = pos_search.search_start_pos.to_chunkpos();
    let matrix_spec = templ.collect.then(|| PendingOpMatrixSpec {
        min: chunk_pos.to_tilepos(),
        matrix_size: ChunkPos::CHUNK_SIZE,
        spacing: 1,
    });
    for gpos in chunk_pos.get_tilepositions_within_chunk() {
        new_pending_ops.push(PendingOp {
            oplist: root_oplist,
            input: PendingOpInput {
                dim: pos_search.dimension_ref,
                gpos,
            },
            purpose: PendingOpPurpose::ValueProbe(PendingOpValueProbe {
                filtered_op,
                requester: pos_search.requester,
                max_emitted_results: u32::MAX,
                mark_last_success_in_batch: pos_search.collect_all_successes,
                matrix_spec,
            }),
        });
    }
}
