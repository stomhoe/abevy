#[allow(unused_imports)]
use bevy::prelude::*;
use bevy_replicon::prelude::*;
use common::common_components::{AssetScoped, Prefix, SparedFromHotReloading};
use serde::{Deserialize, Serialize};

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, Eq, PartialEq, Hash)]
#[require(Replicated, Prefix::trunc("Faction"), AssetScoped, SparedFromHotReloading,)]
pub struct Faction;



#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, Eq, PartialEq, Hash)]
#[require(Replicated, Prefix::trunc("FactionInstTemplate"), AssetScoped, SparedFromHotReloading,)]
pub struct FactionInstTempl;

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, Eq, PartialEq, Hash)]
#[require(Replicated, Prefix::trunc("Culture"), AssetScoped, SparedFromHotReloading,)]
pub struct Culture;

#[derive(Component, Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct GroupPlayerAuthority {
    #[entities]
    pub player: Entity,
}

#[derive(Component, Debug, Clone)]
pub struct IsAffiliatedToMyFaction;

#[derive(Component, Debug, Clone, Eq, PartialEq, Hash)]
pub struct BelongsToAPlayerFaction;

#[derive(Component, Debug, Clone, Default)]
pub struct PlayerMembers(pub Vec<Entity>);
impl PlayerMembers {
    pub fn insert(&mut self, player: Entity) -> bool {
        if self.contains(player) {
            return false;
        }
        self.0.push(player);
        true
    }

    pub fn remove(&mut self, player: Entity) -> bool {
        let Some(idx) = self.0.iter().position(|&ent| ent == player) else {
            return false;
        };
        self.0.swap_remove(idx);
        true
    }

    pub fn contains(&self, player: Entity) -> bool {
        self.0.contains(&player)
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = Entity> + '_ {
        self.0.iter().copied()
    }
}

#[derive(Component, Debug, PartialEq, Eq, Hash, Clone)]
pub struct InterFactionEvent(u32);
impl InterFactionEvent {
    pub fn new(nid: u32) -> Self { Self(nid) }
    pub fn nid(&self) -> u32 { self.0 }
}

#[derive(Component, Debug, Default, Clone)]
#[allow(dead_code, )]
pub struct Inclination(i32);

#[derive(Component, Debug, Clone)]
pub struct RelationShip {
    #[entities]
    pub source: Entity,
    #[entities]
    pub destination: Entity,
}

#[derive(Component, Debug, Clone)]
pub enum RelationShipStatus {
    Neutral,
    AtWar,
    Truce,
    Ally(Ally),
}

#[derive(Component, Debug, Default, Clone)]
pub enum Ally {
    #[default]
    Defense,
    Attack,
}
