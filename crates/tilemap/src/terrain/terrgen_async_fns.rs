use bevy::{ecs::entity::EntityHashMap, prelude::*};
use ::common::*;

use crate::terrain::{
    terrgen_async_resources::*,
    terrgen_helpers::*,
    terrprobe::{terrprobe_messages::*},
    terrgen_messages::*,
};
use ::tilemap_shared::*;

pub(crate) fn build_pending_ops_for_launch(work_items: Vec<ChunkTerrGenWork>) -> Vec<PendingOp> {
    let total_area: usize = work_items
        .iter()
        .map(|work| {
            (ChunkPos::CHUNK_SIZE.x / work.oplist_size.x()) as usize
                * (ChunkPos::CHUNK_SIZE.y / work.oplist_size.y()) as usize
        })
        .sum();
    let mut batch = Vec::with_capacity(total_area);
    for work in work_items {
        for_each_root_gpos(&work, |gpos| {
            trace!(
                target: TERRGEN_SYSTEM,
                "Spawning terr operation {:?} at {:?} in chunk {:?}, oplist_size: {:?}",
                work.root_oplist,
                gpos,
                work.chunk_pos,
                work.oplist_size
            );
            batch.push(PendingOp {
                oplist: work.root_oplist,
                input: PendingOpInput {
                    dimension_ref: work.dim_ref,
                    gpos,
                },
                purpose: PendingOpPurpose::ChunkTerrainGen {
                    chunk_pos: work.chunk_pos,
                },
            });
        });
    }
    batch
}


pub(crate) fn process_pending_ops_batch(
    pending_ops: Vec<PendingOp>,
    context: TerrGenTaskContext,
    gen_settings: GlobalGenSettings,
    capture_debug: bool,
) -> TerrGenOpTaskResult {
    use crate::terrain::terrgen_expression::EvalContext;

    let mut result = TerrGenOpTaskResult::default();
    let mut pending_queue = pending_ops;
    let mut emitted_per_probe: EntityHashMap<u32> = EntityHashMap::new();
    let mut last_success_idx_for_requester: EntityHashMap<usize> = EntityHashMap::new();
    let mut sampled_matrices_by_requester: EntityHashMap<SampledValues> = EntityHashMap::new();

    while let Some(ev) = pending_queue.pop() {
        upsert_sample_matrix_for_pending(&ev, &mut sampled_matrices_by_requester);
        let root_oplist_hash = ev.oplist.0;
        if !context.oplists.contains_key(root_oplist_hash) {
            error!(target: "terrgen_systems", "Oplist hash {:?} not found in terrgen_process_pending_ops", root_oplist_hash);
            continue;
        }
        let Ok(&my_oplist_size) = context.oplist_sizes.get(root_oplist_hash) else {
            error!(target: "terrgen_systems", "OplistSize not found for oplist hash {:?}", root_oplist_hash);
            continue;
        };

        let dimension_hash = ev.dimension_ref().0;
        let filtered_op = ev.filtered_op();
        let filter = if filtered_op != HashId::default() {
            context.filters.get_opt(filtered_op)
        } else {
            None
        };

        let mut frame_stack = vec![EvalFrame {
            oplist: root_oplist_hash,
            gpos: ev.gpos(),
            oplist_size: my_oplist_size,
            variables: HashIdMap::new(),
        }];

        while let Some(mut frame) = frame_stack.pop() {
            let Ok(oplist) = context.oplists.get(frame.oplist) else { continue; };
            if oplist.bifurcations.is_empty() {
                continue;
            }
            let Ok(oplist_id) = context.oplist_ids.get(frame.oplist) else { continue; };
            let eval_context = EvalContext {
                global_pos: frame.gpos,
                dimension_hash,
                gen_settings: &gen_settings,
                oplist_size: frame.oplist_size,
                oplist_id,
                noises: &context.noises,
                variables: &frame.variables,
            };
            let (output_value, computed_vars) = oplist.expr_tree.eval(&frame.variables, &eval_context);
            frame.variables = computed_vars;
            if capture_debug {
                push_debug_sample(
                    &mut result,
                    &context,
                    ev.dimension_ref(),
                    frame.gpos,
                    frame.oplist,
                    output_value,
                    &frame.variables,
                );
            }

            let destination_i = (output_value as usize).min(oplist.bifurcations.len() - 1);
            try_emit_filter_match(
                &ev,
                &context,
                frame.oplist,
                frame.gpos,
                output_value,
                &frame.variables,
                filter,
                &mut emitted_per_probe,
                &mut last_success_idx_for_requester,
                &mut sampled_matrices_by_requester,
                &mut result,
            );

            let Some(bifurcation) = oplist.bifurcations.get(destination_i) else { continue; };
            collect_branch_outputs(
                &mut result,
                &ev,
                frame.oplist,
                frame.gpos,
                frame.oplist_size,
                dimension_hash,
                &bifurcation.biome_tags,
                &bifurcation.tiles,
            );

            if let Some(child_oplist_hash) = bifurcation.oplist
                && let Ok(&child_oplist_size) = context.oplist_sizes.get(child_oplist_hash)
            {
                frame.spawn_bifurcation_frames(&mut frame_stack, child_oplist_hash, child_oplist_size);
            }
        }
        result.mark_pending_op_complete(&ev);
    }

    for (_, sample_idx) in last_success_idx_for_requester.drain() {
        let Some(sample) = result.sampled_value_events.get_mut(sample_idx) else { continue; };
        sample.is_last = true;
    }
    for (requester, matrix) in sampled_matrices_by_requester.drain() {
        result.sampled_value_matrix_events.push(SampledValuesCollected {
            requester,
            matrix,
        });
    }
    result
}
