

use game_common::define_weightedsampler;
use ::tilemap_shared::*;

use bevy::{ecs::entity::MapEntities, prelude::*};
use bevy_replicon::prelude::Replicated;
use common::common_components::*;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
#[allow(unused_imports)] use bevy::prelude::*;
use std::hash::Hash;

define_weightedsampler!(ColorSampler, [u8; 4], "ColorWeightedSampler");
#[derive(Component, Debug, Deserialize, Serialize, Copy, Clone, Hash, PartialEq, Eq, Reflect, MapEntities)]
pub struct ColorSamplerRef(#[entities] pub Entity);