
#[allow(unused_imports)] use bevy::prelude::*;
use bevy_replicon::prelude::*;
use fnl::{FastNoiseLite, NoiseSampleRange};

use noiz::DynamicConfigurableSampleable;

use tilemap_shared::{GlobalGenSettings, GlobalTilePos};
use std::hash::{Hash};

use {common::common_components::*, };
use serde::{Deserialize, Serialize};

#[derive(Component, Debug, Default, Clone, Hash, PartialEq, Serialize, Deserialize)]
#[require(AssetScoped, Replicated, )]
pub struct Terrgen;

#[derive(Component, Default, Serialize, Deserialize, PartialEq, Debug, Clone)]
#[require(Terrgen, Prefix::trunc("Noise"), HotReload, )]
pub struct FnlNoiseComp(pub FastNoiseLite);
impl FnlNoiseComp {
    pub fn new(id: StrId) -> Self {
        Self(FastNoiseLite::new(id))
    }
    pub fn sample(&self, pos: GlobalTilePos, dim_hash_id: HashId, range: NoiseSampleRange, complementary: bool, extra_seed: i32, settings: &GlobalGenSettings) -> f32 {
        self.0.sample(pos.into(), range, complementary, extra_seed.wrapping_add(settings.seed).wrapping_add(dim_hash_id.as_i32()), settings.world_freq)
    }
}

//             .replicate::<NoizRef>()
#[derive(Component)]
pub struct Noiz(pub Box<dyn DynamicConfigurableSampleable<Vec2, f32> + Send + Sync >);

#[derive(Component, Debug, Default, Copy, Clone)]
#[require(Replicated, Prefix::trunc("FailedPosSearches"), AssetScoped, )]
pub struct FailedSearchOplistFilterHolder;
