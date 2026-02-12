#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::Replicated;
use common::common_components::*;
use serde::{Deserialize, Serialize};

#[derive(Component, Debug, Default, Copy, Clone, Deserialize, Serialize)]
#[require(AssetScoped, Replicated, Prefix::trunc("TileWSampler"), )]
pub struct TileWeightedSampler;
