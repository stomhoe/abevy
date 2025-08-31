


use bevy::{ecs::{entity::MapEntities, entity_disabling::Disabled}, prelude::*};
use bevy_ecs_tilemap::tiles::TilePos;
use bevy_replicon::{prelude::Replicated, shared::server_entity_map::ServerEntityMap};
use common::{common_components::{DisplayName, StrId}, common_states::GameSetupType};
use debug_unwraps::DebugUnwrapExt;
use dimension::dimension_components::{MultipleDimensionRefs};
use game_common::{game_common_components::DimensionRef, game_common_components_samplers::EntiWeightedSampler};
use tilemap_shared::{AaGlobalGenSettings, ChunkPos, GlobalTilePos, HashablePosVec};
use crate::{chunking_components::*, chunking_resources::{AaChunkRangeSettings, LoadedChunks}, terrain_gen::{terrgen_components::*, terrgen_oplist_components::*, terrgen_resources::* }, tile::tile_components::* };
use std::mem::take;

// HACER Q CADA UNA DE ESTAS ENTITIES APAREZCA EN LOS SETTINGS EN SETUP Y SEA CONFIGURABLE

// PARA HACER ISLAS CON FORMA CUSTOM (P. EJ CIRCULAR O DISCO O ALGO RARO Q NO SE PUEDE HACER CON NOISE), MARCAR EN UN PUNTO EXTREMADAMENTE OCÉANICO CON UNA TILE MARKER Y DESP HACER OTRO SISTEMA Q LO PONGA TODO POR ENCIMA, SOBREESCRIBIENDO LO Q HABÍA ANTES


#[allow(unused_parens)]
pub fn spawn_terrain_operations (
    mut commands: Commands, 
    res_chunk: Res<AaChunkRangeSettings>,
    chunks_query: Query<(Entity, &ChunkPos, &ChildOf), (Without<PendingOps>, )>, 
    oplists: Query<(Entity, &MultipleDimensionRefs, &OplistSize), (With<OperationList>, )>,
    mut ew_pending_ops: EventWriter<PendingOp>,

) -> Result {
    if chunks_query.is_empty() { return Ok(()); }

    let chunk_area = res_chunk.approximate_number_of_tiles() * 4;
    let mut batch = Vec::with_capacity(chunk_area * 4);
    'oplist: for (chunk_ent, chunk_pos, dim_ref) in chunks_query.iter() {
        //SE LES PODRÍA AGREGAR MARKER COMPONENTS A LOS CHUNKS PARA POR EJEMPLO ESPECIFICAR SI ES UN DUNGEON

//PONER MARKERS A TODAS LAS POSICIONES SUITABLE, DESPUES HACER UNA QUERY Q COMPARA LAS TILESMARCADAS COMO Q YA GENERARON UNA ESTRUCTURA NO PROCEDURAL CON LAS Q NO. SI LA DISTANCIA ES SUFICIENTE, SPAWNEAR UNA EN LA SIGUIENTE
        //EN ESTE PUNTO SE PODRÍA GENERAR UN CAMINO RANDOM QUE SEA UN VEC DE COORDS, Y DESPUES PASARLO ABAJO Y Q SE OCUPEN?? PA GENERAR DUNGEONS NASE
        let now = std::time::Instant::now();

        let mut pending_ops_count: i32 = 0;

        for (oplist_ent, oplist_dim_refs, oplist_size) in oplists.iter() {
            if !oplist_dim_refs.0.contains(&dim_ref.0) {
                continue;
            }
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
                    batch.push(PendingOp {
                        oplist: oplist_ent,
                        chunk: chunk_ent,
                        pos: global_pos,
                        variables: VariablesArray::default(),
                    });

                }
            }
            pending_ops_count += (ChunkPos::CHUNK_SIZE.element_product() / oplist_size.inner().element_product()) as i32;
            if commands.get_entity(chunk_ent).is_err() {break 'oplist;}

        }
        
        if pending_ops_count <= 0 {      
            trace!("No operations to spawn for chunk {:?} in dimension {:?}", chunk_pos, dim_ref);      
            continue;
        }
        commands.entity(chunk_ent).try_insert(PendingOps(pending_ops_count));

        trace!("Spawned terrain operations for chunk {:?} in {:?}", chunk_pos, now.elapsed());
    }
    ew_pending_ops.write_batch(batch);
    Ok(())
}

#[derive(Event, Debug)]
pub struct PendingOp {oplist: Entity, chunk: Entity, pos: GlobalTilePos, variables: VariablesArray}

#[allow(unused_parens)]
pub fn produce_tiles(mut cmd: Commands, 
    gen_settings: Res<AaGlobalGenSettings>,
    res_chunk: Res<AaChunkRangeSettings>,
    oplist_query: Query<(&OperationList, &OplistSize ), ( )>,
    mut pending_ops_events: ResMut<Events<PendingOp>>,
    fnl_noises: Query<&FnlNoise,>,
    mut chunk_query: Query<&ChunkPos>,
    weight_maps: Query<(&EntiWeightedSampler, ), ( )>,
    mut ewriter_produced_tiles: EventWriter<InstantiatedTiles>,
) -> Result {

    if pending_ops_events.is_empty() { return Ok(()); }

    let chunk_area = res_chunk.approximate_number_of_tiles() * 2;

    let mut new_pending_ops_events = Vec::with_capacity(chunk_area);
    let mut produced_tiles_events = Vec::with_capacity(chunk_area);

    for mut ev in pending_ops_events.drain() {

        let Ok(&chunk_pos) = chunk_query.get_mut(ev.chunk)
        else { continue };
        

        let (oplist, &my_oplist_size) = oplist_query.get(ev.oplist)?;
        let global_pos = ev.pos;

        let pos_within_chunk = global_pos.get_pos_within_chunk(chunk_pos, my_oplist_size.inner());


        unsafe{

        for ((operation, operands, stackarr_out_i)) in oplist.trunk.iter() {
            let mut operation_acc_val: f32 = 0.0;
            let mut selected_operand_i = 0; 

            for (operand_i, operand) in operands.iter().enumerate() {
                let curr_operand_val = match operand {
                    Operand::StackArray(i) => ev.variables[*i],
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
                    Operand::PoissonDisk(poisson_disk) => {
                        poisson_disk.sample(&gen_settings, global_pos, my_oplist_size)
                    },
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
            ev.variables[*stackarr_out_i] = operation_acc_val;

        }
        let destination_i = (ev.variables[0] as usize).min(oplist.bifurcations.len() - 1).max(0);
        trace!("Destination index for bifurcation: {}", destination_i);

        let bifurcation = oplist.bifurcations.get(destination_i).debug_unwrap_unchecked();
        
        if let Some(oplist) = bifurcation.oplist {
            spawn_bifurcation_oplists(&mut ev, &oplist_query, &mut new_pending_ops_events, oplist, my_oplist_size);
        }

        if bifurcation.tiles.len() > 0 {
            let tiles = InstantiatedTiles::new(&ev, &mut cmd, &bifurcation.tiles, pos_within_chunk, my_oplist_size, &weight_maps, &gen_settings);
            produced_tiles_events.push(tiles);
        }


    }}
    pending_ops_events.send_batch(new_pending_ops_events);
    ewriter_produced_tiles.write_batch(produced_tiles_events);

    Ok(())
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, Reflect, )]
pub enum OplistCollectedTiles {
    Array([Entity; 4]),   
    Batch(Vec<Entity>),
}
impl OplistCollectedTiles {
    pub fn iter(&self) -> OplistCollectedTilesIter<'_> {
        match self {
            OplistCollectedTiles::Array(arr) => OplistCollectedTilesIter::Array { arr, idx: 0 },
            OplistCollectedTiles::Batch(vec) => OplistCollectedTilesIter::Batch { vec, idx: 0 },
        }
    }
    pub fn iter_mut(&mut self) -> OplistCollectedTilesIterMut<'_> {
        match self {
            OplistCollectedTiles::Array(arr) => OplistCollectedTilesIterMut::Array { arr: arr.as_mut_slice(), idx: 0 },
            OplistCollectedTiles::Batch(vec) => OplistCollectedTilesIterMut::Batch { vec: vec.as_mut_slice(), idx: 0 },
        }
    }

 
}
pub enum OplistCollectedTilesIterMut<'a> { Array { arr: &'a mut [Entity], idx: usize }, Batch { vec: &'a mut [Entity], idx: usize }, }
impl<'a> Iterator for OplistCollectedTilesIterMut<'a> {
    type Item = &'a mut Entity;
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            OplistCollectedTilesIterMut::Array { arr, idx } => {
                while *idx < arr.len() {
                    let i = *idx; *idx += 1;
                    let ptr = &mut arr[i] as *mut Entity;
                    unsafe { if *ptr != Entity::PLACEHOLDER { return Some(&mut *ptr); } }
                }
                None
            }
            OplistCollectedTilesIterMut::Batch { vec, idx } => {
                if *idx < vec.len() {
                    let i = *idx; *idx += 1;
                    let ptr = &mut vec[i] as *mut Entity;
                    unsafe { Some(&mut *ptr) }
                } else { None }
            }
        }
    }
}
pub enum OplistCollectedTilesIter<'a> { Array { arr: &'a [Entity; 4], idx: usize }, Batch { vec: &'a Vec<Entity>, idx: usize },}

impl<'a> Iterator for OplistCollectedTilesIter<'a> {
    type Item = Entity;
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            OplistCollectedTilesIter::Array { arr, idx } => {
                while *idx < arr.len() {
                    let ent = arr[*idx]; *idx += 1;
                    if ent != Entity::PLACEHOLDER { return Some(ent); }
                }
                None
            }
            OplistCollectedTilesIter::Batch { vec, idx } => {
                if *idx < vec.len() {
                    let ent = vec[*idx]; *idx += 1; Some(ent)
                } else { None }
            }
        }
    }
}

impl Default for OplistCollectedTiles {
    fn default() -> Self { Self::Array([Entity::PLACEHOLDER; 4]) }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, Reflect, Event, )]
pub struct InstantiatedTiles { pub chunk: Entity, pub tiles: OplistCollectedTiles }

impl InstantiatedTiles {
    #[allow(unused_parens, )]
    fn insert_tile_recursive(
        &mut self,
        cmd: &mut Commands,
        tiling_ent: Entity,
        tile_pos: TilePos,
        global_pos: GlobalTilePos,
        oplist_size: OplistSize,
        weight_maps: &Query<(&EntiWeightedSampler,), ()>,
        gen_settings: &AaGlobalGenSettings,
        depth: u32
    ) {
        if let Ok((wmap, )) = weight_maps.get(tiling_ent) {
            if let Some(tiling_ent) = wmap.sample_with_pos(gen_settings, global_pos) {

                if depth > 6 {
                    warn!("Tile insertion depth exceeded 6, stopping recursion for tile {:?}", tiling_ent);
                    return;
                }
                self.insert_tile_recursive( cmd, tiling_ent, tile_pos, global_pos, oplist_size, weight_maps, gen_settings, depth + 1);
            }
        } else { 
            let tile_ent = cmd.entity(tiling_ent).clone_and_spawn_with(|builder|{
                builder.deny::<BundleToDenyOnTileClone>();
            })
            .try_insert((
                TilemapChild,
                tile_pos, TileRef(tiling_ent), InitialPos(global_pos), global_pos, oplist_size))
            .id();

            // Insert into the array if there's space, otherwise switch to Batch
            match &mut self.tiles {
                OplistCollectedTiles::Array(arr) => {
                    if let Some(slot) = arr.iter_mut().find(|e| **e == Entity::PLACEHOLDER) {
                        *slot = tile_ent;
                    } else {
                        // No space left, convert to Batch
                        let mut batch = arr.iter().cloned().filter(|e| *e != Entity::PLACEHOLDER).collect::<Vec<_>>();
                        batch.push(tile_ent);
                        self.tiles = OplistCollectedTiles::Batch(batch);
                    }
                }
                OplistCollectedTiles::Batch(vec) => {
                    vec.push(tile_ent);
                }
            }
        }
    }
    pub fn new(precursor: &PendingOp, cmd: &mut Commands, tiling_ents: &Vec<Entity>,
        pos_within_chunk: TilePos, oplist_size: OplistSize,
        weight_maps: &Query<(&EntiWeightedSampler,), ()>, gen_settings: &AaGlobalGenSettings,
    ) -> Self {
        let mut instance = Self { chunk: precursor.chunk, tiles: OplistCollectedTiles::default() };
        for tile in tiling_ents.iter().cloned() {
            instance.insert_tile_recursive(cmd, tile, pos_within_chunk, precursor.pos, oplist_size, weight_maps, gen_settings, 0);
        }
        instance
    }
    pub fn take_tiles(&mut self) -> OplistCollectedTiles {take(&mut self.tiles)}

}


fn spawn_bifurcation_oplists(
    event: &mut PendingOp,
    oplist_query: &Query<(&OperationList, &OplistSize), ()>,
    new_pending_ops: &mut Vec<PendingOp>,
    bif_ent: Entity,
    my_oplist_size: OplistSize,
) {
    unsafe{
        let (_oplist, &child_oplist_size) = oplist_query.get(bif_ent).debug_expect_unchecked("faltacoso");
        if my_oplist_size != child_oplist_size
            && (event.pos.0.abs().as_uvec2() % child_oplist_size.inner() == UVec2::ZERO)
        {
            let x_end = child_oplist_size.x() as i32; let y_end = child_oplist_size.y() as i32;
            for x in 0..x_end {
                for y in 0..y_end {
                    let pos = event.pos + GlobalTilePos::new(x, y);

                    let variables = if x == x_end - 1 && y == y_end - 1 {
                        take(&mut event.variables)
                    } else {
                        event.variables.clone()
                    };

                    let new_event = PendingOp{ oplist: bif_ent, chunk: event.chunk, pos, variables };
                    new_pending_ops.push(new_event);
                }
            }
         
        } else {
            new_pending_ops.push(PendingOp{ oplist: bif_ent, chunk: event.chunk, pos: event.pos, variables: take(&mut event.variables) });
        }
    }
}


#[allow(unused_parens)]
pub fn process_tiles(mut cmd: Commands, 
    mut er_instantiated_tiles: EventMutator<InstantiatedTiles>,
    mut ew_processed_tiles: EventWriter<ProcessedTiles>,
    chunk_query: Query<(&ChildOf), ()>,
    entity_query: Query<Entity, Or<(With<Disabled>, Without<Disabled>)>>,
    tile_query: Query<(&GlobalTilePos, &TilePos, &OplistSize, Has<TilemapChild>, Option<&Transform>, &TileRef), (With<Tile>, Without<Disabled>, )>,
    oritile_query: Query<(Option<&MinDistancesMap>, Option<&KeepDistanceFrom>), (With<Disabled>)>,
    min_dists_query: Query<(&MinDistancesMap), (With<Disabled>)>,
    mut regpos_map: ResMut<RegisteredPositions>,
    state: Res<State<GameSetupType>>,
) {
    let is_host = state.get() != &GameSetupType::AsJoiner;

    
    if er_instantiated_tiles.is_empty() { return; }

    let mut processed_tiles_events = Vec::with_capacity(er_instantiated_tiles.len());
    for ev in er_instantiated_tiles.read() {
        let Ok((child_of)) = chunk_query.get(ev.chunk) else { continue; };

        for mut tile_ent in ev.tiles.iter_mut() {

            // let Ok(entity) = entity_query.get(*tile_ent) else { 
            //     error!("PROCESSTILES Tile entity {:?} does not exist", tile_ent);
            //     *tile_ent = Entity::PLACEHOLDER;
            //     continue; 
            // };

            let Ok((&global_pos, &pos_within_chunk, &oplist_size, tilemap_child, transform, &tile_ref)) = tile_query.get(*tile_ent)
            else { 
                error!("PROCESSTILES Failed to get components for tile entity {:?}", tile_ent);
                *tile_ent = Entity::PLACEHOLDER;
                cmd.entity(*tile_ent).try_despawn(); continue; };

            // Process tile
            let Ok((min_dists, keep_distance_from)) = oritile_query.get(tile_ref.0)
            else { 
                error!("PROCESSTILES Failed to get original tile components for tile entity {:?}", tile_ent);
                *tile_ent = Entity::PLACEHOLDER;
                cmd.entity(*tile_ent).try_despawn(); 
                continue; 
            };

            let dimref = DimensionRef(child_of.parent());

            if false == regpos_map.check_min_distances(&mut cmd, is_host, (tile_ref.0, dimref, global_pos, min_dists, keep_distance_from), min_dists_query) {
                *tile_ent = Entity::PLACEHOLDER;
                cmd.entity(*tile_ent).try_despawn(); continue;
            }

            cmd.entity(*tile_ent).try_insert((dimref, ));
            trace!("Spawned tile {:?} at global pos {:?} in dimension {:?}", tile_ent, global_pos, dimref);

            if tilemap_child {
                if let Some(transform) = transform {
                    let displacement: Vec2 = Vec2::from(pos_within_chunk) * oplist_size.inner().as_vec2() * GlobalTilePos::TILE_SIZE_PXS.as_vec2();
                    let displacement = transform.translation + displacement.extend(0.0);
                    cmd.entity(*tile_ent).try_insert((ChildOf(ev.chunk), Transform::from_translation(displacement))).try_remove::<(TilemapChild, TilePos, Disabled)>();
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
                error!("Tile {:?} at {:?} with pos within chunk {:?} is not a TilemapChild, despawning on client", tile_ent, global_pos, pos_within_chunk);
                *tile_ent = Entity::PLACEHOLDER;
                cmd.entity(*tile_ent).try_despawn();
            }
        }
        let protiles = ProcessedTiles { chunk: ev.chunk, tiles: ev.take_tiles() };
        processed_tiles_events.push(protiles);
        
    }
    ew_processed_tiles.write_batch(processed_tiles_events);
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, Reflect, Event, )]
pub struct ProcessedTiles { pub chunk: Entity, pub tiles: OplistCollectedTiles }





#[derive(serde::Deserialize, Event, serde::Serialize, Clone, MapEntities, Reflect)]
pub struct NewlyRegPos (#[entities] pub Entity, pub (DimensionRef, GlobalTilePos));

// No olvidarse de agregarlo al Plugin del módulo

// ----------------------> NO OLVIDARSE DE AGREGARLO AL Plugin DEL MÓDULO <-----------------------------
//                                                       ^^^^
#[allow(unused_parens)]
pub fn sync_register_new_pos(
    trigger: Trigger<NewlyRegPos>,
    mut own_map: ResMut<RegisteredPositions>,
    loaded_chunks: Res<LoadedChunks>,
    state: Res<State<GameSetupType>>,

) {
    let is_host = state.get() != &GameSetupType::AsJoiner;

    if is_host { return; }

    let registered_positions = trigger.event().clone();

    let (dim, glob_pos) = registered_positions.1;

    own_map.0.entry(registered_positions.0).or_default().push((dim, glob_pos));

    let chunk_pos: ChunkPos = glob_pos.into();

    if let Some(loaded) = loaded_chunks.0.get(&(dim, chunk_pos)) {
        
    } 

}
