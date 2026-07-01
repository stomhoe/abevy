use bevy::{ecs::entity::EntityHashMap, platform::collections::HashMap, prelude::*};
use common::common_components::*;
use serde::{Deserialize, Serialize};
use tilemap_shared::*;

#[derive(Component, Serialize, Deserialize, Clone)]
#[require(Prefix::trunc("Pack"), AssetScoped, SelectedForHotReload)]
pub struct Pack;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct PackSystems;

#[derive(Component, Debug, Clone, Default)]
pub struct PackRaceOrBitSampler(pub HashIdWeightedSampler);

#[derive(Component, Debug, Clone, Default)]
pub struct PackMemberRankSampler(pub EntityHashMap<CappedNormalDist>);

#[derive(Component, Debug, Clone, Default)]
pub struct PackRaceOrBitSpawnQuotas(pub EntityHashMap<(u32, u32)>);

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

#[derive(Component, Debug, Clone, Default, )]
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

common::define_entity_map_systems_no_replicate!(
    main_component: Pack,
    with_filters: (With<game_common::game_common_components::Templ>, ),
    abbreviation: Pack,
    target: "pack",
    entity_prefix: "Pack",
    despawn_trigger: (Pack, game_common::game_common_components::Templ),
    id_type: common::common_components::StrId,
);
