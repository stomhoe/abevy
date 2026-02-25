use bevy::prelude::*;
use std::f32::consts::PI;

use crate::terrain::{
    terrprobe::{terrprobe_components::TerrProbeTempl, terrprobe_messages::TerrProbeJob},
    terrgen_messages::PendingOp,
};
use ::tilemap_shared::*;

pub fn process_sun_pattern(
    pos_search: TerrProbeJob,
    templ: &TerrProbeTempl,
    root_oplist: DimensionRootOplist,
    curr_iteration_batch_i: i16,
    new_pending_ops: &mut Vec<PendingOp>,
    new_pos_searches: &mut Vec<TerrProbeJob>,
    search_failed: &mut Vec<Entity>,
) {
    if curr_iteration_batch_i as u16 >= templ.max_batches {
        error!(target: "pos_search", "No more batches to search for {:?}", pos_search);
        return;
    }

    let ray_curve_per_distance = match templ.probe_pattern {
        crate::terrain::terrprobe::terrprobe_messages::ProbePattern::Radial(ray_curve_per_distance) => {
            ray_curve_per_distance.filter(|curve| *curve >= 0.0)
        }
        _ => None,
    };
    let divisions = if ray_curve_per_distance.is_some() { 16 } else { 8 };
    let calculate_pos = |i_within_batch: u16, base_angle: f32| -> GlobalTilePos {
        let global_i = (curr_iteration_batch_i as u16 * templ.iterations_per_batch as u16 + i_within_batch)
            as f32
            * templ.step_size as f32;
        let angle = base_angle + ray_curve_per_distance.map_or(0.0, |curve| global_i * curve);
        pos_search.search_start_pos
            + GlobalTilePos::from(IVec2::new(
                (global_i * angle.cos()) as i32,
                (global_i * angle.sin()) as i32,
            ))
    };

    let start_i_within_batch = (curr_iteration_batch_i == 0) as u16;
    for i in 0..divisions {
        let angle = 2.0 * PI * (i as f32) / (divisions as f32);
        for i_within_batch in start_i_within_batch..templ.iterations_per_batch {
            new_pending_ops.push(PendingOp {
                oplist: root_oplist,
                dimension_ref: pos_search.dimension_ref,
                gpos: calculate_pos(i_within_batch, angle),
                filtered_op: pos_search.templ_ent,
                requester: pos_search.requester,
                max_emitted_results: templ.max_emitted_results,
            });
        }
    }

    if curr_iteration_batch_i as u16 + 1 < templ.max_batches {
        let mut next_search = pos_search.clone();
        next_search.curr_iteration_batch_i = curr_iteration_batch_i + 1;
        new_pos_searches.push(next_search);
    } else {
        error!(target: "pos_search", "No more batches to search for {:?}", templ.opfilter);
        search_failed.push(pos_search.requester);
    }
}
