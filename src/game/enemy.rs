use bevy::app::{App, Plugin, Update};
use bevy::prelude::*;
use crate::AppState;
use crate::game::enemy::systems::{enemy_movement, spawn_enemies, update_enemy_direction};
use crate::game::SimulationState;

mod components;
mod systems;

const ENEMY_SIZE: f32 = 64.0;

const NUMBER_OF_ENEMIES: i32 = 10;

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(OnEnter(AppState::Game), spawn_enemies)
            .add_systems(Update, (update_enemy_direction, enemy_movement).run_if(in_state(SimulationState::Running)))
        ;
    }
}
