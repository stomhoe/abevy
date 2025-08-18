


use bevy::{ecs::entity_disabling::Disabled, prelude::*};
use bevy_ecs_tilemap::tiles::TilePos;
use bevy_replicon::shared::server_entity_map::ServerEntityMap;
use bevy_replicon_renet::renet::RenetServer;
use common::common_states::GameSetupType;
use debug_unwraps::DebugUnwrapExt;
use dimension::dimension_components::{DimensionRef, MultipleDimensionRefs};
use player::player_components::{HostPlayer, OfSelf, Player};

use crate::{chunking_components::*, terrain_gen::{terrgen_components::*, terrgen_resources::* }, tile::tile_components::* , };



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

        let chunk_area = ChunkInitState::SIZE.element_product() as i32;
        let mut pending_ops_count: i32 = 0;

        for (oplist_ent, oplist_dim_refs, oplist_size) in oplists.iter() {
            if !oplist_dim_refs.0.contains(&dim_ref.0) {
                continue;
            }
            let mut batch = Vec::with_capacity(chunk_area as usize);
            for x in 0..ChunkInitState::SIZE.x / oplist_size.x() {
                for y in 0..ChunkInitState::SIZE.y / oplist_size.y() {
                    let pos_within_chunk = IVec2::new(x as i32, y as i32);
                    let global_pos = chunk_pos.to_tilepos() + GlobalTilePos(pos_within_chunk * oplist_size.inner().as_ivec2());
                    trace!(
                        target: "terrgen",
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
            pending_ops_count += (ChunkInitState::SIZE.element_product() / oplist_size.inner().element_product()) as i32;
            if commands.get_entity(chunk_ent).is_err() {break 'oplist;}

            commands.spawn_batch(batch);
        }
        
        if pending_ops_count <= 0 {      
            warn!(target: "terrgen", "No operations to spawn for chunk {:?} in dimension {:?}", chunk_pos, dim_ref);      
            continue;
        }

        commands.entity(chunk_ent).try_insert((PendingOperations(pending_ops_count), ProducedTiles::new_with_chunk_capacity()));
        trace!(target: "terrgen", "Spawned terrain operations for chunk {:?} in {:?}", chunk_pos, now.elapsed());
    }
    Ok(())
}

#[allow(unused_parens)]
pub fn produce_tiles(mut cmd: Commands, 
    gen_settings: Res<GlobalGenSettings>,
    oplist_query: Query<(&OperationList, &ProducedTiles, &OplistSize ), ( )>,
    mut instantiated_oplist_query: Query<(Entity, &InputOperand, &OplistRef, &ChunkRef, &GlobalTilePos), ()>, 
    operands: Query<(Option<&FnlNoise>, ), ( )>,
    mut chunk_query: Query<(&mut PendingOperations, &mut ProducedTiles, &ChunkPos, &ChildOf), (Without<OperationList> )>,
    weight_maps: Query<(&HashPosEntiWeightedSampler, ), ( )>,
    tile_query: Query<(Has<TilemapChild>, Option<&Transform>), (With<Tile>, With<Disabled>, )>,
    state : Res<State<GameSetupType>>,
) -> Result {
    let is_host = state.get() != &GameSetupType::AsJoiner;

    for (enti, &input_operand, &oplist_ref, &chunk_ref, &global_tile_pos) in instantiated_oplist_query.iter_mut() {
        cmd.entity(enti).despawn();//NO PONER ABAJO

        let Ok((mut pending_ops_count, mut chunk_tiles, &chunk_pos, child_of)) = chunk_query.get_mut(chunk_ref.0)
        else { continue };

        let mut acc_val: f32 = input_operand.0;


        let (oplist, oplist_tiles, &my_oplist_size) = oplist_query.get(oplist_ref.0)?;

        let pos_within_chunk = global_tile_pos.get_pos_within_chunk(chunk_pos, my_oplist_size);
        trace!(target: "terrgen", "Producing tiles at {:?} with oplist {:?} for chunk {:?}", pos_within_chunk, oplist_ref.0, chunk_pos);

        for ((operand, operation)) in oplist.trunk.iter() {

            let num_operand = match operand {
                Operand::Entities(entities) => {
                    entities.iter().fold(1.0, |acc, &ent| {
                        if let Ok((fnl_comp, )) = operands.get(ent) {
                            acc * fnl_comp.map_or(1.0, |fnl| fnl.get_val(global_tile_pos))
                        } else {
                            acc
                        }
                    })
                },      Operand::Value(val) => *val,
                Operand::HashPos => global_tile_pos.normalized_hash_value(&gen_settings, 0),
                Operand::PoissonDisk(poisson_disk) => poisson_disk.sample(&gen_settings, global_tile_pos, my_oplist_size),
                _ => {
                    error!(target: "terrgen", "Unsupported operand type as numeric value: {:?}", operand);
                    0.0
                },
            };

            match operation {
                Operation::Add => acc_val += num_operand,
                Operation::Subtract => acc_val -= num_operand,
                Operation::Multiply => acc_val *= num_operand,
                Operation::MultiplyOpo => acc_val *= (1.0 - num_operand),
                Operation::Divide => if num_operand != 0.0 { acc_val /= num_operand },
                Operation::Min => acc_val = acc_val.min(num_operand),
                Operation::Max => acc_val = acc_val.max(num_operand),
                Operation::Pow => if acc_val >= 0.0 || num_operand.fract() == 0.0 { acc_val = acc_val.powf(num_operand) },
                Operation::Modulo => if num_operand != 0.0 { acc_val = acc_val % num_operand },
                Operation::Log => if acc_val > 0.0 && num_operand > 0.0 && num_operand != 1.0 { acc_val = acc_val.log(num_operand) },
                Operation::Assign => {acc_val = num_operand;},
                Operation::Mean => {acc_val = acc_val.lerp(num_operand, 0.5);},
                Operation::Abs => acc_val = acc_val.abs(),
                Operation::MultiplyNormalized => acc_val *= (num_operand - 0.5) * 2.,
                Operation::MultiplyNormalizedAbs => acc_val *= ((num_operand - 0.5) * 2.).abs(),
            }
        }
        chunk_tiles.insert_clonespawned_with_pos(&oplist_tiles, &mut cmd, global_tile_pos, pos_within_chunk, &weight_maps, &tile_query, &gen_settings,
            my_oplist_size, DimensionRef(child_of.parent()), is_host);

        if acc_val > oplist.threshold {
            if let Some(bifover_ent) = oplist.bifurcation_over {
                pending_ops_count.0 += spawn_bifurcation(&mut cmd, &oplist_query, bifover_ent, acc_val, &chunk_ref, global_tile_pos, my_oplist_size);
            }
        } else {
            if let Some(bifunder_ent) = oplist.bifurcation_under {
                pending_ops_count.0 += spawn_bifurcation(&mut cmd, &oplist_query, bifunder_ent, acc_val, &chunk_ref, global_tile_pos, my_oplist_size);
            }
        }

        pending_ops_count.0 -= 1;
        
        if pending_ops_count.0 <= 0  {
            cmd.entity(chunk_ref.0).try_remove::<PendingOperations>().try_insert(TilesReady);
        }
    }
    Ok(())
}


fn spawn_bifurcation(
    cmd: &mut Commands,
    oplist_query: &Query<(&OperationList, &ProducedTiles, &OplistSize), ()>,
    bif_ent: Entity,
    acc_val: f32,
    chunk_ref: &ChunkRef,
    global_tile_pos: GlobalTilePos,
    my_oplist_size: OplistSize,
) -> i32 {
    unsafe{
        let (_oplist, _tiles, &child_oplist_size) = oplist_query.get(bif_ent).debug_expect_unchecked("faltacoso");
        if my_oplist_size != child_oplist_size
            && (global_tile_pos.0.abs().as_uvec2() % child_oplist_size.inner() == UVec2::ZERO)
        {
            let mut batch = Vec::with_capacity(child_oplist_size.size());
            for x in 0..child_oplist_size.x() as i32 {
                for y in 0..child_oplist_size.y() as i32 {
                    let pos = global_tile_pos + GlobalTilePos::new(x, y);
                    batch.push((OplistRef(bif_ent), InputOperand(acc_val), chunk_ref.clone(), pos));
                }
            }
            cmd.spawn_batch(batch);
            child_oplist_size.size() as i32
        } else {
            cmd.spawn((OplistRef(bif_ent), InputOperand(acc_val), chunk_ref.clone(), global_tile_pos));
            1
        }
    }
}

//#[cfg(not(feature = "headless_server"))]
#[allow(unused_parens)]
pub fn client_change_operand_entities(
    mut query: Query<(&mut OperationList), (Added<OperationList>)>, 
    mut map: ResMut<ServerEntityMap>,
)
{
    for mut oplist in query.iter_mut() {
        for (operand, _) in &mut oplist.trunk {
            let Operand::Entities(entities) = operand 
            else { continue };

            let mut new_entities = Vec::with_capacity(entities.len());
            for ent in entities.iter() {
                if let Some(new_ent) = map.server_entry(*ent).get() {
                    new_entities.push(new_ent);
                } else {
                    error!(target: "oplist_loading", "Entity {} not found in ServerEntityMap", ent);
                    new_entities.push(Entity::PLACEHOLDER);
                }
            }
            *operand = Operand::Entities(new_entities);
        
        }
    }
}

