#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;

#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default, Reflect, )]
#[states(scoped_entities)]//el default es para que el despawnonexit se active al salir de ese estado, cambiar el estado inicial en .insert_state::<AppState>
pub enum AppState {NoSession, #[default]StatefulGameSession, }

#[derive(SubStates, Debug, Clone, PartialEq, Eq, Hash, Default, Reflect, )]
#[source(AssetLoading = AssetLoading::SpawnLocalEntities)]
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
    //asset loading is triggered
    LocalInProcess,
    //init systems are executed
    SpawnLocalEntities,
    //asset loading is triggered
    LoadingReplicatedCollections,
    //init systems are executed
    #[default]//el default es para que el despawnonexit se active al salir de ese estado, cambiar el estado inicial en .insert_state::<AssetsLoadingState>
    SpawnReplicatedEntities,
}


#[allow(unused_parens, dead_code)]
#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default, Reflect, )]
#[reflect(State, Default)]
#[states(scoped_entities,)]
/// usar esto para el despawnonexit porque el client no tiene el state en replicatedfinished
pub enum ReplicatedAssetsSession {#[default]KeepAlive, DespawnAll,}

#[allow(unused_parens, dead_code)]
#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default, Reflect, )]
#[reflect(State, Default)]
#[states(scoped_entities,)]
pub enum TerrainHotReloading {#[default]KeepAlive, DespawnAll,}

