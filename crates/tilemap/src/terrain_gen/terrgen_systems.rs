


use bevy::{ecs::{entity::MapEntities, entity_disabling::Disabled}, prelude::*};
use bevy_ecs_tilemap::tiles::TilePos;
use bevy_replicon::{prelude::Replicated, shared::server_entity_map::ServerEntityMap};
use common::{common_components::{DisplayName, StrId}, common_states::GameSetupType};
use debug_unwraps::DebugUnwrapExt;
use dimension::dimension_components::{MultipleDimensionRefs};
use game_common::{game_common_components::DimensionRef, game_common_components_samplers::EntiWeightedSampler};
use tilemap_shared::{AaGlobalGenSettings, ChunkPos, GlobalTilePos};
use crate::{chunking_components::*, terrain_gen::{terrgen_components::*, terrgen_oplist_components::*, terrgen_resources::* }, tile::tile_components::* , };
use std::mem::take;

// HACER Q CADA UNA DE ESTAS ENTITIES APAREZCA EN LOS SETTINGS EN SETUP Y SEA CONFIGURABLE

// PARA HACER ISLAS CON FORMA CUSTOM (P. EJ CIRCULAR O DISCO O ALGO RARO Q NO SE PUEDE HACER CON NOISE), MARCAR EN UN PUNTO EXTREMADAMENTE OCÉANICO CON UNA TILE MARKER Y DESP HACER OTRO SISTEMA Q LO PONGA TODO POR ENCIMA, SOBREESCRIBIENDO LO Q HABÍA ANTES


#[allow(unused_parens)]
pub fn spawn_terrain_operations (
    mut commands: Commands, 
    chunks_query: Query<(Entity, &ChunkPos, &ChildOf), (With<UninitializedChunk>, Without<PendingOperations>, )>, 
    oplists: Query<(Entity, &MultipleDimensionRefs, &OplistSize), (With<OperationList>, )>,
) -> Result {
    
    'oplist: for (chunk_ent, chunk_pos, dim_ref) in chunks_query.iter() {
        //SE LES PODRÍA AGREGAR MARKER COMPONENTS A LOS CHUNKS PARA POR EJEMPLO ESPECIFICAR SI ES UN DUNGEON

//PONER MARKERS A TODAS LAS POSICIONES SUITABLE, DESPUES HACER UNA QUERY Q COMPARA LAS TILESMARCADAS COMO Q YA GENERARON UNA ESTRUCTURA NO PROCEDURAL CON LAS Q NO. SI LA DISTANCIA ES SUFICIENTE, SPAWNEAR UNA EN LA SIGUIENTE
        //EN ESTE PUNTO SE PODRÍA GENERAR UN CAMINO RANDOM QUE SEA UN VEC DE COORDS, Y DESPUES PASARLO ABAJO Y Q SE OCUPEN?? PA GENERAR DUNGEONS NASE
        let now = std::time::Instant::now();

        let chunk_area = ChunkPos::CHUNK_SIZE.element_product() as i32;
        let mut pending_ops_count: i32 = 0;

        for (oplist_ent, oplist_dim_refs, oplist_size) in oplists.iter() {
            if !oplist_dim_refs.0.contains(&dim_ref.0) {
                continue;
            }
            let mut batch = Vec::with_capacity(chunk_area as usize);
            for x in 0..ChunkPos::CHUNK_SIZE.x / oplist_size.x() {
                for y in 0..ChunkPos::CHUNK_SIZE.y / oplist_size.y() {
                    let pos_within_chunk = IVec2::new(x as i32, y as i32);
                    let global_pos = chunk_pos.to_tilepos() + GlobalTilePos(pos_within_chunk * oplist_size.inner().as_ivec2());
                    trace!(
                        "Spawning terrain operation {:?} at {:?} in chunk {:?}, pos_within_chunk: {:?}, oplist_size: {:?}",
                        oplist_ent,
                        global_pos,
                        chunk_ent,
                        pos_within_chunk,
                        oplist_size
                    );
                    if commands.get_entity(chunk_ent).is_err() {
                        break 'oplist;
                    }
                    batch.push((
                        OplistRef(oplist_ent), ChunkRef(chunk_ent),
                        global_pos,
                    ));
                    
                }
            }
            pending_ops_count += (ChunkPos::CHUNK_SIZE.element_product() / oplist_size.inner().element_product()) as i32;
            if commands.get_entity(chunk_ent).is_err() {break 'oplist;}

            commands.spawn_batch(batch);
        }
        
        if pending_ops_count <= 0 {      
            trace!("No operations to spawn for chunk {:?} in dimension {:?}", chunk_pos, dim_ref);      
            continue;
        }

        commands.entity(chunk_ent).try_insert((PendingOperations(pending_ops_count), ProducedTiles::new_with_chunk_capacity()));
        trace!("Spawned terrain operations for chunk {:?} in {:?}", chunk_pos, now.elapsed());
    }
    Ok(())
}

#[allow(unused_parens)]
pub fn produce_tiles(mut cmd: Commands, 
    gen_settings: Res<AaGlobalGenSettings>,
    oplist_query: Query<(&OperationList, &OplistSize ), ( )>,
    mut instantiated_oplist_query: Query<(Entity, &mut VariablesArray, &OplistRef, &ChunkRef, &GlobalTilePos), ()>, 
    fnl_noises: Query<&FnlNoise,>,
    mut chunk_query: Query<(&mut PendingOperations, &mut ProducedTiles, &ChunkPos, ), ()>,
    weight_maps: Query<(&EntiWeightedSampler, ), ( )>,
) -> Result {

    for (openti, mut variables, &oplist_ref, &chunk_ref, &global_pos) in instantiated_oplist_query.iter_mut() {
        cmd.entity(openti).try_despawn();//NO PONER ABAJO

        let Ok((mut pending_ops_count, mut chunk_tiles, &chunk_pos, )) = chunk_query.get_mut(chunk_ref.0)
        else { continue };

        let (oplist, &my_oplist_size) = oplist_query.get(oplist_ref.0)?;

        let pos_within_chunk = global_pos.get_pos_within_chunk(chunk_pos, my_oplist_size.inner());

        unsafe{

        for ((operation, operands, stackarr_out_i)) in oplist.trunk.iter() {
            let mut operation_acc_val: f32 = 0.0;
            let mut selected_operand_i = 0; 

            for (operand_i, operand) in operands.iter().enumerate() {
                let curr_operand_val = match operand {
                    Operand::StackArray(i) => variables[*i],
                    Operand::Value(val) => *val,
                    Operand::NoiseEntity(ent, sample_range, compl, seed) => {
                        if let Ok(noise) = fnl_noises.get(*ent) {
                            noise.sample(global_pos, *sample_range, *compl, *seed, &gen_settings)
                        } else {
                            error!("Entity {} not found in terrgens", ent);
                            continue;
                        }
                    },
                    Operand::HashPos(seed) => global_pos.normalized_hash_value(&gen_settings, *seed),
                    Operand::PoissonDisk(poisson_disk) => poisson_disk.sample(&gen_settings, global_pos, my_oplist_size),
                };

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
                    (Operation::i_Max, _, false) => {if curr_operand_val > operation_acc_val { operation_acc_val = curr_operand_val; selected_operand_i = operand_i; }}
                    (Operation::i_Max, _, true) => {if curr_operand_val > operation_acc_val { selected_operand_i = operand_i; } operation_acc_val = selected_operand_i as f32;}
       
                    (_, 0, _) => {operation_acc_val = curr_operand_val;},
                }

                trace!(
                    "{} with operand {:?} at stack array index {}: prev_value: {}, curr_value: {}, {:?}, {:?}",
                    operation,
                    operand,
                    *stackarr_out_i,
                    prev_value,
                    operation_acc_val,
                    global_pos,
                    chunk_pos
                );      
            }
            trace!("Operation result for stack array index {}: {}", *stackarr_out_i, operation_acc_val);
            variables[*stackarr_out_i] = operation_acc_val;

        }
        let destination_i = (variables[0] as usize).min(oplist.bifurcations.len() - 1).max(0);
        trace!("Destination index for bifurcation: {}", destination_i);

        let bifurcation = oplist.bifurcations.get(destination_i).debug_unwrap_unchecked();

        chunk_tiles.insert_as_instanced_tiles(&mut cmd, &bifurcation.tiles, pos_within_chunk, global_pos, my_oplist_size, &weight_maps, &gen_settings);

        if let Some(oplist) = bifurcation.oplist {
            pending_ops_count.0 += spawn_bifurcation_oplists(&mut cmd, &oplist_query, oplist, take(&mut variables), &chunk_ref, global_pos, my_oplist_size);
        }

        pending_ops_count.0 -= 1;
        
        if pending_ops_count.0 <= 0  {
            cmd.entity(chunk_ref.0).try_remove::<PendingOperations>().try_insert(TilesInstantiated);
        }
    }}
    Ok(())
}

#[allow(unused_parens)]
pub fn process_tiles(mut cmd: Commands, 
    chunk_query: Query<(Entity, &ProducedTiles, &ChildOf), (With<TilesInstantiated>)>,
    tile_query: Query<(&GlobalTilePos, &TilePos, &OplistSize, Has<TilemapChild>, Option<&Transform>, &TileRef), (With<Tile>, With<Disabled>, )>,
    oritile_query: Query<(Option<&MinDistancesMap>, Option<&KeepDistanceFrom>), (With<Disabled>)>,
    min_dists_query: Query<(&MinDistancesMap), (With<Disabled>)>,
    mut regpos_map: ResMut<RegisteredPositions>,
    state: Res<State<GameSetupType>>,
) {
    let is_host = state.get() != &GameSetupType::AsJoiner;

    for (chunk_ent, produced_tiles, child_of) in chunk_query.iter() {
        for tile_ent in produced_tiles.0.iter() {
            let Ok((&global_pos, &pos_within_chunk, &oplist_size, tilemap_child, transform, &tile_ref)) = tile_query.get(*tile_ent)
            else { cmd.entity(*tile_ent).try_despawn(); continue; };

         
            let Ok((oritile_min_dists, register_pos)) = oritile_query.get(tile_ref.0)
            else { cmd.entity(*tile_ent).try_despawn(); continue; };

            if false == regpos_map.check_min_distances(&mut cmd, is_host, (tile_ref.0, global_pos, oritile_min_dists, register_pos), min_dists_query) {
                cmd.entity(*tile_ent).try_despawn(); continue;
            }

            cmd.entity(*tile_ent).try_insert((DimensionRef(child_of.parent()), ));

            if tilemap_child {
                if let Some(transform) = transform {
                    let displacement: Vec2 = Vec2::from(pos_within_chunk) * oplist_size.inner().as_vec2() * GlobalTilePos::TILE_SIZE_PXS.as_vec2();
                    let displacement = transform.translation + displacement.extend(0.0);
                    cmd.entity(*tile_ent).try_insert((ChildOf(chunk_ent), Transform::from_translation(displacement))).try_remove::<(TilemapChild, TilePos, Disabled)>();
                    //SI SE QUIERE SACAR EL CHILDOF CHUNK, HAY Q REAJUSTAR EL TRANSFORM
                }
            
            } else if is_host {
                let mut displacement: Vec3 = Into::<Vec2>::into(global_pos).extend(0.0);
                if let Some(transform) = transform {
                    displacement += transform.translation;
                } 

                cmd.entity(*tile_ent)
                    .try_remove::<(Tile, Disabled, OplistSize, TilePos, GlobalTilePos)>()
                    .try_insert((Replicated, child_of.clone(), Transform::from_translation(displacement)));
                
            } else {
                cmd.entity(*tile_ent).try_despawn();
            }
        }
        cmd.entity(chunk_ent).try_insert(TilesReady);
    }
}




fn spawn_bifurcation_oplists(
    cmd: &mut Commands,
    oplist_query: &Query<(&OperationList, &OplistSize), ()>,
    bif_ent: Entity,
    variables: VariablesArray,
    chunk_ref: &ChunkRef,
    global_tile_pos: GlobalTilePos,
    my_oplist_size: OplistSize,
) -> i32 {
    unsafe{
        let (_oplist, &child_oplist_size) = oplist_query.get(bif_ent).debug_expect_unchecked("faltacoso");
        if my_oplist_size != child_oplist_size
            && (global_tile_pos.0.abs().as_uvec2() % child_oplist_size.inner() == UVec2::ZERO)
        {
            let mut batch = Vec::with_capacity(child_oplist_size.size());
            for x in 0..child_oplist_size.x() as i32 {
                for y in 0..child_oplist_size.y() as i32 {
                    let pos = global_tile_pos + GlobalTilePos::new(x, y);
                    batch.push((OplistRef(bif_ent), variables.clone(), chunk_ref.clone(), pos));
                }
            }
            cmd.spawn_batch(batch);
            child_oplist_size.size() as i32
        } else {
            cmd.spawn((OplistRef(bif_ent), variables.clone(), chunk_ref.clone(), global_tile_pos));
            1
        }
    }
}


#[derive(serde::Deserialize, Event, serde::Serialize, Clone, MapEntities)]
pub struct NewlyRegPos (#[entities] pub Entity, pub GlobalTilePos);

// No olvidarse de agregarlo al Plugin del módulo

// ----------------------> NO OLVIDARSE DE AGREGARLO AL Plugin DEL MÓDULO <-----------------------------
//                                                       ^^^^
#[allow(unused_parens)]
pub fn sync_register_new_pos(
    trigger: Trigger<NewlyRegPos>,
    mut own_map: ResMut<RegisteredPositions>,
    state: Res<State<GameSetupType>>,

) {
    let is_host = state.get() != &GameSetupType::AsJoiner;

    if is_host { return; }

    let registered_positions = trigger.event().clone();

    own_map.0.entry(registered_positions.0).or_default().push(registered_positions.1);
    
}
