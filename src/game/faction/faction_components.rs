#[allow(unused_imports)] use bevy::prelude::*;
use superstate::{SuperstateInfo};

use crate::game::game_components::SourceDest;


#[derive(Component, Debug, PartialEq, Eq, Hash, )]
pub struct Faction(u32);
impl Faction {
    pub fn new(nid: u32) -> Self {
        Self(nid)
    }
    pub fn nid(&self) -> u32 {self.0}
}

#[derive(Component, Debug,)]
//esto en la Faction nuestra y en nuestros beings
pub struct SelfFaction();

//esto en cada Being
#[allow(dead_code)]
pub struct BelongsToFaction(pub Entity);


#[derive(Component, Debug, PartialEq, Eq, Hash, )]
pub struct InterFactionEvent(u32);
impl InterFactionEvent {
    pub fn new(nid: u32) -> Self {
        Self(nid)
    }
    pub fn nid(&self) -> u32 {self.0}
}


#[derive(Bundle, Debug, )]
pub struct InterFactionEventBundle{
    pub good_will_event: InterFactionEvent,
    pub source_dest: SourceDest,
    pub inclination: Inclination,
}


#[derive(Component, Debug, Default, )]
pub struct Inclination(i32);



//cada relationship es una entidad, se supone que es hacia una dirección porque uno puede ser defense ally del otro y el otro no (por ej mercenarios)
#[derive(Component, Debug, )]
pub struct RelationShip{
    pub source: Entity,
    pub destination: Entity,
}

//adjuntaselo opcionalmente a NonNeutralState o a cualquier cosa

#[derive(Component, Debug, Default)]
#[require(SuperstateInfo<NonNeutralState>)]
pub struct NonNeutralState;



#[derive(Component, Debug, Default)]
#[require(NonNeutralState)]
pub struct AtWar{}

#[derive(Component, Debug, Default)]
#[require(NonNeutralState)]
pub struct Truce{}


#[derive(Component, Debug, Default)]
#[require(NonNeutralState)]
pub enum Ally{
    #[default]
    Defense,
    Attack
}



