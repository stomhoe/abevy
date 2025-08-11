


use bevy::prelude::*;

use crate::{chunking_components::*, terrain_gen::{terrgen_components::*, terrgen_resources::* }, tile::tile_components::* , };



// HACER Q CADA UNA DE ESTAS ENTITIES APAREZCA EN LOS SETTINGS EN SETUP Y SEA CONFIGURABLE

// PARA HACER ISLAS CON FORMA CUSTOM (P. EJ CIRCULAR O DISCO O ALGO RARO Q NO SE PUEDE HACER CON NOISE), MARCAR EN UN PUNTO EXTREMADAMENTE OCÉANICO CON UNA TILE MARKER Y DESP HACER OTRO SISTEMA Q LO PONGA TODO POR ENCIMA, SOBREESCRIBIENDO LO Q HABÍA ANTES


#[allow(unused_parens)]
pub fn spawn_terrain_operations (
    mut commands: Commands, 
    chunks_query: Query<(Entity, &ChunkPos), (With<UninitializedChunk>, Without<PendingOperations>, )>, 
    oplists: Query<(Entity), (With<OperationList>,  With<RootOpList>)>,
) -> Result {
    for (chunk_ent, chunk_pos) in chunks_query.iter() {
        //SE LES PODRÍA AGREGAR MARKER COMPONENTS A LOS CHUNKS PARA POR EJEMPLO ESPECIFICAR SI ES UN DUNGEON

//PONER MARKERS A TODAS LAS POSICIONES SUITABLE, DESPUES HACER UNA QUERY Q COMPARA LAS TILESMARCADAS COMO Q YA GENERARON UNA ESTRUCTURA NO PROCEDURAL CON LAS Q NO. SI LA DISTANCIA ES SUFICIENTE, SPAWNEAR UNA EN LA SIGUIENTE
        //EN ESTE PUNTO SE PODRÍA GENERAR UN CAMINO RANDOM QUE SEA UN VEC DE COORDS, Y DESPUES PASARLO ABAJO Y Q SE OCUPEN?? PA GENERAR DUNGEONS NASE
        let now = std::time::Instant::now();

        let chunk_area = ChunkInitState::SIZE.element_product() as i32;
        let mut pending_ops_count: i32 = 0;

        oplists.iter().for_each(|oplist_ent| {
            let mut batch = Vec::with_capacity((chunk_area) as usize);
            for x in 0..ChunkInitState::SIZE.x { 
                for y in 0..ChunkInitState::SIZE.y {
                    let pos_within_chunk = IVec2::new(x as i32, y as i32);
                    batch.push((
                        OplistRef(oplist_ent), ChunkRef(chunk_ent),
                        chunk_pos.to_tilepos() + GlobalTilePos(pos_within_chunk),
                    ));
                }
            }
            pending_ops_count += 1;
            commands.spawn_batch(batch);
        });

        commands.entity(chunk_ent).insert((PendingOperations(pending_ops_count*chunk_area), ProducedTiles::new_with_chunk_capacity()));
        trace!(target: "terrgen", "Spawned terrain operations for chunk {:?} in {:?}", chunk_pos, now.elapsed());
    }
    Ok(())
}

#[allow(unused_parens)]
pub fn produce_tiles(mut cmd: Commands, 
    gen_settings: Res<GlobalGenSettings>,
    mut query: Query<(Entity, &InputOperand, &OplistRef, &ChunkRef, &GlobalTilePos), (Added<InputOperand>, )>, 
    oplist_query: Query<(&OperationList, &ProducedTiles ), ( )>,
    operands: Query<(Option<&FnlNoise>, ), ( )>,
    mut chunk_query: Query<(&mut PendingOperations, &mut ProducedTiles, &ChunkPos), (Without<OperationList> )>,
    weight_maps: Query<(&HashPosEntiWeightedSampler, ), ( )>,
) -> Result {
    for (enti, &input_operand, &oplist_ref, &chunk_ref, &global_tile_pos) in query.iter_mut() {

        let Ok((mut pending_ops_count, mut chunk_tiles, &chunk_pos)) = chunk_query.get_mut(chunk_ref.0) 
        else { continue };
        
        let mut acc_val: f32 = input_operand.0; 

        cmd.entity(enti).despawn();//NO PONER ABAJO

        let (oplist, oplist_tiles) = oplist_query.get(oplist_ref.0)?;

        let pos_within_chunk = global_tile_pos.get_pos_within_chunk(chunk_pos);
        trace!(target: "terrgen", "Producing tiles at {:?} with oplist {:?}", pos_within_chunk, oplist_ref.0);

        for ((operand, operation)) in oplist.trunk.iter() {
            
            let num_operand = match operand {
                Operand::Entities(entities) => {
                    if let Ok((fnl_comp, )) = operands.get(entities[0]) {
                        fnl_comp.map_or(0.0, |fnl| fnl.get_val(global_tile_pos))
                    } else {
                        0.0 // Si no hay componente, asumimos 0
                    }
                },
                Operand::Value(val) => *val,
                Operand::HashPos => global_tile_pos.normalized_hash_value(&gen_settings, 0),
                Operand::PoissonDisk(poisson_disk) => poisson_disk.sample(&gen_settings, global_tile_pos),
                _ => {
                    error!(target: "terrgen", "Unsupported operand type as numeric value: {:?}", operand);
                    0.0
                },
            };

            match operation {
                Operation::Add => acc_val += num_operand,
                Operation::Subtract => acc_val -= num_operand,
                Operation::Multiply => acc_val *= num_operand,
                Operation::Divide => if num_operand != 0.0 { acc_val /= num_operand },
                Operation::Min => acc_val = acc_val.min(num_operand),
                Operation::Max => acc_val = acc_val.max(num_operand),
                Operation::Pow => if acc_val >= 0.0 || num_operand.fract() == 0.0 { acc_val = acc_val.powf(num_operand) },
                Operation::Modulo => if num_operand != 0.0 { acc_val = acc_val % num_operand },
                Operation::Log => if acc_val > 0.0 && num_operand > 0.0 && num_operand != 1.0 { acc_val = acc_val.log(num_operand) },
                Operation::Assign => {acc_val = num_operand;},
                Operation::Mean => {acc_val = acc_val.lerp(num_operand, 0.5);},
            }
        }
        chunk_tiles.insert_clonespawned_with_pos(&oplist_tiles, &mut cmd, global_tile_pos, pos_within_chunk, &weight_maps, &gen_settings);
        
        
        if acc_val > oplist.threshold {
            if let Some(bifover_ent) = oplist.bifurcation_over {
                cmd.spawn((OplistRef(bifover_ent), InputOperand(acc_val), chunk_ref.clone(), global_tile_pos));
                continue;
            }
        }
        else {
            if let Some(bifunder_ent) = oplist.bifurcation_under {
                cmd.spawn((OplistRef(bifunder_ent), InputOperand(acc_val), chunk_ref.clone(), global_tile_pos));
                continue;
            }
        }

        pending_ops_count.0 -= 1;
        if pending_ops_count.0 <= 0 {
            cmd.entity(chunk_ref.0).remove::<PendingOperations>().insert(TilesReady);
        }
    }
    Ok(())
}