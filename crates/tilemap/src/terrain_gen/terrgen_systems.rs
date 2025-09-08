


use bevy::{ecs::{entity::{EntityHashSet, MapEntities}, entity_disabling::Disabled}, platform::collections::{HashMap, HashSet}, prelude::*};
use bevy_ecs_tilemap::tiles::TilePos;
use bevy_replicon::{prelude::{Replicated, SendMode, ToClients}, shared::server_entity_map::ServerEntityMap};
use common::{common_components::{DisplayName, HashId, StrId}, common_states::GameSetupType};
use debug_unwraps::DebugUnwrapExt;
use dimension_shared::{Dimension, DimensionRef, DimensionRootOplist, MultipleDimensionRefs, RootInDimensions};
use game_common::{game_common_components::{EntiZeroRef, FacingDirection }, game_common_components_samplers::EntiWeightedSampler};
use crate::{chunking_components::*, chunking_resources::{AaChunkRangeSettings, LoadedChunks}, terrain_gen::{terrgen_components::*, terrgen_events::*, terrgen_oplist_components::*, terrgen_resources::*}, tile::{tile_components::*, } };
use std::{f32::consts::PI, mem::take};
use ::tilemap_shared::*;


// HACER Q CADA UNA DE ESTAS ENTITIES APAREZCA EN LOS SETTINGS EN SETUP Y SEA CONFIGURABLE

// PARA HACER ISLAS CON FORMA CUSTOM (P. EJ CIRCULAR O DISCO O ALGO RARO Q NO SE PUEDE HACER CON NOISE), MARCAR EN UN PUNTO EXTREMADAMENTE OCÉANICO CON UNA TILE MARKER Y DESP HACER OTRO SISTEMA Q LO PONGA TODO POR ENCIMA, SOBREESCRIBIENDO LO Q HABÍA ANTES


#[allow(unused_parens)]
pub fn spawn_terrain_operations (
    mut commands: Commands, 
    res_chunk: Res<AaChunkRangeSettings>,
    chunks_query: Query<(Entity, &ChunkPos, &ChildOf), (Without<OperationsLaunched>, )>, 
    dimension_query: Query<(&DimensionRootOplist, &HashId), ()>,
    oplists: Query<(Entity, &OplistSize), (With<OperationList>, )>,
    mut ew_pending_ops: EventWriter<PendingOp>,

) -> Result {
    if chunks_query.is_empty() { return Ok(()); }

    let chunk_area = res_chunk.approximate_number_of_tiles() * 4;
    let mut batch = Vec::with_capacity(chunk_area * 4);
    'chunk_for: for (chunk_ent, chunk_pos, dim_ref) in chunks_query.iter() {
        //SE LES PODRÍA AGREGAR MARKER COMPONENTS A LOS CHUNKS PARA POR EJEMPLO ESPECIFICAR SI ES UN DUNGEON

//PONER MARKERS A TODAS LAS POSICIONES SUITABLE, DESPUES HACER UNA QUERY Q COMPARA LAS TILESMARCADAS COMO Q YA GENERARON UNA ESTRUCTURA NO PROCEDURAL CON LAS Q NO. SI LA DISTANCIA ES SUFICIENTE, SPAWNEAR UNA EN LA SIGUIENTE
        //EN ESTE PUNTO SE PODRÍA GENERAR UN CAMINO RANDOM QUE SEA UN VEC DE COORDS, Y DESPUES PASARLO ABAJO Y Q SE OCUPEN?? PA GENERAR DUNGEONS NASE


        let Ok((dim_root_op_list, hash_id)) = dimension_query.get(dim_ref.0) else {
            error!("No root operation list for chunk {:?} in dimension {:?}", chunk_pos, dim_ref);
            continue;
        };

        let Ok((oplist, oplist_size)) = oplists.get(dim_root_op_list.0) else {
            error!("Dimension references non-existent root operation list {:?}", dim_root_op_list);
            continue;
        };

        for x in 0..ChunkPos::CHUNK_SIZE.x / oplist_size.x() {
            for y in 0..ChunkPos::CHUNK_SIZE.y / oplist_size.y() {
                let pos_within_chunk = IVec2::new(x as i32, y as i32);
                let global_pos = chunk_pos.to_tilepos() + GlobalTilePos(pos_within_chunk * oplist_size.inner().as_ivec2());
                trace!(
                    "Spawning terrain operation {:?} at {:?} in chunk {:?}, pos_within_chunk: {:?}, oplist_size: {:?}",
                    oplist,
                    global_pos,
                    chunk_ent,
                    pos_within_chunk,
                    oplist_size
                );
                if commands.get_entity(chunk_ent).is_err() {
                    continue 'chunk_for;
                }
                batch.push(PendingOp {
                    oplist, chunk_ent, pos: global_pos, dimension_hash_id: hash_id.into_i32(), 
                    variables: VariablesArray::default(), studied_op_ent: Entity::PLACEHOLDER,
                });
            }
        }
        if commands.get_entity(chunk_ent).is_err() {continue 'chunk_for;}

        commands.entity(chunk_ent).try_insert(OperationsLaunched);

    }
    ew_pending_ops.write_batch(batch);
    Ok(())
}



#[allow(unused_parens)]
pub fn produce_tiles(mut cmd: Commands, 
    gen_settings: Res<AaGlobalGenSettings>,
    res_chunk: Res<AaChunkRangeSettings>,
    oplist_query: Query<(&OperationList, &OplistSize ), ( )>,
    mut pending_ops_events: ResMut<Events<PendingOp>>,
    fnl_noises: Query<&FnlNoise,>,
    studied_ops: Query<&StudiedOp,>,
    weight_maps: Query<(&EntiWeightedSampler, ), ( )>,
    mut ewriter_instantiated_tiles: EventWriter<InstantiatedTiles>,
    mut ewriter_sampled_value: EventWriter<SuitablePosFound>,
) -> Result {

    if pending_ops_events.is_empty() { return Ok(()); }

    let chunk_area = res_chunk.approximate_number_of_tiles() * 2;

    let mut new_pending_ops_events = Vec::with_capacity(chunk_area);
    let mut produced_tiles_events = Vec::with_capacity(chunk_area);
    let mut sampled_value_events = Vec::new();

    'eventfor: for mut ev in pending_ops_events.drain() {unsafe{
   
        let (oplist, &my_oplist_size) = oplist_query.get(ev.oplist)?;
        let global_pos = ev.pos;
        

        for (op_i, (operation, operands, stackarr_out_i)) in oplist.trunk.iter().enumerate() {
            let mut operation_acc_val: f32 = 0.0;
            let mut selected_operand_i = 0; 

            for (operand_i, operand) in operands.iter().enumerate() {
                let mut curr_operand_val = match &operand.element {
                    OperandElement::StackArray(i) => ev.variables[*i],
                    OperandElement::Value(val) => *val,
                    OperandElement::NoiseEntity(ent, sample_range, compl, operand_seed) => {
                        match fnl_noises.get(*ent) {
                            Ok(noise) => noise.sample(global_pos, *sample_range, *compl, *operand_seed + ev.dimension_hash_id, &gen_settings),
                            Err(_) => {
                                error!("Entity {} not found in terrgens", ent);
                                continue;
                            }
                        }
                    },
                    OperandElement::HashPos(seed) => global_pos.normalized_hash_value(&gen_settings, *seed),
                    OperandElement::PoissonDisk(poisson_disk) => poisson_disk.sample(&gen_settings, global_pos, my_oplist_size),
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
                    (Operation::Abs, _, _) => {operation_acc_val = operation_acc_val.abs(); break;},
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

            if let Ok(ref mut sop) = studied_ops.get(ev.studied_op_ent) {
                if  sop.checked_oplist == ev.oplist && (op_i == sop.op_i as usize || (sop.op_i <= -1 && op_i == oplist.trunk.len() - 1))
                {    
                    if (sop.lim_below <= operation_acc_val && operation_acc_val <= sop.lim_above)
                    {
                        sampled_value_events.push(SuitablePosFound {
                            studied_op_ent: ev.studied_op_ent,
                            val: operation_acc_val,
                            found_pos: ev.pos,
                        });
                    }
                    continue 'eventfor;
                }
            }
            trace!("Operation result for stack array index {}: {}", *stackarr_out_i, operation_acc_val);
            ev.variables[*stackarr_out_i] = operation_acc_val;

        }
        let destination_i = (ev.variables[0] as usize).min(oplist.bifurcations.len() - 1).max(0);
        trace!("Destination index for bifurcation: {}", destination_i);

        let bifurcation = oplist.bifurcations.get(destination_i).debug_unwrap_unchecked();
        
        if let Some(oplist) = bifurcation.oplist {
            spawn_bifurcation_oplists(&mut ev, &oplist_query, &mut new_pending_ops_events, oplist, my_oplist_size);
        }

        if bifurcation.tiles.len() > 0 && ev.studied_op_ent == Entity::PLACEHOLDER {
            let tiles = InstantiatedTiles::from_op(&mut cmd, &ev, &bifurcation.tiles, my_oplist_size, &weight_maps, &gen_settings);
            produced_tiles_events.push(tiles);
        }
    }}
    pending_ops_events.send_batch(new_pending_ops_events); 
    ewriter_instantiated_tiles.write_batch(produced_tiles_events);
    ewriter_sampled_value.write_batch(sampled_value_events);

    Ok(())
}




fn spawn_bifurcation_oplists(
    ev: &mut PendingOp, oplist_query: &Query<(&OperationList, &OplistSize), ()>,
    new_pending_ops: &mut Vec<PendingOp>, oplist: Entity, my_oplist_size: OplistSize,
) {unsafe{
    let (_, &child_oplist_size) = oplist_query.get(oplist).debug_expect_unchecked("OplistSize not found");

    if my_oplist_size != child_oplist_size
    && (ev.pos.0.abs().as_uvec2() % child_oplist_size.inner() == UVec2::ZERO)
    && ev.studied_op_ent == Entity::PLACEHOLDER
    {
        let x_end = child_oplist_size.x() as i32; let y_end = child_oplist_size.y() as i32;
        for x in 0..x_end {
            for y in 0..y_end {
                let pos = ev.pos + GlobalTilePos::new(x, y);

                new_pending_ops.push(PendingOp{ oplist, pos, ..(*ev).clone()  });
            }
        }
    } else {new_pending_ops.push(PendingOp{ oplist, ..(*ev).clone() });}
}}


#[allow(unused_parens)]
pub fn process_tiles(mut cmd: Commands, 
    mut events_instantiated_tiles: ResMut<Events<InstantiatedTiles>>,
    mut ewriter_processed_tiles: EventWriter<Tiles2TmapProcess>,
    chunk_query: Query<(&ChildOf), ()>,
    dimension_query: Query<(Entity), (With<Dimension>, )>,
    mut tile_query: Query<(&EntiZeroRef, &GlobalTilePos, &mut DimensionRef, Option<&mut Transform>, ), (With<Tile>, Or<(With<Disabled>, Without<Disabled>)>, )>,

    oritile_query: Query<(Has<ChunkOrTilemapChild>, Option<&MinDistancesMap>, Option<&KeepDistanceFrom>), (With<Disabled>)>,
    min_dists_query: Query<(&MinDistancesMap), (With<Disabled>)>,
    mut regpos_map: ResMut<RegisteredPositions>,

    state: Res<State<GameSetupType>>,
) {
    let is_host = state.get() != &GameSetupType::AsJoiner;

    //TODO: insert_with_bundle vec<entity, bundle> en vez de ir insertando uno por uno
    
    if events_instantiated_tiles.is_empty() { return; }

    let child_ofs_capacity = events_instantiated_tiles.len() /100;
    trace!("Processing {} instantiated tiles events, reserving space for {} ChildOf components", events_instantiated_tiles.len(), child_ofs_capacity);
    //TODO HACER EVENTOS ESPECIALES?

    let mut instantiated_tiles_events_to_retransmit = Vec::with_capacity(events_instantiated_tiles.len());
    let mut processed_tiles_events = Vec::with_capacity(events_instantiated_tiles.len());
    let mut to_insert_replicated = Vec::new();


    'eventfor: for mut ev in events_instantiated_tiles.drain() {

        let tiles_len = ev.tiles.len();

        'tilefor: for (tile_i, tile_ent) in ev.tiles.iter_mut().enumerate() {

            let Ok((&ezero, &global_pos, mut placeholder_dim_ref, transform, )) = tile_query.get_mut(*tile_ent)
            else {
                ev.retransmission_count += 1;
                if ev.retransmission_count >= 10 as u16 {
                    error!("Tile entity {:?} in instantiated tiles event for chunk or dim {:?} does not exist or isn't a Tile", tile_ent, ev.chunk_or_dim);
                }else{
                    instantiated_tiles_events_to_retransmit.push(ev);
                }
                continue 'eventfor; 
            };

            let Ok((chunk_child, min_dists, keep_distance_from)) = oritile_query.get(ezero.0)
            else { 
                error!("{:?}'s Tileref{:?} does not exist or doesnt't have Disabled component", tile_ent, ezero.0);
                cmd.entity(*tile_ent).try_despawn(); 
                *tile_ent = Entity::PLACEHOLDER;
                continue 'tilefor; 
            };

            let dim_ent = if let Ok((chunk_childof)) = chunk_query.get(ev.chunk_or_dim) { 
                if (chunk_child == false) {
                    trace!("Got chunk's ChildOf for chunk {:?} for tile {:?}: {:?}", ev.chunk_or_dim, tile_ent, chunk_childof.parent());
                }
                chunk_childof.parent() 
            } 
            else if (chunk_child == false){
                let Ok(dim_ent) = dimension_query.get(ev.chunk_or_dim) else {
                    error!("Failed to get dimension {:?} for orphan tile {:?}", ev.chunk_or_dim, tile_ent);
                    cmd.entity(*tile_ent).try_despawn(); 
                    *tile_ent = Entity::PLACEHOLDER;
                    if tile_i == tiles_len - 1 { continue 'eventfor; }
                    if tile_i == 0{
                        trace!("PROCESSTILES Failed to get chunk's ChildOf for chunk {:?}", ev.chunk_or_dim);
                    }
                    continue 'tilefor; 
                };
                info!("Got dimension {:?} for orphan tile {:?}", ev.chunk_or_dim, tile_ent);
                dim_ent
            } else{
                cmd.entity(*tile_ent).try_despawn(); 
                *tile_ent = Entity::PLACEHOLDER;
                
                if tile_i == tiles_len - 1 { continue 'eventfor; }
                if tile_i == 0{
                    trace!("PROCESSTILES Failed to get chunk's ChildOf for chunk {:?}", ev.chunk_or_dim);
                }
                
                continue 'tilefor; 

            };
            let dimref = DimensionRef(dim_ent); *placeholder_dim_ref = dimref;

            if false == regpos_map.check_min_distances(&mut cmd, is_host, (tile_ent.clone(), ezero, dimref, global_pos, min_dists, keep_distance_from), min_dists_query) {
                cmd.entity(*tile_ent).try_despawn(); 
                *tile_ent = Entity::PLACEHOLDER;
                continue 'tilefor; 
            }
            
            trace!("Spawned tile {:?} at global pos {:?} in dimension {:?}", tile_ent, global_pos, dimref);

            if transform.is_some() {
                cmd.entity(*tile_ent).try_remove::<(Disabled, TilePos, OplistSize, )>();
                
                if chunk_child  {
                    cmd.entity(*tile_ent).try_insert((ChildOf(ev.chunk_or_dim), ));
                }
            }
            
            if ! chunk_child  {
                if is_host {
                    cmd.entity(*tile_ent).try_insert((ChildOf(dim_ent)));
                    to_insert_replicated.push((*tile_ent, Replicated));
                }
                else{
                    cmd.entity(*tile_ent).try_despawn();
                    *tile_ent = Entity::PLACEHOLDER;
                }
            } 
                
        }
        let protiles = Tiles2TmapProcess { chunk: ev.chunk_or_dim, tiles: ev.take_tiles() };
        processed_tiles_events.push(protiles);
        
    }
    cmd.insert_batch(to_insert_replicated);

    ewriter_processed_tiles.write_batch(processed_tiles_events);
    events_instantiated_tiles.send_batch(instantiated_tiles_events_to_retransmit);
}





#[allow(unused_parens)]
pub fn search_suitable_position(
    mut cmd: Commands,
    mut events_pos_search: ResMut<Events<PosSearch>>, mut ewriter_search_failed: EventWriter<SearchFailed>,
    mut ewriter_pending_ops: EventWriter<PendingOp>, mut ereader_suitable_pos_found: EventReader<SuitablePosFound>,
    studied_ops: Query<&StudiedOp, ( )>,
) {
    let mut new_pending_ops = Vec::new();
    let mut new_pos_searches = Vec::new();
    let mut search_failed_evs = Vec::new();
    let mut found_suitable_positions = EntityHashSet::new();


    for found_ev in ereader_suitable_pos_found.read() {
        found_suitable_positions.insert(found_ev.studied_op_ent);
    }
    
    for pos_search in events_pos_search.drain() {

        if found_suitable_positions.contains(&pos_search.studied_op_ent) {
            info!("Found suitable position for {:?}", pos_search.studied_op_ent);
            continue;
        }

        let (studied_op_ent, step_size, curr_iteration_batch_i, iterations_per_batch, max_batches, dimension_hash_id) = 
        (pos_search.studied_op_ent, pos_search.step_size, pos_search.curr_iteration_batch_i, pos_search.iterations_per_batch, pos_search.max_batches, pos_search.dimension_hash_id);

        let Ok(studied_op) = studied_ops.get(studied_op_ent) else {//ERRROR: ENTTIY NO SPAWNEÓ TODAVÍA
            if curr_iteration_batch_i == 0 {
                // If we want to retry, push a new PosSearch with decremented batch index
                let mut new_search = pos_search;
                new_search.curr_iteration_batch_i -= 1;
                new_pos_searches.push(new_search);
            } else if curr_iteration_batch_i == -2 {
                error!("StudiedOp entity {:?} not found in search_suitable_position, giving up", studied_op_ent);
                search_failed_evs.push(SearchFailed(studied_op_ent));
            }
            continue;
        };
        let curr_iteration_batch_i = curr_iteration_batch_i.max(0);

        match pos_search.search_pattern {
            SearchPattern::Radial(explore_angle) => {

                let calculate_pos = |i_within_batch: u16, probe_direction: f32| -> GlobalTilePos {
                    let global_i = (curr_iteration_batch_i as u16 * iterations_per_batch as u16 + i_within_batch) as f32 * step_size as f32;
                    studied_op.search_start_pos + GlobalTilePos::from(IVec2::new(
                    (global_i * probe_direction.cos()) as i32, (global_i * probe_direction.sin()) as i32,
                    ))
                };

                if let Some(explore_angle) = explore_angle {

                    let start_i_within_batch = (curr_iteration_batch_i == 0) as u16;

                    for i_within_batch in start_i_within_batch..iterations_per_batch {
                        new_pending_ops.push(PendingOp {
                            oplist: studied_op.root_oplist,
                            dimension_hash_id,
                            pos: calculate_pos(i_within_batch, explore_angle),
                            studied_op_ent,
                            variables: VariablesArray::default(),
                            chunk_ent: Entity::PLACEHOLDER,
                        });
                    }
                    if curr_iteration_batch_i as u16 + 1 < max_batches {
                        new_pos_searches.push(PosSearch {
                            curr_iteration_batch_i: curr_iteration_batch_i + 1,
                            search_pattern: SearchPattern::Radial(Some(explore_angle)),
                            ..pos_search
                        });
                    } else {
                        error!("No more batches to search for {:?}", studied_op);
                        search_failed_evs.push(SearchFailed(studied_op_ent));
                    }
                } else {
                    if curr_iteration_batch_i as u16 >= max_batches {
                        error!("curr No more batches to search for {:?}", pos_search);
                        continue;
                    }
                    let divisions = 8;
                    for i in 0..divisions {
                        let angle = 2.0 * PI * (i as f32) / (divisions as f32);
                        new_pos_searches.push(PosSearch{
                            search_pattern: SearchPattern::Radial(Some(angle)),
                            ..pos_search
                        });
                    }
                }
            }
            
            SearchPattern::Spiral(mut curr_length_in_dir, mut steps_taken, mut dir_vec, mut pos, mut turns) => {
            // Spiral search: move in a direction for curr_length_in_dir steps, then turn 90°, increase length every two turns

                trace!("Spiral search started at pos {:?}, dir_vec {:?}, curr_length_in_dir {}, turns {}", 
                    pos, dir_vec, curr_length_in_dir, turns);

                for _ in 0..iterations_per_batch {
                    pos = pos + GlobalTilePos(dir_vec * step_size as i32);

                    new_pending_ops.push(PendingOp {
                        dimension_hash_id,
                        oplist: studied_op.root_oplist,
                        chunk_ent: Entity::PLACEHOLDER,
                        pos,
                        variables: VariablesArray::default(),
                        studied_op_ent,
                    });

                    steps_taken += 1;
                    if steps_taken >= curr_length_in_dir {
                        steps_taken = 0;
                        
                        dir_vec = dir_vec.perp();
                        curr_length_in_dir += turns as u32;
                        turns = !turns;
                    }
                }
                if curr_iteration_batch_i as u16 + 1 < max_batches {
                    new_pos_searches.push(PosSearch{
                        curr_iteration_batch_i: curr_iteration_batch_i + 1,
                        search_pattern: SearchPattern::Spiral(curr_length_in_dir, steps_taken, dir_vec, pos, turns),
                        ..pos_search
                    });
                } else {
                    error!("No more batches to search for {:?}", studied_op);
                    //cmd.entity(studied_op_ent).try_despawn();
                    search_failed_evs.push(SearchFailed(studied_op_ent));
                }
            },   
        }
    }
    ewriter_pending_ops.write_batch(new_pending_ops);
    events_pos_search.send_batch(new_pos_searches);
    ewriter_search_failed.write_batch(search_failed_evs);
}


