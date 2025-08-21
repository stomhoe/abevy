
use bevy::ecs::entity::MapEntities;
#[allow(unused_imports)] use bevy::prelude::*;
use bevy_replicon::prelude::*;
use fnl::{FastNoiseLite, NoiseSampleRange};

use noiz::DynamicConfigurableSampleable;
use serde::{Deserialize, Serialize};
use std::hash::{Hasher, Hash};
use std::collections::hash_map::DefaultHasher;

use crate::terrain_gen::terrgen_resources::AaGlobalGenSettings;
use crate::tile::tile_components::*;

use {common::common_components::*, };
use strum_macros::{AsRefStr, Display, };
use std::ops::{Index, IndexMut};

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, Hash, PartialEq, Reflect)]
#[require(Replicated, SessionScoped, AssetScoped, TgenScoped, )]
pub struct TerrGen;

#[derive(Component, Default, Reflect, Serialize, Deserialize, PartialEq, Debug, Clone)]
#[require(TerrGen, EntityPrefix::new("Noise"), )]
pub struct FnlNoise(pub FastNoiseLite);
impl FnlNoise {
    pub fn new(id: StrId) -> Self {
        Self(FastNoiseLite::new(id))
    }
    pub fn sample(&self, pos: GlobalTilePos, range: NoiseSampleRange, complementary: bool, extra_seed: i32, settings: &AaGlobalGenSettings) -> f32 {
        self.0.sample(pos.into(), range, complementary, extra_seed + settings.seed, settings.world_freq)
    }
}


//            .replicate::<NoizRef>()
#[derive(Component, )]
pub struct Noiz(pub Box<dyn DynamicConfigurableSampleable<Vec2, f32> + Send + Sync >);

