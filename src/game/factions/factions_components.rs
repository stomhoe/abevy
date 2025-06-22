#[allow(unused_imports)] use bevy::prelude::*;
use superstate::{SuperstateInfo};


#[derive(Component, Default, Debug,)]
pub struct Faction(u32);

#[derive(Component, Debug,)]
pub struct SelfFaction();

pub struct BelongsToFaction(pub Entity);


#[derive(Component, Debug, )]
//q sea parte de una entity propia]
pub struct GoodWillEvent{
    pub source: Entity,
    pub destination: Entity,
}

#[derive(Component, Debug, )]
pub struct RelationShip{
    pub source: Entity,
    pub destination: Entity,
}

#[derive(Component, Debug, Default, )]
pub struct Inclination(i32);

#[derive(Component, Debug, Default)]
#[require(SuperstateInfo<NonNeutralState>)]
pub struct NonNeutralState{
    remaining_days: Option<u32>,
}

#[derive(Component, Debug, Default)]
#[require(NonNeutralState)]
pub struct AtWar{}

#[derive(Component, Debug, Default)]
#[require(NonNeutralState)]
pub struct Truce{}


#[derive(Component, Debug, Default)]
#[require(SuperstateInfo<Ally>, NonNeutralState)]
pub struct Ally{}

#[derive(Component, Debug, Default)]
#[require(Ally)]
pub struct DefenseAlly();

#[derive(Component, Debug, Default)]
#[require(Ally)]
pub struct AttackAlly();


