use bevy::prelude::*;

use crate::game::{tilemap::tilemap_syscomres::*, SimRunningSystems};

// Module tilemap
mod tilemap_syscomres;
//mod tilemap_events;

mod formation_generation;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct TilemapSystems;

pub struct MyTileMapPlugin;
#[allow(unused_parens)]
impl Plugin for MyTileMapPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(bevy_ecs_tilemap::TilemapPlugin)
            .add_systems(Update, (spawn_chunks_around_camera, despawn_outofrange_chunks).in_set(SimRunningSystems).in_set(TilemapSystems))
            .init_resource::<ChunkManager>()
            
        ;
    }
}