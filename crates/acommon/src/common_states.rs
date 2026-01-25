#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;

#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default, Reflect, )]
#[states(scoped_entities)]//el default es para que el despawnonexit se active al salir de ese estado
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

// #[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default, Reflect, )]
// #[states(scoped_entities)]
// pub enum ConnectionAttempt {#[default]Not, Triggered, PostAttempt,}

#[allow(unused_parens, dead_code)]
#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default, Reflect, )]
#[states(scoped_entities,)]
pub enum AssetLoading {
    #[default]
    NotStarted,

    /// Asset loading is triggered
    LoadingAssetsIntoHandles,
    

    /// Init systems which spawn entities are executed
    SpawnReplicatedEntities,
}




#[allow(unused_parens, dead_code)]
#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default, Reflect, )]
#[reflect(State, Default)]
#[states(scoped_entities,)]
pub enum AssetHotReloadState {#[default]Stopped, Ongoing,}

