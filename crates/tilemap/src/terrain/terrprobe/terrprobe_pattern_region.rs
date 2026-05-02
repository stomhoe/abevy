use bevy::prelude::*;
use common::common_components::HashId;

use crate::terrain::{
    terrprobe::{terrprobe_components::TerrProbeTempl, terrprobe_messages::TerrProbeJob},
    terrgen_messages::{PendingOp, PendingOpInput, PendingOpMatrixSpec, PendingOpPurpose, PendingOpValueProbe},
};
use ::tilemap_shared::*;

pub fn process_region_pattern(
    pos_search: TerrProbeJob,
    templ: &TerrProbeTempl,
    filtered_op: HashId,
    root_oplist: DimensionRootOplist,
    spacing: u16,
    region_multiplier: f32,
    new_pending_ops: &mut Vec<PendingOp>,
) {
    let spacing = spacing.max(16) as usize;
    let spacing_u16 = spacing as u16;
    let region_pos = pos_search.search_start_pos.to_chunkpos().to_region_pos();
    let (base_min_chunk, base_max_chunk_excl) = region_pos.chunk_bounds();
    let base_min_tile = base_min_chunk.to_tilepos();
    let base_max_tile_excl = base_max_chunk_excl.to_tilepos();

    let base_w = (base_max_tile_excl.0.x - base_min_tile.0.x).max(1);
    let base_h = (base_max_tile_excl.0.y - base_min_tile.0.y).max(1);
    let target_w = ((base_w as f32 * region_multiplier.max(0.0001)).round() as i32).max(1);
    let target_h = ((base_h as f32 * region_multiplier.max(0.0001)).round() as i32).max(1);
    let extra_w = target_w - base_w;
    let extra_h = target_h - base_h;
    let left_extra = extra_w.div_euclid(2);
    let right_extra = extra_w - left_extra;
    let bottom_extra = extra_h.div_euclid(2);
    let top_extra = extra_h - bottom_extra;

    let min_tile = GlobalTilePos(IVec2::new(
        base_min_tile.0.x - left_extra,
        base_min_tile.0.y - bottom_extra,
    ));
    let max_tile_excl = GlobalTilePos(IVec2::new(
        base_max_tile_excl.0.x + right_extra,
        base_max_tile_excl.0.y + top_extra,
    ));

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
                oplist: root_oplist,
                input: PendingOpInput {
                    dim: pos_search.dimension_ref,
                    gpos: GlobalTilePos(IVec2::new(x, y)),
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
}
