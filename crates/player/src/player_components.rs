#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
use common::{common_components::*, };
use serde::{Deserialize, Serialize};

#[derive(Component, Debug, Clone)]
pub struct Mine;

#[derive(Debug, Component, Default, Serialize, Deserialize, Clone)]
#[require(Replicated, Prefix::trunc("Player"), SparedFromHotReloading, )]
pub struct Player;

#[derive(Component, Debug, Default, Clone)]
pub struct TrustedForUnaCosa;

#[derive(Component, Debug, Default, Clone)]
pub struct TrustedForOtracosa;

#[derive(Component, Debug, Default, Clone)]
pub struct TrustedForMovement;

#[derive(Debug, Component, Default, Serialize, Deserialize, Clone)]
#[require(Player)]
pub struct HostPlayer;
