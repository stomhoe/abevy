use bevy::prelude::*;
use common::common_states::*;
use serde::{Deserialize, Serialize};



#[derive(SubStates, Debug, Clone, PartialEq, Eq, Hash, Default)]
#[source(GamePhase = GamePhase::Setup)]
#[states(scoped_entities)]
pub enum GameSetupScreen {#[default]GameSettings, CharacterCreation,}

#[derive(SubStates, Debug, Clone, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[source(GamePhase = GamePhase::ActiveGame)]
#[states(scoped_entities)]
pub enum SimulationState {#[default]Running, Paused,}
impl SimulationState {
    pub fn is_running(&self) -> bool {
        matches!(self, SimulationState::Running)
    }
    pub fn is_paused(&self) -> bool {
        matches!(self, SimulationState::Paused)
    }
}
