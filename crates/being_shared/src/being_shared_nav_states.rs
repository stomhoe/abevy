use bevy::{ecs::entity::MapEntities, prelude::*};
use serde::{Deserialize, Serialize};
use superstate::SuperstateInfo;
use tilemap_shared::{DimensionRef, GlobalTilePos};

#[derive(Component, Debug, Default, Clone)]
#[require(SuperstateInfo<BehavorialNavState>, )]
pub struct BehavorialNavState;

#[derive(Component, Debug, Deserialize, Serialize, Copy, Clone, MapEntities)]
#[relationship(relationship_target = NavChasers)]
#[require(BehavorialNavState, )]
pub struct NavChasing {
    #[relationship] #[entities]
    pub target: Entity,
    pub stop_distance: f32,
}
impl NavChasing {
    pub fn new(target: Entity, stop_distance: f32) -> Self {
        Self {
            target,
            stop_distance: stop_distance.max(0.0),
        }
    }

    pub fn chase_target_pos(
        &self,
        chaser_ent: Entity,
        chaser_dim: DimensionRef,
        targets_query: &Query<(Entity, &GlobalTilePos, &DimensionRef), >,
    ) -> Option<GlobalTilePos> {
        if self.target == chaser_ent {
            return None;
        }
        let Ok((_target_ent, target_gpos, &target_dim, )) = targets_query.get(self.target) else {
            return None;
        };
        if target_dim != chaser_dim {
            return None;
        }
        Some(*target_gpos)
    }
}

#[derive(Component, Debug, )]
#[relationship_target(relationship = NavChasing)]
pub struct NavChasers(Vec<Entity>);

#[derive(Component, Debug, Deserialize, Serialize, Clone, MapEntities)]
#[require(BehavorialNavState, )]
pub struct Fleeing {
    #[entities]
    pub threats: Vec<Entity>,
    pub desired_distance_tiles: f32,
}
impl Default for Fleeing {
    fn default() -> Self {
        Self {
            threats: Vec::default(),
            desired_distance_tiles: 20.0,
        }
    }
}
impl Fleeing {
    pub fn new(flee_from: Entity) -> Self {
        Self::with_distance(flee_from, 20.0)
    }

    pub fn with_distance(flee_from: Entity, desired_distance_tiles: f32) -> Self {
        Self {
            threats: vec![flee_from],
            desired_distance_tiles: desired_distance_tiles.max(0.0),
        }
    }

    pub fn add_threat(&mut self, threat: Entity) {
        if self.threats.iter().any(|ent| *ent == threat) {
            return;
        }
        self.threats.push(threat);
    }

    pub fn primary_threat(&self) -> Option<Entity> {
        self.threats.first().copied()
    }

    pub fn is_empty(&self) -> bool {
        self.threats.is_empty()
    }
}

#[derive(Debug, Deserialize, Serialize, Copy, Clone, PartialEq, Eq, Hash)]
pub enum NavOrderSource {
    Wandering,
    Chasing,
    Fleeing,
}
impl NavOrderSource {
    pub fn tie_break_rank(self) -> u8 {
        match self {
            Self::Fleeing => 3,
            Self::Chasing => 2,
            Self::Wandering => 1,
        }
    }
}

#[derive(Component, Debug, Copy, Clone, Default)]
pub struct GoTo {
    pub pos: GlobalTilePos,
    pub stop_distance: f32,
    pub source: Option<NavOrderSource>,
    pub updated_tick: u32,
}
impl GoTo {
    pub fn new(pos: GlobalTilePos, stop_distance: f32) -> Self {
        Self {
            pos,
            stop_distance: stop_distance.max(0.0),
            source: None,
            updated_tick: 0,
        }
    }

    pub fn with_source(pos: GlobalTilePos, stop_distance: f32, source: NavOrderSource, updated_tick: u32) -> Self {
        Self {
            pos,
            stop_distance: stop_distance.max(0.0),
            source: Some(source),
            updated_tick,
        }
    }
}
