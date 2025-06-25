
use bevy::{ecs::query, log::tracing_subscriber::layer, platform::collections::{HashMap, HashSet}, prelude::*};
use bevy_ecs_tilemap::{map::*, prelude::{MaterialTilemap, MaterialTilemapHandle}, tiles::*, MaterialTilemapBundle, TilemapBundle};

use crate::game::tilemap::{terrain_gen::{terrain_gen_components::*, terrain_gen_resources::*, terrain_gen_utils::TileDto}, tilemap_components::Chunk, tilemap_resources::*};
use fastnoise_lite::FastNoiseLite;


#[allow(unused_parens)]
pub fn setup(mut commands: Commands, query: Query<(),()>, world_settings: Res<WorldGenSettings>) {

    let humidity: FastNoiseLite = FastNoiseLite::default();


    //TODO instanciar todas las instancias de noise
    commands.spawn((
        FnlComp (humidity),
    ));


    //TODO hallar punto del terreno con 
}


