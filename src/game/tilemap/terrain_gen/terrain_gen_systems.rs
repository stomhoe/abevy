
use bevy::{ecs::query, log::tracing_subscriber::layer, platform::collections::{HashMap, HashSet}, prelude::*};
use bevy_ecs_tilemap::{map::*, prelude::{MaterialTilemap, MaterialTilemapHandle}, tiles::*, MaterialTilemapBundle, TilemapBundle};

use crate::game::tilemap::{terrain_gen::{terrain_gen_components::*, terrain_gen_resources::*, terrain_gen_utils::{TileInstantiationData, UniqueTileDto}}, tile_imgs::NidImgMap, tilemap_components::{ ChunkPos, UninitializedChunk}, tilemap_resources::*};
use fastnoise_lite::FastNoiseLite;




#[allow(unused_parens)]
pub fn setup(mut commands: Commands, query: Query<(),()>, world_settings: Res<WorldGenSettings>, asset_server: Res<AssetServer>) {

    let humidity: FastNoiseLite = FastNoiseLite::default();



    //TODO instanciar todas las instancias de noise
    commands.spawn(FnlComp(humidity));


    //TODO hallar punto del terreno con 
}



#[allow(unused_parens)]
pub fn add_tiles2spawn_within_chunk (
    mut commands: Commands, 
    chunks_query: Query<(Entity, &ChunkPos), (With<UninitializedChunk>, Without<TilesReady>, Without<Children>)>, 
    noise_query: Query<&FnlComp>, 
    gen_settings: Res<WorldGenSettings>,
) {

    //crear entities comúnes de tiles acá o en setup

    for (chunk_ent, chunk_pos) in chunks_query.iter() {

        let mut tiles_ready = TilesReady(Vec::new());
        
        for x in 0..CHUNK_SIZE.x { 
            for y in 0..CHUNK_SIZE.y {
                let pos_within_chunk = UVec2::new(x, y);
                let tilepos = chunkpos_to_tilepos(chunk_pos.0) + pos_within_chunk.as_ivec2();
                add_tiles_for_tilepos( &mut commands, &mut tiles_ready, noise_query, tilepos, pos_within_chunk);
            }} 
        commands.entity(chunk_ent).insert(tiles_ready);
    }
}

fn add_tiles_for_tilepos(mut commands: &mut Commands, tiles2spawn: &mut TilesReady, 
    noise_query: Query<&FnlComp>, tilepos: IVec2, pos_within_chunk: UVec2,
) {

    //si una tile es suitable para una edificación, o spawnear una village o algo, se le puede añadir un componente SuitableForVillage o algo así, para que se pueda identificar la tile. después se puede hacer un sistema q borre los árboles molestos en un cierto radio. el problema es si hay múltiples marcadas adyacentemente, en ese caso va a haber q chequear distancias a otras villages

   
    //tiles2spawn.push(tileinfo);   
}

const TC_RED: TileColor = TileColor(Color::srgb(1., 0., 0.));
const TC_GREEN: TileColor = TileColor(Color::srgb(0., 1., 0.));
const TC_BLUE: TileColor = TileColor(Color::srgb(0., 0., 1.));


