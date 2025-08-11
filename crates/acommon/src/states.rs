#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;


#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
#[states(scoped_entities)]
pub enum AppState {#[default]PreGame, StatefulGameSession, }

#[derive(SubStates, Debug, Clone, PartialEq, Eq, Hash, Default)]
#[source(AppState = AppState::PreGame)]
#[states(scoped_entities)]
pub enum PreGameState {
    #[default]
    MainMenu,
    Settings
}


#[derive(SubStates, Debug, Clone, PartialEq, Eq, Hash, Default)]
#[source(AppState = AppState::StatefulGameSession)]
#[states(scoped_entities)]
pub enum GamePhase {#[default]Setup, ActiveGame,}

#[derive(SubStates, Debug, Clone, PartialEq, Eq, Hash, Default)]
#[source(GamePhase = GamePhase::Setup)]
#[states(scoped_entities)]
pub enum GameSetupType {#[default]Singleplayer, HostLobby, JoinerLobby,}

#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
#[states(scoped_entities)]
pub enum ConnectionAttempt {#[default]Not, Triggered, PostAttempt,}

#[allow(unused_parens, dead_code)]
#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
#[states(scoped_entities,)]
pub enum LocalAssetsLoadingState {
    NotStarted,//HACER EL DEFAULT ESTE SI SE QUIERE HACER ALGO ANTES DE CARGAR LOS ASSETS
    #[default]
    InProcess,
    Finished,
}


#[allow(unused_parens, dead_code)]
#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
#[states(scoped_entities,)]
pub enum ReplicatedAssetsLoadingState {#[default]NotStarted, InProcess, Finished,}

