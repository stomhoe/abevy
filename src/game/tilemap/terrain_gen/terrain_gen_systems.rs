
use bevy::{ecs::query, log::tracing_subscriber::layer, math::U8Vec2, platform::collections::{HashMap, HashSet}, prelude::*};
use bevy_ecs_tilemap::tiles::*;

use debug_unwraps::DebugUnwrapExt;
use fastnoise_lite::FastNoiseLite;

use crate::game::tilemap::{terrain_gen::{terrain_gen_components::*, terrain_gen_resources::*, terrain_gen_utils::UniqueTileDto}, tile_imgs::*, tilemap_components::*, chunking_resources::CHUNK_SIZE};

// NO OLVIDARSE DE INICIALIZARLO EN EL Plugin DEL MÓDULO
#[derive(Resource, Default)]

pub struct TileInstantationDataMap {//TODO faltan templates para habilitar la randomizacion entre tiles del mismo tipo
    pub data: HashMap<u32, Entity>,
}



#[allow(unused_parens)]
pub fn setup(mut commands: Commands, query: Query<(),()>, world_settings: Res<WorldGenSettings>, _asset_server: Res<AssetServer>) {

    let humidity: FastNoiseLite = FastNoiseLite::default();

    let grass_instantiation_data = TileInstantiationData {
        image_nid: IMG_WHITE,
        color: Color::srgb(1.0, 0.0, 0.0),
        used_shader: UsedShader::Grass,
        ..Default::default()
    };

    commands.spawn(grass_instantiation_data);

    //TODO instanciar todas las instancias de noise
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
    tile_insta_data_query: Query<Entity, With<TileInstantiationData>>,
) {

    //crear entities comúnes de tiles acá o en setup

    for (chunk_ent, chunk_pos) in chunks_query.iter() {

        let mut tiles_ready = TilesReady(Vec::new());
        
        for x in 0..CHUNK_SIZE.x { 
            for y in 0..CHUNK_SIZE.y {
                let pos_within_chunk = U8Vec2::new(x, y);
                let tilepos = chunk_pos.to_tilepos() + pos_within_chunk.as_ivec2();
                add_tiles_for_tilepos( &mut commands, &mut tiles_ready, noise_query, tilepos, pos_within_chunk, tile_insta_data_query);
        }} 


        commands.entity(chunk_ent).insert(tiles_ready);
    }
}

fn add_tiles_for_tilepos(mut commands: &mut Commands, tiles2spawn: &mut TilesReady, 
    noise_query: Query<&FnlComp>, tilepos: IVec2, pos_within_chunk: U8Vec2,
    tile_instantiation_data: Query<Entity, With<TileInstantiationData>>,
) {unsafe {

    //si una tile es suitable para una edificación, o spawnear una village o algo, se le puede añadir un componente SuitableForVillage o algo así, para que se pueda identificar la tile. después se puede hacer un sistema q borre los árboles molestos en un cierto radio. el problema es si hay múltiples marcadas adyacentemente, en ese caso va a haber q chequear distancias a otras villages
    let tile_inst_data_entity = tile_instantiation_data.single().debug_expect_unchecked("hola");
    let tile_entity = commands.spawn_empty().id();
    
    
    tiles2spawn.0.push(UniqueTileDto::new(tile_entity, pos_within_chunk, tile_inst_data_entity)
    
);   
}}

const TC_RED: TileColor = TileColor(Color::srgb(1., 0., 0.));
const TC_GREEN: TileColor = TileColor(Color::srgb(0., 1., 0.));
const TC_BLUE: TileColor = TileColor(Color::srgb(0., 0., 1.));


