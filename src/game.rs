use bevy::{
    prelude::*,
};
use crate::game::setup_menus::SetupMenusPlugin;
use crate::AppState;
use crate::game::game_systems::*;

pub mod player;
mod setup_menus;
mod game_systems;
mod tilemap;
mod beings;
mod things;
mod server;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct ConfinementSystemSet;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct MovementSystemSet;
pub struct GamePlugin;
impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {

        app
            .add_plugins(SetupMenusPlugin)

            .configure_sets(Update, MovementSystemSet.before(ConfinementSystemSet).run_if(in_state(SimulationState::Running)))
            .add_systems(OnEnter(GamePhase::InGame),  (spawn_player,))

            .add_systems(Update, toggle_simulation.run_if(in_state(GamePhase::InGame)))

            .init_state::<SimulationState>()
            .init_state::<GamePhase>()
            .init_state::<GameMp>()
            .init_state::<SelfMpKind>()
        ;
    }
}

#[derive(SubStates, Debug, Clone, PartialEq, Eq, Hash, Default)]
#[source(AppState = AppState::GameDomain)]
#[states(scoped_entities)]
pub enum GamePhase {#[default]Setup, InGame,}

#[derive(SubStates, Debug, Clone, PartialEq, Eq, Hash, Default)]
#[source(GamePhase = GamePhase::InGame)]
#[states(scoped_entities)]
enum SimulationState {#[default]Running, Paused,}

#[derive(SubStates, Debug, Clone, PartialEq, Eq, Hash, Default)]
#[source(AppState = AppState::GameDomain)]
#[states(scoped_entities)]
pub enum GameMp {#[default]Singleplayer, Multiplayer,}


#[derive(SubStates, Debug, Clone, PartialEq, Eq, Hash, Default)]
#[source(GameMp = GameMp::Multiplayer)]
#[states(scoped_entities)]
pub enum SelfMpKind {#[default]Host, Client,}

