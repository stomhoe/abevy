use bevy::app::FixedMain;
use bevy::prelude::*;
use crate::game::beings::{BeingsPlugin, MovementSystems};
use crate::game::tilemap::terrain_gen::TerrainGenSystems;
use crate::game::time::ClockPlugin;
use crate::game::factions::FactionsPlugin;
use crate::game::player::{PlayerInputSystems, PlayerPlugin};
use crate::game::setup_menus::SetupMenusPlugin;
use crate::game::tilemap::{MyTileMapPlugin, TilemapsSystems};
use crate::AppState;
use crate::game::game_systems::*;

pub mod player;
mod setup_menus;
mod game_systems;
mod game_components;
mod game_resources;
mod tilemap;
mod beings;
mod things;
mod server;
mod factions;
mod time;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct SimRunningSystems;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct SimPausedSystems;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct IngameSystems;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct NetworkSystems;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct HostOnlySystems;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct ClientSystems;

pub struct GamePlugin;
impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {

        app
            .add_plugins((SetupMenusPlugin, PlayerPlugin, BeingsPlugin, FactionsPlugin, MyTileMapPlugin, ClockPlugin, ))
            
            .add_systems(OnEnter(GamePhase::InGame),  (spawn_player_beings,))

            .add_systems(Update, 
                
                (debug_system, toggle_simulation, force_z_index).in_set(IngameSystems)
            )

            .configure_sets(Update, (
                PlayerInputSystems.before(MovementSystems),//NO SÉ SI HACE FALTA, MEJOR SACARLO PARA Q VAYA MÁS RÁPIDO
                IngameSystems.run_if(in_state(GamePhase::InGame)),
                SimRunningSystems.run_if(in_state(SimulationState::Running).and(in_state(GamePhase::InGame))),
                SimPausedSystems.run_if(in_state(SimulationState::Paused).and(in_state(GamePhase::InGame))),
                HostOnlySystems.run_if((in_state(SelfMpKind::Host).and(in_state(GameMp::Multiplayer))).or(in_state(GameMp::Singleplayer))),
                ClientSystems.run_if(in_state(SelfMpKind::Client).and(in_state(GameMp::Multiplayer))),
            ))

            .configure_sets(FixedUpdate, (
                NetworkSystems.run_if(in_state(GameMp::Multiplayer)),
                ClientSystems.run_if(in_state(SelfMpKind::Client).and(in_state(GameMp::Multiplayer))),

            ))

            .init_state::<SimulationState>()
            .init_state::<GamePhase>()
            .init_state::<GameMp>()
            .init_state::<SelfMpKind>()

            .insert_resource(Time::<Fixed>::from_seconds(0.1))
            
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

