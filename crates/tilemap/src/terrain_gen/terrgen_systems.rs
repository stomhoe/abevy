


use bevy::{ecs::{entity::{EntityHashSet, }, entity_disabling::Disabled}, platform::collections::{HashMap, HashSet}, prelude::*};
use common::{common_components::{AnyDisabling, DisplayName, HashId, StrId}, common_tag_components::HashedTagsVec, };
use debug_unwraps::DebugUnwrapExt;
use dimension_shared::{DimensionRef, DimensionRootOplist};
use game_common::{game_common_components::*, game_common_components_samplers::EntityWeightedSampler};
use crate::{chunking_components::*, chunking_resources::{AaChunkRangeSettings, LoadedChunks}, terrain_gen::{terrgen_components::*, terrgen_messages::*, terrgen_oplist_components::*, terrgen_resources::*}, tile::tile_components::*, tilemap_resources::MassCollectedTiles };
use std::{f32::consts::PI, };
use ::tilemap_shared::*;

#[allow(unused_parens)]
pub fn launch_terrain_gen_operations (
    mut commands: Commands, 
    chunks_query: Query<(Entity, &ChunkPos, &DimensionRef), (Without<TerrGenOpsLaunched>, With<Chunk>, With<ReadyForTerrgen>)>, 
    dimension_query: Query<(&DimensionRootOplist, &HashId), ()>,
    oplists: Query<(Entity, &OplistSize), (With<OperationList>, )>,
    mut ew_pending_ops: MessageWriter<PendingOp>,

) -> Result {
    if chunks_query.is_empty() { return Ok(()); }

    let chunk_area = chunks_query.iter().len() * ChunkPos::CHUNK_SIZE.element_product() as usize * 4;
    let mut batch = Vec::with_capacity(chunk_area);
    chunks_query.iter().for_each(|(chunk_ent, chunk_pos, &dim_ref)| {
        let Ok((dim_root_op_list, hash_id)) = dimension_query.get(dim_ref.0) else {
            error!("No root operation list for chunk {:?} in dimension {:?}", chunk_pos, dim_ref);
            return;
        };
        let Ok((oplist, oplist_size)) = oplists.get(dim_root_op_list.0) else {
            error!("Dimension references non-existent root operation list {:?}", dim_root_op_list);
            return;
        };
        for x in 0..ChunkPos::CHUNK_SIZE.x / oplist_size.x() {
            for y in 0..ChunkPos::CHUNK_SIZE.y / oplist_size.y() {
                let pos_within_chunk = IVec2::new(x as i32, y as i32);
                let gpos = chunk_pos.to_tilepos() + GlobalTilePos(pos_within_chunk * oplist_size.inner().as_ivec2());
                trace!(
                    "Spawning terrain operation {:?} at {:?} in chunk {:?}, pos_within_chunk: {:?}, oplist_size: {:?}",
                    oplist,
                    gpos,
                    chunk_ent,
                    pos_within_chunk,
                    oplist_size
                );
                if commands.get_entity(chunk_ent).is_err() {
                    return;
                }
                batch.push(PendingOp {
                    oplist, dim_ref, gpos, dimension_hash_id: hash_id.as_i32(), 
                    variables: VariablesArray::default(), filtered_op: Entity::PLACEHOLDER,
                });
            }
        }
        if commands.get_entity(chunk_ent).is_err() { return; }
        commands.entity(chunk_ent).try_insert(TerrGenOpsLaunched);
    });   
    ew_pending_ops.write_batch(batch);
    Ok(())
}
#[allow(unused_parens)]
/// input: PendingOp messages. output: PendingOp messages (for bifurcations), SuitablePosFound messages
pub fn process_pending_ops_and_collect_tiles(mut cmd: Commands, 
    mut pending_ops_events: ResMut<Messages<PendingOp>>,
    gen_settings: Single<&GlobalGenSettings>,
    oplist_query: Query<(&OperationList, &OplistSize, Option<&HashedTagsVec>), ( )>,
    fnl_noises: Query<&FnlNoiseComp,>,
    op_filters: Query<&OpFilter,>,
    weight_maps: Query<(&EntityWeightedSampler, ), ( )>,
    mut collected: ResMut<MassCollectedTiles>,
    mut ewriter_sampled_value: MessageWriter<SuitablePosFound>,
) {
    if pending_ops_events.is_empty() { return; }

    let mut new_pending_ops_events = Vec::with_capacity(pending_ops_events.len());
    let mut sampled_value_events = Vec::new();

    for mut ev in pending_ops_events.drain() {unsafe{
   
        let Ok((oplist, &my_oplist_size, oplist_tags)) = oplist_query.get(ev.oplist)
        else {
            error!("Oplist entity {:?} not found in terrgen_process_pending_ops", ev.oplist);
            continue;
        };
        let global_pos = ev.gpos;
        
        oplist.trunk.iter().enumerate().for_each(|(op_i, (operation, operands, stackarr_out_i))| {
            let mut operation_acc_val: f32 = 0.0;
            let mut selected_operand_i = 0; 

            for (operand_i, operand) in operands.iter().enumerate() {
                let mut curr_operand_val = match &operand.element {
                    OperandElement::StackArray(i) => ev.variables[*i],
                    OperandElement::Value(val) => *val,
                    OperandElement::NoiseEntity(ent, sample_range, compl, operand_seed) => {
                        let Ok(noise) = fnl_noises.get(*ent) else {
                            error!("Noise entity {} not found", ent);
                            return;
                        };
                        let seed = operand_seed.wrapping_add(ev.dimension_hash_id);
                        noise.sample(global_pos, *sample_range, *compl, seed, &gen_settings)   
                    },
                    OperandElement::HashPos(seed) => global_pos.normalized_hash_value(&gen_settings, *seed) as f32,
                    OperandElement::PoissonDisk(poisson_disk) => poisson_disk.sample(&gen_settings, global_pos, true, my_oplist_size) as f32,
                };
                if operand.complement && !matches!(operand.element, OperandElement::NoiseEntity(_, _, _, _)) {
                    curr_operand_val = 1.0 - curr_operand_val;
                }
                let is_last = operand_i == operands.len() - 1;   
                let prev_value = operation_acc_val;

                match (operation, operand_i, is_last) {
                    (Operation::Add, 1.., _) => operation_acc_val += curr_operand_val,
                    (Operation::Subtract, 1.., _) => operation_acc_val -= curr_operand_val,
                    (Operation::Multiply, 1.., _) => operation_acc_val *= curr_operand_val,
                    (Operation::MultiplyOpo, 1.., _) => operation_acc_val *= (1.0 - curr_operand_val),
                    (Operation::Divide, 1.., _) => if curr_operand_val != 0.0 { operation_acc_val /= curr_operand_val },
                    (Operation::Min, 1.., _) => operation_acc_val = operation_acc_val.min(curr_operand_val),
                    (Operation::Max, 1.., _) => operation_acc_val = operation_acc_val.max(curr_operand_val),
                    (Operation::Average, _, false) => {operation_acc_val += curr_operand_val;},
                    (Operation::Average, _, true) => {operation_acc_val += curr_operand_val; operation_acc_val /= operands.len() as f32;},
                    (Operation::Linear, 0, _) => {operation_acc_val = curr_operand_val; trace!("conti: {}", curr_operand_val)},
                    (Operation::Linear, 1, _) => {operation_acc_val *= curr_operand_val; trace!("beach: {}", curr_operand_val)},
                    (Operation::Linear, 2, _) => {operation_acc_val += curr_operand_val;},
                    (Operation::Linear, 3.., _) => {operation_acc_val *= curr_operand_val; trace!("res: {}", operation_acc_val); },
                    (Operation::MultiplyNormalized, 1.., _) => operation_acc_val *= (curr_operand_val - 0.5) * 2.,
                    (Operation::MultiplyNormalizedAbs, 1.., _) => operation_acc_val *= ((curr_operand_val - 0.5) * 2.).abs(),
                    (Operation::Abs, _, _) => {operation_acc_val = operation_acc_val.abs();},
                    (Operation::i_Max, 0, _) => { operation_acc_val = curr_operand_val; }
                    (Operation::i_Max, _, false) => {if curr_operand_val > operation_acc_val 
                        { operation_acc_val = curr_operand_val; selected_operand_i = operand_i; }}
                    (Operation::i_Max, _, true) => {if curr_operand_val > operation_acc_val
                        { selected_operand_i = operand_i; } operation_acc_val = selected_operand_i as f32;}
                    (Operation::i_Norm, 0, false) => { operation_acc_val = curr_operand_val; }
                    (Operation::i_Norm, 0, true) => { operation_acc_val = curr_operand_val * (oplist.bifurcations.len() - 1) as f32; }
                    (Operation::i_Norm, _, false) => { operation_acc_val *= curr_operand_val; }
                    (Operation::i_Norm, 1.., true) => { operation_acc_val *= curr_operand_val * (oplist.bifurcations.len() - 1) as f32; }
                    (Operation::Clamp, 0, true) => {operation_acc_val = curr_operand_val.max(0.0).min(1.0);},
                    (Operation::Clamp, 0, _) => {operation_acc_val = curr_operand_val;},
                    (Operation::Clamp, 1, false) => {operation_acc_val = curr_operand_val.max(operation_acc_val);},
                    (Operation::Clamp, 1, true) => {operation_acc_val = curr_operand_val.max(operation_acc_val).min(1.0);},
                    (Operation::Clamp, 2.., _) => {operation_acc_val = curr_operand_val.min(operation_acc_val);},
                    (_, 0, _) => {operation_acc_val = curr_operand_val;},
                }

                trace!(
                    "{} with operand {:?} at stack array index {}: prev_value: {}, curr_value: {}, {:?},",
                    operation, operand, *stackarr_out_i, prev_value, operation_acc_val, global_pos,
                );      
            }

            trace!("Operation result for stack array index {}: {}", *stackarr_out_i, operation_acc_val);
            ev.variables[*stackarr_out_i] = operation_acc_val;

            if (ev.filtered_op != Entity::PLACEHOLDER)  {
                if let Ok(ref mut filter) = op_filters.get(ev.filtered_op) {

                    let Some(oplist_tags) = oplist_tags else {
                        return;
                    };
                    let filter_tags = &filter.tags;
                    
                    let sampling_last_and_is_so = filter.op_i <= -1 && op_i == oplist.trunk.len() - 1;
                    

                    if oplist_tags.intersects(filter_tags) && (op_i == filter.op_i as usize 
                        || sampling_last_and_is_so)
                    {    
                        if (filter.min_val <= operation_acc_val && operation_acc_val <= filter.max_val)
                        {
                            sampled_value_events.push(SuitablePosFound {
                                op_filter_ent: ev.filtered_op,
                                val: operation_acc_val,
                                found_pos: ev.gpos,
                            });
                        }
                        return;
                    } 
                }
                else{
                    trace!(target: "terrgen_process", "Failed to get OpFilter of entity {:?}", ev.filtered_op);
                    return;
                }
            }
        });

        let destination_i = (ev.variables[0] as usize).min(oplist.bifurcations.len() - 1).max(0);   
        trace!("Destination index for bifurcation: {}", destination_i);

        let bifurcation = oplist.bifurcations.get(destination_i).debug_unwrap_unchecked();
        
        if let Some(child_oplist) = bifurcation.oplist {
            let (_, &child_oplist_size, _) = oplist_query.get(child_oplist).debug_expect_unchecked("OplistSize not found");

            spawn_bifurcation_oplists(&mut ev, my_oplist_size, &mut new_pending_ops_events, child_oplist, child_oplist_size);
        }

        if bifurcation.tiles.len() > 0 && ev.filtered_op == Entity::PLACEHOLDER {
            collected.collect_tiles(&mut cmd, &bifurcation.tiles, &ev, my_oplist_size, &weight_maps, &gen_settings);
        }
    }}
    pending_ops_events.write_batch(new_pending_ops_events); 
    ewriter_sampled_value.write_batch(sampled_value_events);
}

fn spawn_bifurcation_oplists(
    ev: &mut PendingOp, my_oplist_size: OplistSize, 
    new_pending_ops: &mut Vec<PendingOp>, child_oplist: Entity, child_oplist_size: OplistSize,
) {
    if my_oplist_size <= child_oplist_size
    {
        if ev.gpos.0.abs().as_uvec2() % child_oplist_size.inner() == UVec2::ZERO
        {
            new_pending_ops.push(PendingOp{ oplist: child_oplist, ..(*ev).clone()  });
        }
    } 
    else{
        let x_end = my_oplist_size.x() as i32 / child_oplist_size.x() as i32; 
        let y_end = my_oplist_size.y() as i32 / child_oplist_size.y() as i32;
        for x in 0..x_end {
            for y in 0..y_end {
                let gpos = ev.gpos + GlobalTilePos::new(x, y);

                new_pending_ops.push(PendingOp{ gpos, oplist: child_oplist, ..(*ev).clone()  });
            }
        }
    }
}



#[allow(unused_parens)]
//input: PosSearch messages. output: SearchFailed or SuitablePosFound(emitted in produce_tiles) 
pub fn search_suitable_positions(
    mut cmd: Commands,
    mut terrain_probe: ResMut<Messages<TerrainProbe>>, mut mwriter_search_failed: MessageWriter<SearchFailed>,
    mut mwriter_pending_ops: MessageWriter<PendingOp>, mut mreader_suitable_pos_found: MessageReader<SuitablePosFound>,
    studied_ops: Query<&OpFilter, ( )>,
    failed_search_oplist_filter_holder: Single<Entity, (With<FailedSearchOplistFilterHolder>)>,
) {
    let mut new_pending_ops = Vec::new();
    let mut new_pos_searches = Vec::new();
    let mut search_failed_evs = Vec::new();
    let mut found_suitable_positions = EntityHashSet::new();

    for found_ev in mreader_suitable_pos_found.read() {
        found_suitable_positions.insert(found_ev.op_filter_ent);
    }
    
    for pos_search in terrain_probe.drain() {

        if found_suitable_positions.contains(&pos_search.operation_filter) {
            info!(target: "pos_search","Found suitable position for {:?}", pos_search.operation_filter);
            continue;
        }

        let (filtered_op, step_size, curr_iteration_batch_i, iterations_per_batch, max_batches, dimension_hash_id) = 
        (pos_search.operation_filter, pos_search.step_size, pos_search.curr_iteration_batch_i, pos_search.iterations_per_batch, pos_search.max_batches, pos_search.dimension_hash_id);

        let Ok(opfilter) = studied_ops.get(filtered_op) else {//ERRROR: ENTTIY NO SPAWNEÓ TODAVÍA
            if curr_iteration_batch_i == 0 {
                // If we want to retry, push a new PosSearch with decremented batch index
                let mut new_search = pos_search;
                new_search.curr_iteration_batch_i -= 1;
                new_pos_searches.push(new_search);
            } else if curr_iteration_batch_i == -2 {
                error!(target: "pos_search", "StudiedOp entity {:?} not found in search_suitable_position, giving up", filtered_op);
                search_failed_evs.push(SearchFailed(filtered_op));
            }
            continue;
        };
        let curr_iteration_batch_i = curr_iteration_batch_i.max(0);

        match pos_search.probe_pattern {
            ProbePattern::Radial(explore_angle) => {

                let calculate_pos = |i_within_batch: u16, probe_direction: f32| -> GlobalTilePos {
                    let global_i = (curr_iteration_batch_i as u16 * iterations_per_batch as u16 + i_within_batch) as f32 * step_size as f32;
                    opfilter.search_start_pos + GlobalTilePos::from(IVec2::new(
                    (global_i * probe_direction.cos()) as i32, (global_i * probe_direction.sin()) as i32,
                    ))
                };

                if let Some(explore_angle) = explore_angle {

                    let start_i_within_batch = (curr_iteration_batch_i == 0) as u16;

                    for i_within_batch in start_i_within_batch..iterations_per_batch {
                        new_pending_ops.push(PendingOp {
                            oplist: opfilter.start_oplist,
                            dimension_hash_id,
                            gpos: calculate_pos(i_within_batch, explore_angle),
                            filtered_op,
                            variables: VariablesArray::default(),
                            dim_ref: DimensionRef(Entity::PLACEHOLDER),
                        });
                    }
                    if curr_iteration_batch_i as u16 + 1 < max_batches {
                        new_pos_searches.push(TerrainProbe {
                            curr_iteration_batch_i: curr_iteration_batch_i + 1,
                            probe_pattern: ProbePattern::Radial(Some(explore_angle)),
                            ..pos_search
                        });
                    } else {
                        error!(target: "pos_search", "No more batches to search for {:?}", opfilter);
                        search_failed_evs.push(SearchFailed(filtered_op));
                    }
                } else {
                    if curr_iteration_batch_i as u16 >= max_batches {
                        error!(target: "pos_search", "curr No more batches to search for {:?}", pos_search);
                        continue;
                    }
                    let divisions = 8;
                    for i in 0..divisions {
                        let angle = 2.0 * PI * (i as f32) / (divisions as f32);
                        new_pos_searches.push(TerrainProbe{
                            probe_pattern: ProbePattern::Radial(Some(angle)),
                            ..pos_search
                        });
                    }
                }
            }
            ProbePattern::Spiral(mut curr_length_in_dir, mut steps_taken, mut dir_vec, mut pos, mut turn_parity) => {
                trace!(target: "pos_search", "Spiral search started at pos {:?}, dir_vec {:?}, curr_length_in_dir {}, turns {}", 
                    pos, dir_vec, curr_length_in_dir, turn_parity);

                for _ in 0..iterations_per_batch {
                    pos = pos + GlobalTilePos(dir_vec.saturating_mul(IVec2::splat(step_size as i32)));  
                     new_pending_ops.push(PendingOp {
                        dimension_hash_id,
                        oplist: opfilter.start_oplist,
                        dim_ref: DimensionRef(Entity::PLACEHOLDER),
                        gpos: pos,
                        variables: VariablesArray::default(),
                        filtered_op,
                    });

                    steps_taken += 1;
                    if steps_taken >= curr_length_in_dir {
                        steps_taken = 0;
                        
                        dir_vec = dir_vec.perp();
                        curr_length_in_dir = curr_length_in_dir.saturating_add(turn_parity as u64);
                        turn_parity = !turn_parity;
                    }
                }
                if curr_iteration_batch_i as u16 + 1 < max_batches {
                    new_pos_searches.push(TerrainProbe{
                        curr_iteration_batch_i: curr_iteration_batch_i + 1,
                        probe_pattern: ProbePattern::Spiral(curr_length_in_dir, steps_taken, dir_vec, pos, turn_parity),
                        ..pos_search
                    });
                } else {
                    error!(target: "pos_search", "No more batches to search for {:?}", opfilter);
                    cmd.entity(filtered_op).try_insert(ChildOf(failed_search_oplist_filter_holder.entity()));
                    search_failed_evs.push(SearchFailed(filtered_op));
                }
            },   
        }
    }
    mwriter_pending_ops.write_batch(new_pending_ops);
    terrain_probe.write_batch(new_pos_searches);
    mwriter_search_failed.write_batch(search_failed_evs);
}



