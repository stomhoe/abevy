#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
use common::{common_components::*, common_states::AppState};
use serde::{Deserialize, Serialize};



#[derive(Component, Debug, Reflect)]
pub struct Mine;


#[derive(Debug, Component, Default, Serialize, Deserialize)]
#[require(Replicated, Prefix::trunc("Player"), SparedFromHotReloading, )]
pub struct Player;

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, )]
pub struct TrustedForUnaCosa;

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, )]
pub struct TrustedForOtracosa;

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, )]
pub struct TrustedMovement;

#[derive(Debug, Component, Default, Serialize, Deserialize)]
#[require(Player)]
pub struct HostPlayer;
