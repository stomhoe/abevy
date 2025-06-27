
use std::hash::{Hash, Hasher};

use bevy::{ecs::query, log::tracing_subscriber::layer, math::U8Vec2, platform::collections::{HashMap, HashSet}, prelude::*};
use bevy_ecs_tilemap::tiles::*;

use debug_unwraps::DebugUnwrapExt;
use fastnoise_lite::FastNoiseLite;
use rand::SeedableRng;
use rand_pcg::Pcg64;
use rand_seeder::Seeder;

use crate::game::tilemap::{terrain_gen::{terrain_gen_components::*, terrain_gen_resources::*, }, tile_imgs::*, chunking_components::*, chunking_resources::CHUNK_SIZE};

// NO OLVIDARSE DE INICIALIZARLO EN EL Plugin DEL MÓDULO
#[derive(Resource, Default)]
pub struct TilesDataMap {//TODO faltan templates para habilitar la randomizacion entre tiles del mismo tipo
    pub data: HashMap<u32, Entity>,//
}//NO SÉ SI USAR ESTO O DIRECTAMENTE PONERLE MARKER COMPONENTS A LOS ENTITIES DE TILE INSTANTIATION DATA

#[allow(unused_parens)]
pub fn setup(mut commands: Commands, query: Query<(),()>, world_settings: Res<WorldGenSettings>, _asset_server: Res<AssetServer>) {

    let humidity: FastNoiseLite = FastNoiseLite::default();

    // let grass_instantiation_data = TileInstantiationData {
    //     image_nid: IMG_WHITE,
    //     color: Color::srgb(1.0, 0.0, 0.0),
    //     used_shader: UsedShader::Grass,
    //     ..Default::default()
    // };

    // commands.spawn(grass_instantiation_data);

    //TODO instanciar todas las instancias de noise y configurarlas acá 
    commands.spawn(FnlComp(humidity));


    //TODO hallar punto del terreno con 
}



#[allow(unused_parens)]
pub fn add_tiles2spawn_within_chunk (
    mut commands: Commands, 
    //TODO EN VEZ DE USAR UNA QUERY, HACER UNA LLAMADA DIRECTA. PONER LOS ARGS EN UN STRUCT
    chunks_query: Query<(Entity, &ChunkPos), (With<UninitializedChunk>, Without<TilesReady>, Without<Children>)>, 
    noise_query: Query<&FnlComp>, 
    gen_settings: Res<WorldGenSettings>,
    clonable_tiles: Query<Entity, With<TileImgNid>>,
    //tile_insta_data_query: Query<Entity, With<TileInstantiationData>>,
) -> Result {

    //crear entities comúnes de tiles acá o en setup

    for (chunk_ent, chunk_pos) in chunks_query.iter() {

        let mut tiles_ready = TilesReady(Vec::new());

        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        chunk_pos.0.hash(&mut hasher);
        let mut rng: Pcg64 = Seeder::from(gen_settings.seed + (hasher.finish() as u32)).into_rng();
        
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

fn add_tiles_for_tilepos(mut co: &mut Commands, tiles2spawn: &mut TilesReady, 
    noise_query: Query<&FnlComp>, tilepos: IVec2, pos_within_chunk: U8Vec2, 
    clonable_tiles: Query<Entity, With<TileImgNid>>,
    rng : &mut Pcg64,

) -> Result {

    //si una tile es suitable para una edificación, o spawnear una village o algo, se le puede añadir un componente SuitableForVillage o algo así, para que se pueda identificar la tile. después se puede hacer un sistema q borre los árboles molestos en un cierto radio. el problema es si hay múltiples marcadas adyacentemente, en ese caso va a haber q chequear distancias a otras villages
    let tile_entity = new_tile(co, pos_within_chunk, MyTileBundle::new(IMG_WHITE, TileFlip::default(), TC_RED, TileVisible(true), UsedShader::Grass));
    
    
    tiles2spawn.0.push(tile_entity);
    
    Ok(()) 
}

const TC_RED: TileColor = TileColor(Color::srgb(1., 0., 0.));
const TC_GREEN: TileColor = TileColor(Color::srgb(0., 1., 0.));
const TC_BLUE: TileColor = TileColor(Color::srgb(0., 0., 1.));

fn add_tilepos_and_push(
    commands: &mut Commands, 
    tiles2spawn: &mut TilesReady,
    pos_within_chunk: U8Vec2, 
    entity: Entity,
) {
    commands.entity(entity).insert((
        TilePos::new(pos_within_chunk.x as u32, pos_within_chunk.y as u32),
    ));
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

#[derive(Bundle, Debug, Default, )]
pub struct MyTileBundle{
    pub img_nid: TileImgNid,
    pub flip: TileFlip,
    pub color: TileColor,
    pub visible: TileVisible,
    pub shader: UsedShader,
}
impl MyTileBundle {
    pub fn new(img_nid: TileImgNid, flip: TileFlip, color: TileColor, visible: TileVisible, shader: UsedShader) -> Self {
        Self { img_nid, flip, color, visible, shader }
    }
}