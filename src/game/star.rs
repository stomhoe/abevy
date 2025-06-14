use bevy::prelude::*;
use crate::AppState;
use crate::game::SimulationState;
use crate::game::star::components::*;
use crate::game::star::events::*;
use crate::game::star::resources::*;
use crate::game::star::systems::*;

mod systems;
pub mod components;
pub mod resources;
pub mod events;

const STAR_SPAWN_INTERVAL: f32 = 10.0;
const NUMBER_OF_STARS: i32 = 100;

pub struct StarPlugin;

impl Plugin for StarPlugin{
    fn build(&self, app: &mut App){
        app
            .init_resource::<Score>()
            .init_resource::<StarSpawnTimer>()
            .add_event::<GameOver>()
            .add_systems(Update, (update_score, tick_start_spawn_timer, spawn_stars_over_time, player_hit_star).run_if(in_state(SimulationState::Running)))
            .add_systems(OnEnter(AppState::Game), spawn_stars);
    }
}