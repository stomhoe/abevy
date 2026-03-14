#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::Replicated;
use common::common_components::*;
use game_common::game_common_samplers::EntityWeightedSampler;
use serde::{Deserialize, Serialize};

#[derive(Component, Debug, Default, Copy, Clone, Deserialize, Serialize)]
#[require(AssetScoped, Replicated, Prefix::trunc("Biome"), HotReload)]
pub struct Biome;

#[derive(Component, Debug, Clone, Default, Deserialize, Serialize, bevy::ecs::entity::MapEntities)]
#[component(map_entities)]
pub struct BiomePackSampler(#[entities] pub EntityWeightedSampler);
