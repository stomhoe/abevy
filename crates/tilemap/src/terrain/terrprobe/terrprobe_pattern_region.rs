use bevy::prelude::*;

use crate::terrain::{
    terrprobe::{terrprobe_components::TerrProbeTempl, terrprobe_messages::TerrProbeJob},
    terrgen_messages::{PendingOp, PendingOpMatrixSpec},
};
use ::tilemap_shared::*;

pub fn process_region_pattern(
    pos_search: TerrProbeJob,
    templ: &TerrProbeTempl,
    root_oplist: DimensionRootOplist,
    spacing: u16,
    _curr_iteration_batch_i: i16,
    new_pending_ops: &mut Vec<PendingOp>,
    _new_pos_searches: &mut Vec<TerrProbeJob>,
    _search_failed: &mut Vec<Entity>,
) {
    let spacing = spacing.max(16) as usize;
    let spacing_u16 = spacing as u16;
    let region_pos = pos_search.search_start_pos.to_chunkpos().to_region_pos();
    let (min_chunk, max_chunk_excl) = region_pos.chunk_bounds();
    let min_tile = min_chunk.to_tilepos();
    let max_tile_excl = max_chunk_excl.to_tilepos();
    let width = (max_tile_excl.0.x - min_tile.0.x) as usize;
    let height = (max_tile_excl.0.y - min_tile.0.y) as usize;
    let cols = width.div_ceil(spacing) as u32;
    let rows = height.div_ceil(spacing) as u32;
    let matrix_spec = templ.collect.then(|| PendingOpMatrixSpec {
        min: min_tile,
        matrix_size: UVec2::new(cols, rows),
        spacing: spacing_u16,
    });

    for y in (min_tile.0.y..max_tile_excl.0.y).step_by(spacing) {
        for x in (min_tile.0.x..max_tile_excl.0.x).step_by(spacing) {
            new_pending_ops.push(PendingOp {
                dimension_ref: pos_search.dimension_ref,
                oplist: root_oplist,
                gpos: GlobalTilePos(IVec2::new(x, y)),
                filtered_op: templ.opfilter_ref.0,
                requester: pos_search.requester,
                max_emitted_results: u32::MAX,
                mark_last_success_in_batch: pos_search.collect_all_successes,
                matrix_spec,
            });
        }
    }
}
