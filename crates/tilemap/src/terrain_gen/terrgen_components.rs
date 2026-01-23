
#[allow(unused_imports)] use bevy::prelude::*;
use bevy_replicon::prelude::*;
use fnl::{FastNoiseLite, NoiseSampleRange};

use noiz::DynamicConfigurableSampleable;
use serde::{Deserialize, Serialize};
use tilemap_shared::{GlobalGenSettings, GlobalTilePos};
use std::hash::{Hasher, Hash};

use {common::common_components::*, };

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, Hash, PartialEq, Reflect)]
#[require(Replicated, SessionScoped, AssetScoped, TgenHotLoadingScoped, )]
pub struct Terrgen;

#[derive(Component, Default, Reflect, Serialize, Deserialize, PartialEq, Debug, Clone)]
#[require(Terrgen, Prefix::trunc("Noise"), )]
pub struct FnlNoiseComp(pub FastNoiseLite);
impl FnlNoiseComp {
    pub fn new(id: StrId) -> Self {
        Self(FastNoiseLite::new(id))
    }
    pub fn sample(&self, pos: GlobalTilePos, range: NoiseSampleRange, complementary: bool, extra_seed: i32, settings: &GlobalGenSettings) -> f32 {
        self.0.sample(pos.into(), range, complementary, extra_seed + settings.seed, settings.world_freq)
    }
}

#[derive(Component, Debug, Default, Deserialize, Serialize, Copy, Clone, Reflect)]
pub struct ComponentName{
    seed: i32,
}
impl ComponentName {
    pub fn new(id: HashId) -> Self {
        Self{ seed: id.as_i32()}
    }
}

//             .replicate::<NoizRef>()
#[derive(Component, )]
pub struct Noiz(pub Box<dyn DynamicConfigurableSampleable<Vec2, f32> + Send + Sync >);


#[derive(Component, Debug, Default, Deserialize, Serialize, Copy, Clone, Reflect)]
#[require(Replicated, Terrgen, Prefix::trunc("Noises"), AssetScoped, TgenHotLoadingScoped,SessionScoped )]
pub struct NoiseHolder;


#[derive(Component, Debug, Default, Deserialize, Serialize, Copy, Clone, Reflect)]
#[require(Replicated, Prefix::trunc("FailedSearches"), AssetScoped, TgenHotLoadingScoped,SessionScoped )]
pub struct FailedSearchOplistFilterHolder;