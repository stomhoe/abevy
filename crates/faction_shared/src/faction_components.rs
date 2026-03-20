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
pub struct FactionOwner {
    #[entities]
    pub player: Entity,
}

#[derive(Component, Debug, Clone)]
pub struct IsAffiliatedToMyFaction;

#[derive(Component, Debug, Clone, Eq, PartialEq, Hash)]
pub struct BelongsToAPlayerFaction;

#[derive(Component, Debug, Deserialize, Serialize, Copy, Clone, Hash, PartialEq, Eq)]
#[relationship(relationship_target = FactionThings)]
pub struct BelongsToFaction(
    #[relationship]
    #[entities]
    pub Entity,
);

#[derive(Component, Debug, Clone)]
#[relationship_target(relationship = BelongsToFaction)]
pub struct FactionThings(Vec<Entity>);


#[derive(Component, Debug, Copy, Clone, Hash, PartialEq, Eq)]
#[relationship(relationship_target = PlayerMembers)]
pub struct PlayerOfFaction {
    #[relationship]
    #[entities]
    faction: Entity,
}
impl PlayerOfFaction {
    pub fn new(faction: Entity) -> Self { PlayerOfFaction { faction } }
}

#[derive(Component, Debug, Clone)]
#[relationship_target(relationship = PlayerOfFaction)]
pub struct PlayerMembers(Vec<Entity>);

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
