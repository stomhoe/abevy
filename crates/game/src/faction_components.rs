#[allow(unused_imports)] use bevy::prelude::*;
use bevy_replicon::prelude::*;
use common::common_components::EntityPrefix;
use superstate::{SuperstateInfo};

use serde::{Deserialize, Serialize};

use crate::player::OfSelf;


#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, Eq, PartialEq, Hash)]
#[require(Replicated, EntityPrefix::new("Faction"))]
pub struct Faction;



#[derive(Component, Debug, Deserialize, Serialize, Clone, Eq, PartialEq, Hash)]
/// entity: Player
pub struct FactionOwner(#[entities] pub Entity);



#[derive(Component, Debug,)]
pub struct BelongsToSelfPlayerFaction;

//esto en cada Being y Player


#[derive(Component, Debug, Deserialize, Serialize, Clone, )]
pub struct BelongsToFaction(#[entities] pub Entity);
impl FromWorld for BelongsToFaction {
    fn from_world(world: &mut World) -> Self {
        let self_faction = world.query_filtered::<Entity, (With<Faction>, With<OfSelf>)>().single(world)
            .expect("BelongsToFaction: No Faction found with OfSelf");
        BelongsToFaction(self_faction)
    }
}


#[derive(Component, Debug, PartialEq, Eq, Hash, )]
pub struct InterFactionEvent(u32);
impl InterFactionEvent {
    pub fn new(nid: u32) -> Self {
        Self(nid)
    }
    pub fn nid(&self) -> u32 {self.0}
}


// #[derive(Bundle, Debug, )]
// pub struct InterFactionEventBundle{
//     pub good_will_event: InterFactionEvent,
//     pub source_dest: SourceDest,
//     pub inclination: Inclination,
// }


#[derive(Component, Debug, Default, )]
pub struct Inclination(i32);



//cada relationship es una entidad, se supone que es hacia una dirección porque uno puede ser defense ally del otro y el otro no (por ej mercenarios)
#[derive(Component, Debug, )]
pub struct RelationShip{
    #[entities]
    pub source: Entity,
    #[entities]
    pub destination: Entity,
}

//adjuntaselo opcionalmente a NonNeutralState o a cualquier cosa

#[derive(Component, Debug, )]
pub enum RelationShipStatus {
    Neutral,
    AtWar,
    Truce,
    Ally(Ally)
}



#[derive(Component, Debug, Default)]
pub enum Ally{
    #[default]
    Defense,
    Attack
}



