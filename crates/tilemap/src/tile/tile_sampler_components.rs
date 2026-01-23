#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::Replicated;
use common::common_components::*;
use game_common::game_common_components::*;
use serde::{Deserialize, Serialize};
use crate::{
};


#[derive(Component, Debug, Default, Deserialize, Serialize, Copy, Clone, Reflect)]
#[require(Replicated, Prefix::trunc("TileWSampler"), )]
pub struct TileWeightedSampler;