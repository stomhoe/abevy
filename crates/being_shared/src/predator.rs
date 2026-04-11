use bevy::{ecs::entity::MapEntities, platform::collections::HashSet, prelude::*};
use common::common_tag_components::TagSet;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(default)]
pub struct PredatorSeri {
    pub do_not_hunt_tags: HashSet<String>,
    pub do_not_hunt_same_kind: bool,
    pub prey_body_size_ratio_tolerance: f32,
    pub min_hunger_to_hunt: f32,
    pub min_hp_ratio_to_hunt: f32,
}
impl PredatorSeri {
    pub const SERI_UNINITIALIZED: f32 = f32::NEG_INFINITY;
    pub const DEFAULT_PREY_BODY_SIZE_RATIO_TOLERANCE: f32 = 1.1;

    pub fn is_uninitialized(&self) -> bool {
        self.min_hunger_to_hunt == Self::SERI_UNINITIALIZED
    }
}

impl Default for PredatorSeri {
    fn default() -> Self {
        Self {
            do_not_hunt_tags: HashSet::default(),
            do_not_hunt_same_kind: true,
            prey_body_size_ratio_tolerance: Self::DEFAULT_PREY_BODY_SIZE_RATIO_TOLERANCE,
            min_hunger_to_hunt: Self::SERI_UNINITIALIZED,
            min_hp_ratio_to_hunt: 0.0,
        }
    }
}

#[derive(Component, Debug, Deserialize, Serialize, Clone)]
pub struct PredatorCfg {
    pub do_not_hunt_tags: TagSet,
    pub do_not_hunt_same_kind: bool,
    pub prey_body_size_ratio_tolerance: f32,
    pub min_hunger_to_hunt: f32,
    pub min_hp_ratio_to_hunt: f32,
}
impl PredatorCfg {
    pub const DEFAULT_PREY_BODY_SIZE_RATIO_TOLERANCE: f32 = PredatorSeri::DEFAULT_PREY_BODY_SIZE_RATIO_TOLERANCE;

    pub fn from_seri(seri: &PredatorSeri) -> Option<Self> {
        if seri.is_uninitialized() {
            return None;
        }
        Some(Self {
            do_not_hunt_tags: TagSet::new(&seri.do_not_hunt_tags),
            do_not_hunt_same_kind: seri.do_not_hunt_same_kind,
            prey_body_size_ratio_tolerance: seri.prey_body_size_ratio_tolerance,
            min_hunger_to_hunt: seri.min_hunger_to_hunt.max(0.0),
            min_hp_ratio_to_hunt: seri.min_hp_ratio_to_hunt.clamp(0.0, 1.0),
        })
    }
}

impl Default for PredatorCfg {
    fn default() -> Self {
        Self {
            do_not_hunt_tags: TagSet::default(),
            do_not_hunt_same_kind: true,
            prey_body_size_ratio_tolerance: Self::DEFAULT_PREY_BODY_SIZE_RATIO_TOLERANCE,
            min_hunger_to_hunt: 0.4,
            min_hp_ratio_to_hunt: 0.0,
        }
    }
}

#[derive(Component, Debug, Default, Deserialize, Serialize, Copy, Clone)]
pub struct Predator;

#[derive(Component, Debug, Deserialize, Serialize, Copy, Clone, PartialEq, MapEntities)]
#[relationship(relationship_target = HuntedBy)]
pub struct Hunting {
    #[relationship]
    #[entities]
    pub prey: Entity,
    #[serde(default)]
    pub retaliating: bool,
    #[serde(default)]
    pub retaliation_stop_distance_tiles: f32,
}

impl Hunting {
    pub fn new(prey: Entity) -> Self {
        Self {
            prey,
            retaliating: false,
            retaliation_stop_distance_tiles: 0.0,
        }
    }

    pub fn with_retaliation(prey: Entity, retaliation_stop_distance_tiles: f32) -> Self {
        Self {
            prey,
            retaliating: true,
            retaliation_stop_distance_tiles: retaliation_stop_distance_tiles.max(0.0),
        }
    }
}

#[derive(Component, Debug)]
#[relationship_target(relationship = Hunting)]
pub struct HuntedBy(Vec<Entity>);
