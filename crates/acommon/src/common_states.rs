#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;

#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default, )]
#[states(scoped_entities)]//el default es para que el despawnonexit se active al salir de ese estado
pub enum AppState {NoSession, #[default]StatefulGameSession, }

#[derive(SubStates, Debug, Clone, PartialEq, Eq, Hash, Default, )]
#[source(AppState = AppState::NoSession)]
#[states(scoped_entities)]
pub enum PreGameState {
    #[default]
    MainMenu,
    Settings
}

#[derive(SubStates, Debug, Clone, PartialEq, Eq, Hash, Default, )]
#[source(AppState = AppState::StatefulGameSession)]
#[states(scoped_entities)]
pub enum GamePhase {#[default]Setup, ActiveGame,}

// #[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default, Reflect, )]
// #[states(scoped_entities)]
// pub enum ConnectionAttempt {#[default]Not, Triggered, PostAttempt,}

#[allow(unused_parens, dead_code)]
#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default, )]
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
#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default, )]
#[states(scoped_entities,)]
pub enum AssetHotReloadState {#[default]Stopped, Ongoing,}

#[derive(Resource, Debug, Clone)]
pub struct HotReloadSelection {
    pub tiles: bool,
    pub sprite_configs_and_animations: bool,
    pub terrain_oplists_and_noises: bool,
    pub probes_and_filters: bool,
    pub terrgen_settings: bool,
    pub beings_inst_templates: bool,
    pub races: bool,
    pub sexes: bool,
}
impl Default for HotReloadSelection {
    fn default() -> Self {
        Self {
            tiles: false,
            sprite_configs_and_animations: true,
            terrain_oplists_and_noises: true,
            probes_and_filters: true,
            terrgen_settings: true,
            beings_inst_templates: false,
            races: false,
            sexes: false,
        }
    }
}

#[derive(Resource, Debug, Default, Clone)]
pub struct HotReloadRequest {
    pub requested: bool,
}
