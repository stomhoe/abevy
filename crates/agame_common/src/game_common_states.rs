use bevy::prelude::*;
use common::states::{AppState, GamePhase};



#[derive(SubStates, Debug, Clone, PartialEq, Eq, Hash, Default)]
#[source(GamePhase = GamePhase::Setup)]
#[states(scoped_entities)]
pub enum GameSetupScreen {#[default]GameSettings, CharacterCreation,}

#[derive(SubStates, Debug, Clone, PartialEq, Eq, Hash, Default)]
#[source(GamePhase = GamePhase::ActiveGame)]
#[states(scoped_entities)]
pub enum SimulationState {#[default]Running, Paused,}