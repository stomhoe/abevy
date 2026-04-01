use bevy::{ecs::entity::{EntityHashMap, MapEntities}, platform::collections::HashMap, prelude::*};
use bevy_replicon::prelude::Replicated;
use common::common_components::*;
use serde::{Deserialize, Serialize};
use tilemap_shared::*;

#[derive(Component, serde::Serialize, serde::Deserialize, Clone)]
#[require(Replicated, Prefix::trunc("Pack"), AssetScoped, HotReload)]
pub struct Pack;


#[derive(Component, Debug, Clone, MapEntities, Default)]
#[component(map_entities)]
pub struct BeingTemplateSampler(#[entities] pub EntityWeightedSampler);

#[derive(Component, Debug, Clone, Default)]
pub struct PackMemberRankSampler(pub EntityHashMap<CappedNormalDist>);

#[derive(Component, Debug, Clone, Default)]
pub struct CenterWeightRankBasedMultiplier(pub EntityHashMap<f32>);

#[derive(Component, Debug, Copy, Clone)]
pub struct GlobalCenterRankWeightMultiplier(pub f32);

#[derive(Component, Debug, Clone, Default)]
pub struct PackOnPreyedOnBehavior(pub StrId);

#[derive(Component, Debug, Copy, Clone)]
pub struct PackAttackAlertEffectivenessFalloff(pub f32);

#[derive(Component, Debug, Copy, Clone)]
pub struct PackCounterRegroupTightness(pub f32);

#[derive(Component, Debug, Clone, Default)]
pub struct PackMinSepToPacksOrRaces(pub EntityHashMap<u8>);
impl PackMinSepToPacksOrRaces {
    pub fn insert(&mut self, entity: Entity, min_inbetween_chunks: u8) {
        self.0.insert(entity, min_inbetween_chunks);
    }

    pub fn min_inbetween_chunks(&self, entity: Entity) -> u8 {
        self.configured_min_inbetween_chunks(entity).unwrap_or(1)
    }

    pub fn configured_min_inbetween_chunks(&self, entity: Entity) -> Option<u8> {
        self.0.get(&entity).copied()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

#[derive(Component, Debug, Clone)]
pub struct PackInitialSizeSampler(pub CappedNormalDist);
impl PackInitialSizeSampler {
    pub fn sample_count(&self, rng: &mut impl rand::Rng) -> usize {
        self.0.sample(rng).round().max(1.0) as usize
    }
}

#[derive(Component, Debug, Copy, Clone)]
pub struct PackSpawnRadius(pub u8);
impl Default for PackSpawnRadius {
    fn default() -> Self {
        Self(7)
    }
}
impl PackSpawnRadius {
    pub fn as_i32(&self) -> i32 {
        i32::from(self.0)
    }
}

#[derive(Component, Debug, Clone, Default, Deserialize, Serialize)]
pub struct SquadAvgCenterPerDim(pub HashMap<DimensionRef, GlobalTilePos>);

#[derive(Component, Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct PendingNaturalSpawnGroupId(pub u64);

#[derive(Component, Debug, Copy, Clone)]
pub struct PendingNaturalSpawnPlacementPolicy {
    pub anchor_gpos: GlobalTilePos,
    pub preferred_radius_tiles: u16,
    pub only_same_island: bool,
}
impl PendingNaturalSpawnPlacementPolicy {
    pub fn hard_max_radius_tiles_for_count(&self, count: usize) -> u16 {
        self.preferred_radius_tiles
            .saturating_add(count.saturating_sub(1) as u16)
    }
}

#[derive(Resource, Debug, Copy, Clone, Default)]
pub struct NextPendingNaturalSpawnGroupId(pub u64);
impl NextPendingNaturalSpawnGroupId {
    pub fn next(&mut self) -> u64 {
        let id = self.0;
        self.0 = self.0.saturating_add(1);
        id
    }
}
