#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;


#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default, Reflect, )]
#[states(scoped_entities)]
pub enum AppState {NoSession, #[default]StatefulGameSession, }




#[derive(SubStates, Debug, Clone, PartialEq, Eq, Hash, Default, Reflect, )]
#[source(AppState = AppState::NoSession)]
#[states(scoped_entities)]
pub enum PreGameState {
    #[default]
    MainMenu,
    Settings
}


#[derive(SubStates, Debug, Clone, PartialEq, Eq, Hash, Default, Reflect, )]
#[source(AppState = AppState::StatefulGameSession)]
#[states(scoped_entities)]
pub enum GamePhase {#[default]Setup, ActiveGame,}

#[derive(SubStates, Debug, Clone, PartialEq, Eq, Hash, Default, Reflect, )]
#[source(GamePhase = GamePhase::Setup)]
#[states(scoped_entities)]
pub enum GameSetupType {#[default]Singleplayer, AsHost, AsJoiner,}

#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default, Reflect, )]
#[states(scoped_entities)]
pub enum ConnectionAttempt {#[default]Not, Triggered, PostAttempt,}

#[allow(unused_parens, dead_code)]
#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default, Reflect, )]
#[states(scoped_entities,)]
pub enum AssetsLoadingState {
    NotStarted,//HACER EL DEFAULT ESTE SI SE QUIERE HACER ALGO ANTES DE CARGAR LOS ASSETS
    #[default]
    LocalInProcess,
    LocalFinished,
    ReplicatedInProcess,
    ReplicatedFinished,
}


#[allow(unused_parens, dead_code)]
#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default, Reflect, )]
#[states(scoped_entities,)]
pub enum LoadedAssetsSession {#[default]KeepAlive, DespawnAll,}

#[allow(unused_parens, dead_code)]
#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default, Reflect, )]
#[states(scoped_entities,)]
pub enum ReplicatedAssetsSession {#[default]KeepAlive, DespawnLocalAssets,}

#[allow(unused_parens, dead_code)]
#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default, Reflect, )]
#[reflect(State, Default)]
#[states(scoped_entities,)]
pub enum TerrainGenHotLoading {#[default]KeepAlive, DespawnAll,}

