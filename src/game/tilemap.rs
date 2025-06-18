use bevy::prelude::*;

// Module tilemap
mod tilemap_syscomres;
//mod tilemap_events;

mod formation_generation;

pub struct TileMapPlugin;
#[allow(unused_parens)]
impl Plugin for TileMapPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(bevy_ecs_tilemap::TilemapPlugin)
            //.add_systems(Update, (somesystem))
            //.add_systems(OnEnter(SomeStateType::Literal), (setup))
            //.add_systems(Update, somesystem.runif(in_state(SomeStateType::Literal)))
        ;
    }
}