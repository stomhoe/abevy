
use std::hash::{Hash, Hasher};

use bevy::{ecs::{query, relationship::Relationship}, log::tracing_subscriber::layer, math::U8Vec2, platform::collections::{HashMap, HashSet}, prelude::*};
use bevy_ecs_tilemap::tiles::*;

use debug_unwraps::DebugUnwrapExt;
use fastnoise_lite::FastNoiseLite;
use rand::SeedableRng;
use rand_pcg::Pcg64;
use rand_seeder::Seeder;


use crate::game::tilemap::{chunking_components::*, chunking_resources::CHUNK_SIZE, terrain_gen::{terrgen_components::*, terrgen_events::*, terrgen_resources::*, terrgen_utils::* }, tile::tile_components::{AppliedShader, FlipAlongX, GlobalTilePos, MyTileBundle, RepeatingTexture, TileWeightedSampler, Tileimg}, };

#[derive(Component, Debug, Default, )]
pub struct TemperateGrass;

#[allow(unused_parens)]
pub fn setup(mut cmd: Commands, query: Query<(),()>, asset_server: Res<AssetServer>, ) 
{
    //TODO cargar todo esto de ficheros

    //HACER Q CADA UNA DE ESTAS ENTITIES APAREZCA EN LOS SETTINGS EN SETUP Y SEA CONFIGURABLE
    let humidity = FastNoiseLite::default();

    let temp_variation = FastNoiseLite::default();

    let continent = FastNoiseLite::default();
    let laker = FastNoiseLite::default();

    
    let bush0 = cmd.spawn((
        MyTileBundle {
            name: Name::new("bush0"),
            img_id: Tileimg::new(&asset_server, "bush/bush0.png"),
            ..Default::default()
        },
        FlipAlongX,
    )).id();


     let bush1 = cmd.spawn((
        MyTileBundle {
            name: Name::new("bush1"),
            img_id: Tileimg::new(&asset_server, "bush/bush1.png"),
            ..Default::default()
        },
        FlipAlongX,
    )).id();
    info!("bush0: {:?}, bush1: {:?}", bush0, bush1);

    let wmap = cmd.spawn(TileWeightedSampler::new(&[(bush0, 3.5), (bush1, 7.5)])).id();
    info!("wmap: {:?}", wmap);
    
    let grasstile_ent = cmd.spawn(( 
        MyTileBundle {
            name: Name::new("tempgrass"),
            img_id: Tileimg::new(&asset_server, "white.png"),
            color: TC_RED,
            shader: AppliedShader::MonoRepeating(
                RepeatingTexture::new_w_red_mask(
                    &asset_server,
                    "texture/world/terrain/temperate_grass/grass.png", 
                    1_000, //scale to be divided by 1M
                ),
            ),
            ..Default::default()
        },
        TemperateGrass,
    )).id();



    let continent = cmd.spawn(TgenNoise::new(continent)).id();
    let humidity = cmd.spawn(TgenNoise::new(humidity)).id();
    let temp_variation = cmd.spawn(TgenNoise::new(temp_variation)).id();

    let land_ops = cmd.spawn(
        OperationList {
            trunk: vec![
                (Operand::Zero, Operation::GetTiles(ProducedTiles::new([grasstile_ent]))),
                (Operand::PoissonDisk(Default::default()), Operation::Assign),
                (Operand::Value(0.5), Operation::GreaterThan(OnCompareConfig {
                    tiles_on_success: ProducedTiles::new([wmap]),
                    ..Default::default()
                })),
            ],
            bifurcation_over: None,
            bifurcation_under: None,
            threshold: 0.5,
        }
    ).id();

    cmd.spawn((
        OperationList {
            trunk: vec![
                (Operand::Entity(continent), Operation::Add),
            ],
            bifurcation_over: Some(land_ops),
            bifurcation_under: None,
            threshold: 0.5,
        },
        RootOpList
    ));

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

        if let Ok((mut pending_ops_count, mut tiles, &chunk_pos)) = chunk_query.get_mut(chunk_ref.0) {
            let mut acc_val: f32 = input_operand.0; 
            //cmd.entity(enti).remove::<InputOperand>();
            cmd.entity(enti).despawn();//NO PONER ABAJO

            let oplist = oplist_query.get(oplist_ref.0)?;

            let pos_within_chunk = global_tile_pos.get_pos_within_chunk(chunk_pos);
            //info!("Producing tiles at {:?} with oplist {:?}", pos_within_chunk, oplist_ref.0);

            for ((operand, operation)) in oplist.trunk.iter() {
                
                let operand = match operand {
                    Operand::Entity(ent) => {
                                        if let Ok((fnl_comp, )) = operands.get(*ent) {
                                            fnl_comp.map_or(0.0, |fnl| fnl.get_val(global_tile_pos))
                                        } else {
                                            warn!("Entity {:?} in Operand not found ", ent);
                                            0.0 // Si no hay componente, asumimos 0
                                        }
                                    },
                    Operand::Value(val) => *val,
                    Operand::HashPos => global_tile_pos.normalized_hash_value(&gen_settings),
                    Operand::Zero => 0.0,
                    Operand::PoissonDisk(poisson_disk) => poisson_disk.sample(&gen_settings, global_tile_pos),
                };

                match operation {
                    Operation::Add => acc_val += operand,
                    Operation::Subtract => acc_val -= operand,
                    Operation::Multiply => acc_val *= operand,
                    Operation::Divide => if operand != 0.0 { acc_val /= operand },
                    Operation::Min => acc_val = acc_val.min(operand),
                    Operation::Max => acc_val = acc_val.max(operand),
                    Operation::Pow => if acc_val >= 0.0 || operand.fract() == 0.0 { acc_val = acc_val.powf(operand) },
                    Operation::Modulo => if operand != 0.0 { acc_val = acc_val % operand },
                    Operation::Log => if acc_val > 0.0 && operand > 0.0 && operand != 1.0 { acc_val = acc_val.log(operand) },
                    Operation::GreaterThan(conf) => {
                        if acc_val > operand {
                            conf.tiles_on_success.insert_cloned_with_pos(&mut cmd, &mut tiles, global_tile_pos, pos_within_chunk, &weight_maps, &gen_settings);
                            match &conf.on_success {
                                NextAction::Continue => {},
                                NextAction::Break => break,
                                NextAction::OverwriteAcc(val) => acc_val = *val,
                            }
                        } else {
                            conf.tiles_on_failure.insert_cloned_with_pos(&mut cmd, &mut tiles, global_tile_pos, pos_within_chunk, &weight_maps, &gen_settings);
                            match &conf.on_failure {
                                NextAction::Continue => {},
                                NextAction::Break => break,
                                NextAction::OverwriteAcc(val) => acc_val = *val,
                            }
                        }
                    },
                    Operation::LessThan(conf) => {
                        if acc_val < operand {
                            conf.tiles_on_success.insert_cloned_with_pos(&mut cmd, &mut tiles, global_tile_pos, pos_within_chunk, &weight_maps, &gen_settings);
                            match &conf.on_success {
                                NextAction::Continue => {},
                                NextAction::Break => break,
                                NextAction::OverwriteAcc(val) => acc_val = *val,
                            }
                        } else {
                            conf.tiles_on_failure.insert_cloned_with_pos(&mut cmd, &mut tiles, global_tile_pos, pos_within_chunk, &weight_maps, &gen_settings);
                            match &conf.on_failure {
                                NextAction::Continue => {},
                                NextAction::Break => break,
                                NextAction::OverwriteAcc(val) => acc_val = *val,
                            }
                        }
                    },
                    Operation::Assign => {acc_val = operand;},
                    Operation::GetTiles(produced_tiles) => {
                        produced_tiles.insert_cloned_with_pos(&mut cmd, &mut tiles, global_tile_pos, pos_within_chunk, &weight_maps, &gen_settings);
                    },
                }
               
            }
            
            
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