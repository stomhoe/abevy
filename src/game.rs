#[allow(unused_imports)]
use bevy::prelude::*;
#[allow(unused_imports)]
use bevy_replicon::prelude::*;
#[allow(unused_imports)]
use superstate::superstate_plugin;

use bevy_asset_loader::prelude::*;

use crate::{
    game::{
        being::{
            being_components::Being,
            movement::MovementSystems,
            race::race_resources::RaceSerisHandles,
            sprite::{
                animation_resources::AnimSerisHandles,
                sprite_resources::{SpriteDataIdEntityMap, SpriteSerisHandles},
            },
            BeingsPlugin,
        }, dimension::{dimension_resources::DimensionEntityMap, DimensionPlugin}, faction::FactionPlugin, game_components::FacingDirection, game_resources::*, game_systems::*, multiplayer::{ClientSystems, MpPlugin}, player::{PlayerInputSystems, PlayerPlugin}, setup_menus::SetupMenusPlugin, tilemap::{terrain_gen::terrgen_resources::{NoiseSerisHandles, OpListEntityMap, OpListSerisHandles, TerrGenEntityMap}, tile::tile_resources::*, ChunkSystems, MyTileMapPlugin}, time::ClockPlugin
    }, AppState
};

pub mod player;
pub mod setup_menus;
pub mod multiplayer;
pub mod game_utils;
pub mod being;
pub mod dimension;

mod game_systems;
mod game_components;
mod game_resources;

mod tilemap;
mod things;
mod faction;
mod time;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
struct SimRunningSystems;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
struct SimPausedSystems;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
struct StatefulSessionSystems;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
struct ActiveGameSystems;


#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct GameDataInitSystems;



pub struct GamePlugin;

#[allow(unused_parens, )]
impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {

        app
            .add_plugins((SetupMenusPlugin, PlayerPlugin, BeingsPlugin, 
                FactionPlugin, DimensionPlugin, MyTileMapPlugin, ClockPlugin, ))

            .add_systems(OnEnter(AppState::StatefulGameSession), (
                (server_or_singleplayer_setup,).run_if(server_or_singleplayer),
            ))
            
            .add_systems(OnEnter(GamePhase::ActiveGame), (
                (spawn_player_beings,).run_if(server_or_singleplayer),
            ))
            .add_systems(OnEnter(AppState::PreGame), (
                remove_server_resources,
            ))

            .add_systems(Update, (
                update_img_sizes_on_load,
                host_on_player_added.run_if(server_or_singleplayer),
                (
                    toggle_simulation, update_transform_z, tick_time_based_multipliers,
                
                ).in_set(ActiveGameSystems),
            ))

            .configure_sets(Update, (
                PlayerInputSystems,
                MovementSystems,

                ActiveGameSystems.run_if(in_state(GamePhase::ActiveGame)),
                SimRunningSystems.run_if(in_state(SimulationState::Running).and(in_state(GamePhase::ActiveGame))),
                SimPausedSystems.run_if(in_state(SimulationState::Paused).and(in_state(GamePhase::ActiveGame))),

                ClientSystems.run_if(not(server_or_singleplayer).or(in_state(GameSetupType::JoinerLobby))),

                ChunkSystems.in_set(ActiveGameSystems).in_set(SimRunningSystems)


            ))
            
            .configure_sets(FixedUpdate, (
                ClientSystems.run_if(not(server_or_singleplayer).or(in_state(GameSetupType::JoinerLobby))),//NO TOCAR
                SimRunningSystems.run_if(in_state(SimulationState::Running).and(in_state(GamePhase::ActiveGame))),
            ))
            .init_resource::<ImageSizeMap>().init_resource::<GlobalEntityMap>()

            .init_state::<LocalAssetsLoadingState>()
            .init_state::<ReplicatedAssetsLoadingState>()
            .init_state::<GamePhase>()
            .init_state::<GameSetupType>()
            .init_state::<GameSetupScreen>()
            .init_state::<SimulationState>()

            .add_loading_state(
                LoadingState::new(LocalAssetsLoadingState::InProcess).continue_to_state(LocalAssetsLoadingState::Complete)
                .load_collection::<SpriteSerisHandles>()
                .load_collection::<AnimSerisHandles>()
                .load_collection::<RaceSerisHandles>()
                .load_collection::<ShaderRepeatTexSerisHandles>()
                .load_collection::<TileSerisHandles>()
                .finally_init_resource::<SpriteDataIdEntityMap>()
                .finally_init_resource::<TilingEntityMap>()
                .finally_init_resource::<TileShaderEntityMap>()
            )
            .add_loading_state(
                LoadingState::new(ReplicatedAssetsLoadingState::InProcess).continue_to_state(ReplicatedAssetsLoadingState::Complete)
                .load_collection::<NoiseSerisHandles>()
                .load_collection::<OpListSerisHandles>()
                //.load_collection::<DimensionSerisHandles>()
                .load_collection::<TileWeightedSamplerSerisHandles>()
                .finally_init_resource::<TerrGenEntityMap>()
                .finally_init_resource::<OpListEntityMap>()
                .finally_init_resource::<DimensionEntityMap>()
            )
            
            //https://github.com/NiklasEi/bevy_asset_loader?tab=readme-ov-file
            //https://github.com/NiklasEi/bevy_asset_loader/blob/main/bevy_asset_loader/examples/finally_init_resource.rs

            .replicate_bundle::<(Being, ChildOf)>()//NO FUNCIONA BIEN LO DE CHILDOF
            .replicate_bundle::<(Being, FacingDirection)>()//PROVISORIO, VA A HABER Q REVISAR


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


#[allow(unused_parens, dead_code)]
#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
#[states(scoped_entities,)]
enum LocalAssetsLoadingState {
    NotStarted,//HACER EL DEFAULT ESTE SI SE QUIERE HACER ALGO ANTES DE CARGAR LOS ASSETS
    #[default]
    InProcess,
    Complete,
}


#[allow(unused_parens, dead_code)]
#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
#[states(scoped_entities,)]
enum ReplicatedAssetsLoadingState {
    #[default]
    NotStarted,//HACER EL DEFAULT ESTE SI SE QUIERE HACER ALGO ANTES DE CARGAR LOS ASSETS
    InProcess,
    Complete,
}