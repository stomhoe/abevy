use bevy::{ecs::entity::MapEntities, platform::collections::HashSet, prelude::*};
use common::common_tag_components::TagSet;
use serde::{Deserialize, Serialize};

#[derive(Component, Debug, Deserialize, Serialize, Clone)]
pub struct PredatorCfg {
    pub territorialism: f32,
    pub pack_size_min: u32,
    pub pack_size_max: u32,
    pub do_not_hunt_tags: TagSet,
    pub prey_body_size_ratio_tolerance: f32,
    pub min_hunger_to_hunt: f32,
    pub min_hp_ratio_to_hunt: f32,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PredatorSeri {
    #[serde(default)]
    pub territorialism: f32,
    #[serde(default)]
    pub pack_size_min: u32,
    #[serde(default)]
    pub pack_size_max: u32,
    #[serde(default)]
    pub do_not_hunt_tags: HashSet<String>,
    #[serde(default)]
    pub prey_body_size_ratio_tolerance: f32,
    #[serde(default = "default_predator_seri_uninitialized")]
    pub min_hunger_to_hunt: f32,
    #[serde(default)]
    pub min_hp_ratio_to_hunt: f32,
}

impl PredatorSeri {
    pub const SERI_UNINITIALIZED: f32 = f32::NEG_INFINITY;

    pub fn is_uninitialized(&self) -> bool {
        self.min_hunger_to_hunt == Self::SERI_UNINITIALIZED
    }
}

impl Default for PredatorSeri {
    fn default() -> Self {
        Self {
            territorialism: 0.0,
            pack_size_min: 1,
            pack_size_max: 1,
            do_not_hunt_tags: HashSet::default(),
            prey_body_size_ratio_tolerance: -1.0,
            min_hunger_to_hunt: Self::SERI_UNINITIALIZED,
            min_hp_ratio_to_hunt: 0.0,
        }
    }
}

impl Default for PredatorCfg {
    fn default() -> Self {
        Self {
            territorialism: 0.0,
            pack_size_min: 1,
            pack_size_max: 1,
            do_not_hunt_tags: TagSet::default(),
            prey_body_size_ratio_tolerance: -1.0,
            min_hunger_to_hunt: 40.0,
            min_hp_ratio_to_hunt: 0.0,
        }
    }
}

impl PredatorCfg {
    pub fn from_seri(seri: &PredatorSeri) -> Option<Self> {
        if seri.is_uninitialized() {
            return None;
        }
        let mut pack_size_min = seri.pack_size_min;
        let mut pack_size_max = seri.pack_size_max;
        if pack_size_min == 0 {
            pack_size_min = 1;
        }
        if pack_size_max < pack_size_min {
            pack_size_max = pack_size_min;
        }
        Some(Self {
            territorialism: seri.territorialism.max(0.0),
            pack_size_min,
            pack_size_max,
            do_not_hunt_tags: TagSet::new(&seri.do_not_hunt_tags),
            prey_body_size_ratio_tolerance: seri.prey_body_size_ratio_tolerance,
            min_hunger_to_hunt: seri.min_hunger_to_hunt.max(0.0),
            min_hp_ratio_to_hunt: seri.min_hp_ratio_to_hunt.clamp(0.0, 1.0),
        })
    }
}

#[derive(Component, Debug, Default, Deserialize, Serialize, Copy, Clone)]
pub struct Predator;

fn default_predator_seri_uninitialized() -> f32 { PredatorSeri::SERI_UNINITIALIZED }

#[derive(Component, Debug, Deserialize, Serialize, Copy, Clone, MapEntities)]
pub struct PredatorDetectedByPrey(#[entities] pub Entity);

#[derive(Component, Debug, Deserialize, Serialize, Copy, Clone, Hash, PartialEq, Eq, MapEntities)]
#[relationship(relationship_target = HuntedBy)]
pub struct Hunting {
    #[relationship]
    #[entities]
    pub prey: Entity,
}

#[derive(Component, Debug)]
#[relationship_target(relationship = Hunting)]
pub struct HuntedBy(Vec<Entity>);
