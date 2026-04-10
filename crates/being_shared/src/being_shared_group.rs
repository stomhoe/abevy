
use bevy::{ecs::entity::{EntityHashMap, EntityHashSet, MapEntities}, prelude::*};

use serde::{Deserialize, Serialize};


#[derive(Component, Debug, Deserialize, Serialize, Copy, Clone, Hash, PartialEq, Eq, )]
pub struct LedBy {
    pub leader: Entity,
}

#[derive(Component, Debug, Deserialize, Serialize, Clone, Reflect, MapEntities)]
pub struct JoinedGroups(#[entities] pub EntityHashSet);
impl JoinedGroups {
    pub fn single(group: Entity) -> Self {
        let mut groups = EntityHashSet::default();
        groups.insert(group);
        Self(groups)
    }

    pub fn insert(&mut self, group: Entity) -> bool {
        self.0.insert(group)
    }

    pub fn remove(&mut self, group: Entity) -> bool {
        self.0.remove(&group)
    }

    pub fn contains(&self, group: Entity) -> bool {
        self.0.contains(&group)
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = Entity> + '_ {
        self.0.iter().copied()
    }
}

#[derive(Component, Debug, Clone, )]
pub struct BeingMembers(pub EntityHashSet);
impl BeingMembers {
    pub fn iter(&self) -> impl Iterator<Item = Entity> + '_ {
        self.0.iter().copied()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}


#[derive(Component, Debug, Default, Deserialize, Serialize, Clone,)]
//#[component(map_entities)]
pub struct MemberRanks(pub EntityHashMap<f32>);


#[derive(Component, Debug, Deserialize, Serialize, Copy, Clone, Hash, PartialEq, Eq, bevy::ecs::entity::MapEntities, )]
#[relationship(relationship_target = SquadMembers)]
pub struct SquadMemberOf(#[relationship]#[entities]pub Entity);

// current physically close distance group of beings we belong to and are currently operating with
#[derive(Component, Debug, )]
#[relationship_target(relationship = SquadMemberOf)]
pub struct SquadMembers(Vec<Entity>);

/// Cached total weight of a predator squad for pack-hunting decisions.
#[derive(Component, Debug, Default, Copy, Clone)]
pub struct SquadWeightSum(pub f32);

#[derive(Component, Debug, Default, Deserialize, Serialize, Copy, Clone)]
pub struct NoSpawnSquadEntity;

#[derive(Component, Debug, Copy, Clone)]
pub struct PreventCleanup;
