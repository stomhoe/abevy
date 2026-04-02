use bevy::{ecs::entity::MapEntities, prelude::*};
use serde::{Deserialize, Serialize};
use superstate::SuperstateInfo;
use tilemap_shared::{DimensionRef, GlobalTilePos};

#[derive(Component, Debug, Default, Clone)]
#[require(SuperstateInfo<BehavorialNavState>, )]
pub struct BehavorialNavState;

#[derive(Component, Debug, Deserialize, Serialize, Copy, Clone, MapEntities)]
#[relationship(relationship_target = Chasers)]
#[require(BehavorialNavState, )]
pub struct Chasing {
    #[relationship] #[entities]
    pub target: Entity,
    pub stop_distance: f32,
}
impl Chasing {
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
#[relationship_target(relationship = Chasing)]
pub struct Chasers(Vec<Entity>);

#[derive(Component, Debug, Deserialize, Serialize, Copy, Clone, MapEntities)]
#[require(BehavorialNavState, )]
pub struct Fleeing(#[entities] pub Entity);
impl Fleeing {
    pub fn new(flee_from: Entity) -> Self {
        Self(flee_from)
    }

    pub fn flee_from(&self) -> Entity {
        self.0
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
