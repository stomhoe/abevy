
use std::hash::{Hash, Hasher};

use bevy::{ecs::{query, relationship::Relationship}, log::tracing_subscriber::layer, math::U8Vec2, platform::collections::{HashMap, HashSet}, prelude::*};
use bevy_ecs_tilemap::tiles::*;

use debug_unwraps::DebugUnwrapExt;
use fastnoise_lite::FastNoiseLite;
use rand::SeedableRng;
use rand_pcg::Pcg64;
use rand_seeder::Seeder;


use crate::game::tilemap::{chunking_components::*, chunking_resources::CHUNK_SIZE, terrain_gen::{terrain_gen_components::*, terrain_gen_resources::*, }, tile::tile_components::{AppliedShader, MyTileBundle, RepeatingTexture, Tileimg}, };

#[derive(Component, Debug, Default, )]
pub struct TemperateGrass;

#[allow(unused_parens)]
pub fn setup(mut commands: Commands, query: Query<(),()>, world_settings: Res<WorldGenSettings>, asset_server: Res<AssetServer>, ) 
{
    //HACER Q CADA UNA DE ESTAS ENTITIES APAREZCA EN LOS SETTINGS EN SETUP Y SEA CONFIGURABLE
    let humidity: FastNoiseLite = FastNoiseLite::default();

    let temp_variation: FastNoiseLite = FastNoiseLite::default();

    // commands.spawn(grass_instantiation_data);

    let tile_entity = commands.spawn(( 
        MyTileBundle {//TODO ponerle un componente Name de alguna forma???
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

    //TODO instanciar todas las instancias de noise y configurarlas acá 
    commands.spawn(FnlComp { noise: humidity, ..Default::default()});


    //TODO hallar punto del terreno con 
}

#[derive(Component, Debug, Default, )]
pub struct WorldTilePos(IVec2);

#[allow(unused_parens)]
pub fn add_tiles2spawn_within_chunk (
    mut commands: Commands, 
    //TODO EN VEZ DE USAR UNA QUERY, HACER UNA LLAMADA DIRECTA. PONER LOS ARGS EN UN STRUCT
    chunks_query: Query<(Entity, &ChunkPos), (With<UninitializedChunk>, Without<TilesReady>, Without<Children>)>, 
    noise_query: Query<&FnlComp>, 
    gen_settings: Res<WorldGenSettings>,
    clonable_tiles: Query<Entity, (With<Tileimg>, Without<TilePos>)>,
    //tile_insta_data_query: Query<Entity, With<TileInstantiationData>>,
) -> Result {

    //crear entities comúnes de tiles acá o en setup

    for (chunk_ent, chunk_pos) in chunks_query.iter() {

        //SE LES PODRÍA AGREGAR MARKER COMPONENTS A LOS CHUNKS PARA POR EJEMPLO ESPECIFICAR SI ES UN DUNGEON

        let mut tiles_ready = TilesReady(Vec::new());

        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        chunk_pos.0.hash(&mut hasher);
        let mut rng: Pcg64 = Seeder::from(gen_settings.seed + (hasher.finish() as i32)).into_rng();
        
        //EN ESTE PUNTO SE PODRÍA GENERAR UN CAMINO RANDOM QUE SEA UN VEC DE COORDS, Y DESPUES PASARLO ABAJO Y Q SE OCUPEN?? PA GENERAR DUNGEONS NASE

        let tiles_ready_expected_count = CHUNK_SIZE.x * CHUNK_SIZE.y;
 
        for x in 0..CHUNK_SIZE.x { 
            for y in 0..CHUNK_SIZE.y {
                let pos_within_chunk = U8Vec2::new(x, y);
                let tilepos = chunk_pos.to_tilepos() + pos_within_chunk.as_ivec2();
                add_tiles_for_tilepos( &mut commands, &mut tiles_ready, noise_query, tilepos, pos_within_chunk, clonable_tiles, &mut rng)?;
        }} 


        commands.entity(chunk_ent).insert(tiles_ready);
    }
    Ok(())
}

fn add_tiles_for_tilepos(mut cmd: &mut Commands, tiles2spawn: &mut TilesReady, 
    noise_query: Query<&FnlComp>, tilepos: IVec2, pos_within_chunk: U8Vec2, 
    mut clonable_tiles: Query<Entity, (With<Tileimg>, Without<TilePos>)>,
    rng : &mut Pcg64,

) -> Result {

    //si una tile es suitable para una edificación, o spawnear una village o algo, se le puede añadir un componente SuitableForVillage o algo así, para que se pueda identificar la tile. después se puede hacer un sistema q borre los árboles molestos en un cierto radio. el problema es si hay múltiples marcadas adyacentemente, en ese caso va a haber q chequear distancias a otras villages
    let mut grass = clonable_tiles.transmute_lens_filtered::<(Entity), (With<TemperateGrass>, Without<TilePos>)>();

    let grass = grass.query().single()?;

    clone_add_tilepos_and_push(
        &mut cmd, 
        tiles2spawn, 
        pos_within_chunk, 
        tilepos,
        grass, 
    );
    
    Ok(()) 
}

const TC_RED: TileColor = TileColor(Color::srgb(1., 0., 0.));
const TC_GREEN: TileColor = TileColor(Color::srgb(0., 1., 0.));
const TC_BLUE: TileColor = TileColor(Color::srgb(0., 0., 1.));

fn clone_add_tilepos_and_push(
    commands: &mut Commands, 
    tiles2spawn: &mut TilesReady,
    pos_within_chunk: U8Vec2, 
    tilepos: IVec2,
    entity: Entity,
) {
    let entity = commands.entity(entity).clone_and_spawn().insert((
        TilePos::new(pos_within_chunk.x as u32, pos_within_chunk.y as u32),
        WorldTilePos(tilepos),//NO SÉ SI METERLE ESTO O NO, PERO HACE CADA TILE MÁS FÁCILMENTE QUERYABLE POR DISTANCIA
    )).id();
    tiles2spawn.0.push(entity);
}


fn new_tile<B: Bundle>(
    commands: &mut Commands, 
    pos_within_chunk: U8Vec2, 
    bundle: B,
) -> Entity {
    commands.spawn((
        TilePos::new(pos_within_chunk.x as u32, pos_within_chunk.y as u32),
        bundle,
    )).id()
}

// ----------------------> NO OLVIDARSE DE AGREGARLO AL Plugin DEL MÓDULO <-----------------------------
//                                                       ^^^^
#[allow(unused_parens)]
pub fn produce_tiles(mut cmd: Commands, curr_tile: Res<CurrGenTileWorldPos>, 
    mut query: Query<(Entity, &FirstOperand, &OperationList, Option<&mut ProducedTiles>), ()>, vals: Query<&Operand>,
){
    if curr_tile.is_changed() {
        for (oplist_ent, first_operand, oplist, tiles) in query.iter_mut() {

            let tiles = if let Some(mut tiles) = tiles {
                std::mem::take(&mut *tiles)
            } else {
                ProducedTiles::default()
            };
            
            let mut acc_val: f32 = first_operand.0; 
            cmd.entity(oplist_ent).remove::<FirstOperand>();

            for ((ent, operation)) in oplist.trunk.iter() {

                if let Ok(&Operand(operand)) = vals.get(*ent) {

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
                                cmd.entity(oplist_ent).insert(conf.tiles_on_success.clone());
                                match &conf.on_success {
                                    NextAction::Continue => {},
                                    NextAction::Break => break,
                                    NextAction::OverwriteAcc(val) => acc_val = *val,
                                }
                            } else {
                                cmd.entity(oplist_ent).insert(conf.tiles_on_failure.clone());
                                match &conf.on_failure {
                                    NextAction::Continue => {},
                                    NextAction::Break => break,
                                    NextAction::OverwriteAcc(val) => acc_val = *val,
                                }
                            }
                        },
                        Operation::LessThan(conf) => {
                            if acc_val < operand {
                                cmd.entity(oplist_ent).insert(conf.tiles_on_success.clone());
                                match &conf.on_success {
                                    NextAction::Continue => {},
                                    NextAction::Break => break,
                                    NextAction::OverwriteAcc(val) => acc_val = *val,
                                }
                            } else {
                                cmd.entity(oplist_ent).insert(conf.tiles_on_failure.clone());
                                match &conf.on_failure {
                                    NextAction::Continue => {},
                                    NextAction::Break => break,
                                    NextAction::OverwriteAcc(val) => acc_val = *val,
                                }
                            }
                        },
                        Operation::Assign => {acc_val = operand;},
                    }
                } else {
                    warn!("Entity {:?} not found in CurrValue query", ent);
                    continue; // Si no se encuentra el valor, se salta a la siguiente iteración
                }
            }

            if acc_val > oplist.threshold {
                if let Some(bifover_ent) = oplist.bifurcation_over {
                    cmd.entity(bifover_ent).insert((FirstOperand(acc_val), tiles));
                    continue;
                }
            }
            else {
                if let Some(bifunder_ent) = oplist.bifurcation_under {
                    cmd.entity(bifunder_ent).insert((FirstOperand(acc_val), tiles));
                    continue;
                }
            }
            cmd.entity(oplist_ent).insert((tiles, Finished));
        }
    }

}


#[allow(unused_parens)]
pub fn update_noise_curr_value(curr_tile: Res<CurrGenTileWorldPos>, mut query: Query<(&mut Operand, &FnlComp)>) {
    for (mut curr_value, fnl_comp) in query.iter_mut() {
        let (noise, offset) = (&fnl_comp.noise, &fnl_comp.offset);
        curr_value.0 = noise.get_noise_2d(
            (curr_tile.0.x + offset.x) as f32, (curr_tile.0.y + offset.y) as f32,
        );
    }
}

#[allow(unused_parens)]
pub fn update_hashval(curr_tile: Res<CurrGenTileWorldPos>, world_settings: Res<WorldGenSettings>, 
    mut query: Query<(&mut Operand,), (With<HashPosComp>,)>) 
{
    for (mut curr_value,) in query.iter_mut() {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        curr_tile.0.hash(&mut hasher);
        world_settings.seed.hash(&mut hasher);
        let hash_val = hasher.finish();
        // Normalize to [0,1] using u64::MAX
        curr_value.0 = (hash_val as f64 / u64::MAX as f64) as f32;
    }
}