use bevy::{
    prelude::*,
};
use crate::AppState;
use crate::game::enemy::EnemyPlugin;
use crate::game::star::StarPlugin;
use crate::game::game_systems::*;

pub mod enemy;
pub mod star;
mod game_systems;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct ConfinementSystemSet;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct MovementSystemSet;
pub struct GamePlugin;
impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {

        app
            .configure_sets(Update, MovementSystemSet.before(ConfinementSystemSet).run_if(in_state(SimulationState::Running)))
            .add_plugins((StarPlugin, EnemyPlugin))
            .add_systems(OnEnter(AppState::Game),  (spawn_player, spawn_camera))

            .add_systems(Update,( keyboard_input.in_set(MovementSystemSet), confine_player_to_window.in_set(ConfinementSystemSet)))
            .add_systems(Update, toggle_simulation.run_if(in_state(AppState::Game)))
            .init_state::<SimulationState>()

        ;
    }
}



#[derive(Component, Default)]
#[require(Sprite, StateScoped::<AppState>)]
pub struct Player {
}
#[derive(SubStates, Debug, Clone, PartialEq, Eq, Hash, Default)]
#[source(AppState = AppState::Game)]
#[states(scoped_entities)]
enum SimulationState {
    #[default]
    Paused,
    Running,
}