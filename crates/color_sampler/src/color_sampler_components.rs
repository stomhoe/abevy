

use game_common::define_weightedsampler;
use ::tilemap_shared::*;

#[allow(unused_imports)]
use bevy::{ecs::entity::MapEntities, prelude::*};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

define_weightedsampler!(ColorSampler, [u8; 4], "ColorWeightedSampler");
