use bevy::prelude::*;
use common::common_components::HashId;
use std::f32::consts::PI;

use crate::terrain::{
    terrprobe::{terrprobe_components::TerrProbeTempl, terrprobe_messages::TerrProbeJob},
    terrgen_messages::{PendingOp, PendingOpInput, PendingOpPurpose, PendingOpValueProbe},
};
use ::tilemap_shared::*;

pub fn process_concentric_pattern(
    pos_search: TerrProbeJob,
    templ: &TerrProbeTempl,
    filtered_op: HashId,
    root_oplist: DimensionRootOplist,
    radius_step: f32,
    sample_spacing: f32,
    curr_iteration_batch_i: i16,
    new_pending_ops: &mut Vec<PendingOp>,
    new_pos_searches: &mut Vec<TerrProbeJob>,
    search_failed: &mut Vec<Entity>,
) {
    if curr_iteration_batch_i as u16 >= templ.max_batches {
        error!(target: "pos_search", "No more batches to search for {:?}", pos_search);
        return;
    }

    let radius_step = radius_step.max(0.0001);
    let sample_spacing = sample_spacing.max(0.0001);
    let start_i_within_batch = (curr_iteration_batch_i == 0) as u16;
    for i_within_batch in start_i_within_batch..templ.iterations_per_batch {
        let ring_i = curr_iteration_batch_i as u32 * templ.iterations_per_batch as u32 + i_within_batch as u32;
        let radius = ring_i as f32 * radius_step;
        if radius <= 0.0 {
            new_pending_ops.push(PendingOp {
                oplist: root_oplist,
                input: PendingOpInput {
                    dim: pos_search.dimension_ref,
                    gpos: pos_search.search_start_pos,
                },
                purpose: PendingOpPurpose::ValueProbe(PendingOpValueProbe {
                    filtered_op,
                    requester: pos_search.requester,
                    max_emitted_results: templ.max_emitted_results,
                    mark_last_success_in_batch: false,
                    matrix_spec: None,
                }),
            });
            continue;
        }

        let circumference = 2.0 * PI * radius;
        let sample_count = ((circumference / sample_spacing).ceil() as u16).max(1);
        for sample_i in 0..sample_count {
            let angle = 2.0 * PI * sample_i as f32 / sample_count as f32;
            let gpos = pos_search.search_start_pos
                + GlobalTilePos::from(IVec2::new(
                    (radius * angle.cos()) as i32,
                    (radius * angle.sin()) as i32,
                ));
            new_pending_ops.push(PendingOp {
                oplist: root_oplist,
                input: PendingOpInput {
                    dim: pos_search.dimension_ref,
                    gpos,
                },
                purpose: PendingOpPurpose::ValueProbe(PendingOpValueProbe {
                    filtered_op,
                    requester: pos_search.requester,
                    max_emitted_results: templ.max_emitted_results,
                    mark_last_success_in_batch: false,
                    matrix_spec: None,
                }),
            });
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
