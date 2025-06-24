use std::mem;

use bevy::{ecs::query, log::tracing_subscriber::layer, platform::collections::{HashMap, HashSet}, prelude::*};
use bevy_ecs_tilemap::{map::*, prelude::{MaterialTilemap, MaterialTilemapHandle}, tiles::*, MaterialTilemapBundle, TilemapBundle};

use crate::game::tilemap::{formation_generation::{formation_generation_components::*, formation_generation_resources::*}, tilemap_components::{Chunk, TilesInstantiated}, tilemap_resources::{*}};


#[allow(unused_parens)]
pub fn setup(mut commands: Commands, query: Query<(),()>, world_settings: Res<WorldGenSettings>) {

    //TODO instanciar todas las instancias de noise
}















