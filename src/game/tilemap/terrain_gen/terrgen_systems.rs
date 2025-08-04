

use bevy::{math::U8Vec2, prelude::*};

use fastnoise_lite::FastNoiseLite;



use crate::game::{game_resources::GlobalEntityMap, tilemap::{chunking_components::*, chunking_resources::CHUNK_SIZE, terrain_gen::{terrgen_components::*, terrgen_resources::* }, tile::tile_components::{GlobalTilePos, TileWeightedSampler, }, }};

#[derive(Component, Debug, Default, )]
pub struct TemperateGrass;

#[allow(unused_parens)]
pub fn init_noises(
    mut cmd: Commands, mut seris_handles: ResMut<NoiseSerisHandles>,
    mut assets: ResMut<Assets<NoiseSerialization>>, mut map: ResMut<TerrGenEntityMap>,
) {
    for handle in std::mem::take(&mut seris_handles.handles) {
        if let Some(seri) = assets.remove(&handle) {
            info!(target: "tiling_loading", "Loading TileSeri from handle: {:?}", handle);
            map.new_noise_ent_from_seri(&mut cmd, seri, );
        }
    }
} 

#[allow(unused_parens)]
pub fn init_oplists(
    mut cmd: Commands, mut seris_handles: ResMut<OpListSerisHandles>,
    mut assets: ResMut<Assets<OpListSeri>>, mut map: ResMut<OpListEntityMap>,
    terr_gen_map: Res<TerrGenEntityMap>,
    mut oplist_query: Query<(&mut OperationList, Option<&RootOpList>)>,
) -> Result {
    for handle in seris_handles.handles.iter() {
        if let Some(seri) = assets.get(handle) {
            info!(target: "oplist_loading", "Loading OpListSeri from handle: {:?}", handle);
            map.new_oplist_ent_from_seri(&mut cmd, seri, &terr_gen_map);
        } 
    }
    for handle in std::mem::take(&mut seris_handles.handles) {
        if let Some(seri) = assets.remove(&handle) {
            if let Some(&oplist_entity) = map.0.get(&seri.id) {
                map.set_bifurcations(
                    seri, oplist_entity, &mut oplist_query,
                )?;
            } else {
                error!(target: "oplist_loading", "OpListSeri with id {} not found in map", seri.id);
            }
        }
    }
    Ok(())
} 

 
#[allow(unused_parens)]
pub fn setup(mut cmd: Commands, query: Query<(),()>, asset_server: Res<AssetServer>, ) 
{
    // TODO cargar todo esto de ficheros

    // HACER Q CADA UNA DE ESTAS ENTITIES APAREZCA EN LOS SETTINGS EN SETUP Y SEA CONFIGURABLE

    // PARA HACER ISLAS CON FORMA CUSTOM (P. EJ CIRCULAR O DISCO O ALGO RARO Q NO SE PUEDE HACER CON NOISE), MARCAR EN UN PUNTO EXTREMADAMENTE OCÉANICO CON UNA TILE MARKER Y DESP HACER OTRO SISTEMA Q LO PONGA TODO POR ENCIMA, SOBREESCRIBIENDO LO Q HABÍA ANTES

    // let land_ops = cmd.spawn(
    //     OperationList {
    //         trunk: vec![
    //             (Operand::Zero, Operation::GetTiles(ProducedTiles::new([grasstile_ent]))),
    //             (Operand::PoissonDisk(Default::default()), Operation::Assign),
    //             (Operand::Value(0.5), Operation::GreaterThan(OnCompareConfig {
    //                 tiles_on_success: ProducedTiles::new([wmap]),
    //                 ..Default::default()
    //             })),
    //         ],
    //         bifurcation_over: None,
    //         bifurcation_under: None,
    //         threshold: 0.5,
    //     }
    // ).id();

    // cmd.spawn((
    //     OperationList {
    //         trunk: vec![
    //             (Operand::Entity(continent), Operation::Add),
    //         ],
    //         bifurcation_over: Some(land_ops),
    //         bifurcation_under: None,
    //         threshold: 0.5,
    //     },
    //     RootOpList
    // ));

    //DimensionRef usarlo para algo con las operations nase

}



#[allow(unused_parens)]
pub fn spawn_terrain_operations (
    mut commands: Commands, 
    //TODO EN VEZ DE USAR UNA QUERY, HACER UNA LLAMADA DIRECTA. PONER LOS ARGS EN UN STRUCT
    chunks_query: Query<(Entity, &ChunkPos), (With<UninitializedChunk>, Without<PendingOperations>, Without<ProducedTiles>, Without<Children>)>, 
    //gen_settings: Res<WorldGenSettings>,
    oplists: Query<(Entity, ), (With<OperationList>,  With<RootOpList>)>,
) -> Result {

    //crear entities comúnes de tiles acá o en setup

    for (chunk_ent, chunk_pos) in chunks_query.iter() {

        //SE LES PODRÍA AGREGAR MARKER COMPONENTS A LOS CHUNKS PARA POR EJEMPLO ESPECIFICAR SI ES UN DUNGEON
        //EN ESTE PUNTO SE PODRÍA GENERAR UN CAMINO RANDOM QUE SEA UN VEC DE COORDS, Y DESPUES PASARLO ABAJO Y Q SE OCUPEN?? PA GENERAR DUNGEONS NASE

        let mut pending_ops_count: i32 = 0;
        for x in 0..CHUNK_SIZE.x { 
            for y in 0..CHUNK_SIZE.y {
                let pos_within_chunk = U8Vec2::new(x, y);
                for (oplist_ent, ) in oplists.iter() {
                    commands.spawn((
                        OplistRef(oplist_ent),
                        ChunkRef(chunk_ent),
                        chunk_pos.to_tilepos() + GlobalTilePos(pos_within_chunk.as_ivec2()),
                    ));
                    pending_ops_count += 1;
                }
            }
        } 
        commands.entity(chunk_ent).insert((PendingOperations(pending_ops_count), ProducedTiles::new_with_chunk_capacity()));

    }
    Ok(())
}

#[allow(unused_parens)]
pub fn produce_tiles(mut cmd: Commands, 
    gen_settings: Res<WorldGenSettings>,
    mut query: Query<(Entity, &InputOperand, &OplistRef, &ChunkRef, &GlobalTilePos), (Added<InputOperand>, )>, 
    oplist_query: Query<(&OperationList ), ( )>,
    operands: Query<(Option<&TgenNoise>, ), ( )>,
    mut chunk_query: Query<(&mut PendingOperations, &mut ProducedTiles, &ChunkPos), ( )>,
    weight_maps: Query<(&TileWeightedSampler, ), ( )>,
    //mut wmap_query: Query<(Option<&WeightedMap<Entity>>, ), ( )>,
) -> Result {
    for (enti, &input_operand, &oplist_ref, &chunk_ref, &global_tile_pos) in query.iter_mut() {

        if let Ok((mut pending_ops_count, mut chunk_tiles, &chunk_pos)) = chunk_query.get_mut(chunk_ref.0) {
            let mut acc_val: f32 = input_operand.0; 
            //cmd.entity(enti).remove::<InputOperand>();
            cmd.entity(enti).despawn();//NO PONER ABAJO

            let oplist = oplist_query.get(oplist_ref.0)?;

            let pos_within_chunk = global_tile_pos.get_pos_within_chunk(chunk_pos);
            //info!("Producing tiles at {:?} with oplist {:?}", pos_within_chunk, oplist_ref.0);

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
                    Operand::HashPos => global_tile_pos.normalized_hash_value(&gen_settings),
                    Operand::PoissonDisk(poisson_disk) => poisson_disk.sample(&gen_settings, global_tile_pos),
                    _ => {
                        error!("Unsupported operand type as numeric value: {:?}", operand);
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

            chunk_tiles.insert_clonespawned_with_pos(&oplist.tiles, &mut cmd, global_tile_pos, pos_within_chunk, &weight_maps, &gen_settings);
            
            
            if acc_val > oplist.threshold {
                if let Some(bifover_ent) = oplist.bifurcation_over {
                    cmd.spawn((OplistRef(bifover_ent), InputOperand(acc_val), chunk_ref.clone(), global_tile_pos));
                    //cmd.entity(enti).despawn();//NO PONER ABAJO
                    continue;
                }
            }
            else {
                if let Some(bifunder_ent) = oplist.bifurcation_under {
                    cmd.spawn((OplistRef(bifunder_ent), InputOperand(acc_val), chunk_ref.clone(), global_tile_pos));
                    //cmd.entity(enti).despawn();//NO PONER ABAJO
                    continue;
                }
            }

            pending_ops_count.0 -= 1;
            if pending_ops_count.0 == 0 {
                cmd.entity(chunk_ref.0).remove::<PendingOperations>().insert(TilesReady);
            }
        }
    }
    Ok(())
}