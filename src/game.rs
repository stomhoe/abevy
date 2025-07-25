#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
#[allow(unused_imports)] use superstate::superstate_plugin;
use crate::game::being::movement::MovementSystems;
use crate::game::being::race::race_resources::RaceSerisHandles;
use crate::game::being::sprite::animation_resources::AnimSerisHandles;
use crate::game::being::sprite::sprite_resources::SpriteSerisHandles;
use crate::game::being::{BeingsPlugin, };
use crate::game::multiplayer::MpPlugin;
use crate::game::time::ClockPlugin;
use crate::game::faction::FactionsPlugin;
use crate::game::player::{PlayerInputSystems, PlayerPlugin};
use crate::game::setup_menus::SetupMenusPlugin;
use crate::game::tilemap::MyTileMapPlugin;
use crate::AppState;
use crate::game::game_systems::*;
use bevy_asset_loader::prelude::*;

pub mod player;
pub mod setup_menus;
pub mod multiplayer;
pub mod game_utils;

mod game_systems;
mod game_components;
mod game_resources;

mod tilemap;
pub mod being;
mod things;
mod faction;
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
pub struct GameDataInitSystems;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct HostSystems;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct ClientSystems;

pub struct GamePlugin;

#[allow(unused_parens, )]
impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {

        app
            .add_plugins((MpPlugin, SetupMenusPlugin, PlayerPlugin, BeingsPlugin, 
                FactionsPlugin, MyTileMapPlugin, ClockPlugin, ))
            
            .add_systems(OnEnter(GamePhase::ActiveGame), (
                (spawn_player_beings,).run_if(server_or_singleplayer),
            ))

            .add_systems(Update, 
                (debug_system, toggle_simulation, force_z_index).in_set(IngameSystems)
            )

            // .configure_sets(OnEnter(AppState::StatefulGameSession), (
            //     GameDataInitSystems.run_if(server_or_singleplayer)
            // ))

            .configure_sets(Update, (
                PlayerInputSystems,
                MovementSystems,
                IngameSystems.run_if(in_state(GamePhase::ActiveGame)),
                SimRunningSystems.run_if(in_state(SimulationState::Running).and(in_state(GamePhase::ActiveGame))),
                SimPausedSystems.run_if(in_state(SimulationState::Paused).and(in_state(GamePhase::ActiveGame))),
                HostSystems.run_if(server_or_singleplayer.or(in_state(GameSetupType::HostLobby))),
                ClientSystems.run_if(not(server_or_singleplayer).and(in_state(GameSetupType::JoinerLobby))),
            ))
            
            .configure_sets(FixedUpdate, (
                HostSystems.run_if(server_or_singleplayer.or(in_state(GameSetupType::HostLobby))),
                ClientSystems.run_if(not(server_or_singleplayer).and(in_state(GameSetupType::JoinerLobby))),
                SimRunningSystems.run_if(in_state(SimulationState::Running).and(in_state(GamePhase::ActiveGame))),
            ))

            .init_state::<SimulationState>()
            .init_state::<GamePhase>()
            .init_state::<GameSetupType>()
            .init_state::<GameSetupScreen>()
            .init_state::<AssetLoadingState>()

            .add_loading_state(
                LoadingState::new(AssetLoadingState::InProcess)
                .continue_to_state(AssetLoadingState::Complete)
                .load_collection::<SpriteSerisHandles>()
                .load_collection::<AnimSerisHandles>()
                .load_collection::<RaceSerisHandles>()
            )
        ;
    }
}

#[derive(SubStates, Debug, Clone, PartialEq, Eq, Hash, Default)]
#[source(AppState = AppState::StatefulGameSession)]
#[states(scoped_entities)]
pub enum GamePhase {#[default]Setup, ActiveGame,}

#[derive(SubStates, Debug, Clone, PartialEq, Eq, Hash, Default)]
#[source(GamePhase = GamePhase::Setup)]
#[states(scoped_entities)]
enum GameSetupScreen {#[default]GameSettings, CharacterCreation,}

#[derive(SubStates, Debug, Clone, PartialEq, Eq, Hash, Default)]
#[source(GamePhase = GamePhase::ActiveGame)]
#[states(scoped_entities)]
enum SimulationState {#[default]Running, Paused,}

#[derive(SubStates, Debug, Clone, PartialEq, Eq, Hash, Default)]
#[source(GamePhase = GamePhase::Setup)]
#[states(scoped_entities)]
pub enum GameSetupType {#[default]Singleplayer, HostLobby, JoinerLobby,}


//usar server_or_singleplayer y not(server_or_singleplayer) para distinguir entre servidor y cliente

#[allow(unused_parens, dead_code)]
#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
#[states(scoped_entities,)]
enum AssetLoadingState {
    NotStarted,//HACER EL DEFAULT ESTE SI SE QUIERE HACER ALGO ANTES DE CARGAR LOS ASSETS
    #[default]
    InProcess,
    Complete,
}