
#[allow(unused_imports)] use bevy::prelude::*;
use bevy_replicon::prelude::*;
use fnl::{FastNoiseLite, NoiseSampleRange};

use noiz::DynamicConfigurableSampleable;

use tilemap_shared::{GlobalGenSettings, GlobalTilePos};
use std::hash::{Hash};

use {common::common_components::*, };
use serde::{Deserialize, Serialize};

#[derive(Component, Debug, Default, Clone, Hash, PartialEq, Serialize, Deserialize)]
pub struct Terrgen;

#[derive(Component, Default, Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct FnlNoiseComp { pub fnl: FastNoiseLite, pub is_tect: bool }
impl FnlNoiseComp {
    pub fn new(id: StrId) -> Self {
        Self { fnl: FastNoiseLite::new(id), is_tect: false }
    }
    pub fn sample(&self, pos: GlobalTilePos, dim_hash_id: HashId, range: NoiseSampleRange, complementary: bool, extra_seed: i32, settings: &GlobalGenSettings) -> f32 {
        let world_frequency = if self.is_tect {
            settings.world_freq * settings.tectonic_frequency
        } else {
            settings.world_freq
        };
        self.fnl.sample(pos.into(), range, complementary, extra_seed.wrapping_add(settings.seed).wrapping_add(dim_hash_id.as_i32()), world_frequency)
    }
}

//             .replicate::<NoizRef>()
#[derive(Component)]
pub struct Noiz(pub Box<dyn DynamicConfigurableSampleable<Vec2, f32> + Send + Sync >);

#[derive(Component, Debug, Default, Copy, Clone)]
#[require(Replicated, Prefix::trunc("FailedPosSearches"), AssetScoped, )]
pub struct FailedSearchOplistFilterHolder;
