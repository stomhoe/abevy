use bevy::{ecs::entity::MapEntities, prelude::*};
use bevy_replicon::prelude::*;
use serde::{Deserialize, Serialize};

use crate::tilemap_shared_samplers::GlobalTilePosWeightedSampler;

#[derive(Component, Deserialize, Asset, TypePath, Clone, Debug, Default)]
pub struct PortalSeri {
    pub dest_dimension: String,
    pub oe_tile: String,
    #[serde(default)]
    pub oe_terrprobe: String,
    #[serde(default)]
    pub one_way: bool,
    #[serde(default)]
    pub dungeon: String,
    #[serde(default)]
    pub offset_pos_destinations: Vec<(f32, (i8, i8))>,
}
impl PortalSeri {
    pub fn no_field_is_empty(&self) -> bool {
        !self.dest_dimension.is_empty() && !self.oe_terrprobe.is_empty()
    }
}

#[derive(Component, Debug, Clone, )]
pub struct PortalRecipe {
    #[entities]
    pub dest_dimension: Entity,
    #[entities]
    pub oe_portal_tile: Entity,
    #[entities]
    pub terrprobe_ent: Entity,
    pub one_way: bool,
    pub sampler: GlobalTilePosWeightedSampler,
}

#[derive(Component, Debug, Clone, Deserialize, Serialize, MapEntities)]
pub struct PortalTo {
    #[entities]
    pub dest_tile: Entity,
    #[serde(skip)]
    pub offset_pos_destinations: GlobalTilePosWeightedSampler
}
impl PortalTo {
    pub fn new(dest_portal: Entity, offset_pos_destinations: GlobalTilePosWeightedSampler) -> Self {
        Self { dest_tile: dest_portal, offset_pos_destinations }
    }
}

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone)]
#[require(Replicated, common::AssetScoped, common::Prefix::trunc("EguiPortalsZeroHolder"), Transform, Visibility)]
pub struct PortalsZeroEguiHolder;
