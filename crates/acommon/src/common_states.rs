#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;

#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default, Reflect, )]
#[states(scoped_entities)]//el default es para que el despawnonexit se active al salir de ese estado, cambiar el estado inicial en .insert_state::<AppState>
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
    NotStarted,

    /// asset loading is triggered
    LoadingReplicatedCollections,
    #[default]//default is for DespawnOnExit<AssetLoading>
    /// init systems are executed
    SpawnReplicatedEntities,
}




#[allow(unused_parens, dead_code)]
#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default, Reflect, )]
#[reflect(State, Default)]
#[states(scoped_entities,)]
pub enum TerrainHotReloading {#[default]KeepAlive, DespawnAll,}

