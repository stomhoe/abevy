use bevy::prelude::*;
use crate::terrain::{
    terrprobe::{terrprobe_components::TerrProbeTempl, terrprobe_messages::TerrProbeJob},
    terrgen_messages::PendingOp,
};
use ::tilemap_shared::*;

pub fn process_spiral_pattern(
    pos_search: TerrProbeJob,
    templ: &TerrProbeTempl,
    root_oplist: DimensionRootOplist,
    curr_iteration_batch_i: i16,
    new_pending_ops: &mut Vec<PendingOp>,
    new_pos_searches: &mut Vec<TerrProbeJob>,
    search_failed: &mut Vec<Entity>,
) {
    let (mut curr_length_in_dir, mut steps_taken, mut dir_vec, mut pos, mut turn_parity) =
        spiral_state_at_batch_start(
            pos_search.search_start_pos,
            templ.step_size,
            curr_iteration_batch_i as u16,
            templ.iterations_per_batch,
        );

    trace!(target: "pos_search", "Spiral search started at pos {:?}, dir_vec {:?}, curr_length_in_dir {}, turns {}", pos, dir_vec, curr_length_in_dir, turn_parity);

    for _ in 0..templ.iterations_per_batch {
        pos = pos + GlobalTilePos(dir_vec.saturating_mul(IVec2::splat(templ.step_size as i32)));
        new_pending_ops.push(PendingOp {
            dimension_ref: pos_search.dimension_ref,
            oplist: root_oplist,
            gpos: pos,
            filtered_op: templ.opfilter_ref.0,
            requester: pos_search.requester,
            max_emitted_results: templ.max_emitted_results,
            mark_last_success_in_batch: false,
            matrix_spec: None,
        });

        steps_taken += 1;
        if steps_taken >= curr_length_in_dir {
            steps_taken = 0;

            dir_vec = dir_vec.perp();
            curr_length_in_dir = curr_length_in_dir.saturating_add(turn_parity as u64);
            turn_parity = !turn_parity;
        }
    }

    if curr_iteration_batch_i as u16 + 1 < templ.max_batches {
        let mut next_search = pos_search.clone();
        next_search.curr_iteration_batch_i = curr_iteration_batch_i + 1;
        new_pos_searches.push(next_search);
    } else {
        error!(target: "pos_search", "No more batches to search for {:?}", templ.opfilter_ref);
        search_failed.push(pos_search.requester);
    }
}

fn spiral_state_at_batch_start(
    start_pos: GlobalTilePos,
    step_size: u16,
    batch_i: u16,
    iterations_per_batch: u16,
) -> (u64, u64, IVec2, GlobalTilePos, bool) {
    let mut curr_length_in_dir: u64 = 1;
    let mut steps_taken: u64 = 0;
    let mut dir_vec = IVec2::new(0, 1);
    let mut pos = start_pos;
    let mut turn_parity = false;
    let steps_to_advance = batch_i as u64 * iterations_per_batch as u64;
    for _ in 0..steps_to_advance {
        pos = pos + GlobalTilePos(dir_vec.saturating_mul(IVec2::splat(step_size as i32)));
        steps_taken += 1;
        if steps_taken >= curr_length_in_dir {
            steps_taken = 0;
            dir_vec = dir_vec.perp();
            curr_length_in_dir = curr_length_in_dir.saturating_add(turn_parity as u64);
            turn_parity = !turn_parity;
        }
    }
    (curr_length_in_dir, steps_taken, dir_vec, pos, turn_parity)
}
