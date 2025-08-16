#[allow(unused_imports)] use bevy::prelude::*;
use bevy_replicon::prelude::*;
use common::common_components::EntityPrefix;
use superstate::{SuperstateInfo};

use serde::{Deserialize, Serialize};



#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, Eq, PartialEq, Hash, Reflect)]
#[require(Replicated, EntityPrefix::new("Faction"))]
pub struct Faction;



#[derive(Component, Debug, Deserialize, Serialize, Copy, Clone, Eq, PartialEq, Hash)]
pub struct FactionOwner { #[entities]pub player: Entity }

#[derive(Component, Debug, Deserialize, Serialize, Copy, Clone, Eq, PartialEq, Hash)]
pub struct FactionLeader { #[entities]pub being: Entity }

#[derive(Component, Debug,)]
pub struct IsAffiliatedToMyFaction;

#[derive(Component, Debug, Deserialize, Serialize, Clone, Eq, PartialEq, Hash)]
pub struct BelongsToAPlayerFaction;


/*
           .replicate::<FactionMembers>()
           .register_type::<BelongsToFaction>()
           .register_type::<FactionMembers>()
*/


//esto en cada Being y Player


/*
           .replicate::<BelongsToFaction>()
           .register_type::<BelongsToFaction>()
           .register_type::<FactionThings>()
*/
#[derive(Component, Debug, Deserialize, Serialize, Copy, Clone, Hash, PartialEq, Eq, Reflect)]
#[relationship(relationship_target = FactionThings)]
pub struct BelongsToFaction(
    #[relationship] #[entities]
    pub Entity,
);

#[derive(Component, Debug, Reflect)]
#[relationship_target(relationship = BelongsToFaction)]
pub struct FactionThings(Vec<Entity>);
impl FactionThings { pub fn entities(&self) -> &[Entity] { &self.0 } }

/*
           .replicate::<PlayerBelongsTofaction>()
           .register_type::<PlayerBelongsTofaction>()
           .register_type::<PlayersInFaction>()
*/
#[derive(Component, Debug, Deserialize, Serialize, Copy, Clone, Hash, PartialEq, Eq, Reflect)]
#[relationship(relationship_target = PlayerMembers)]
pub struct PlayerOfFaction {
    #[relationship] #[entities]
    faction: Entity,
}
impl PlayerOfFaction {pub fn new(faction: Entity) -> Self {PlayerOfFaction { faction }}}

#[derive(Component, Debug, Reflect)]
#[relationship_target(relationship = PlayerOfFaction)]
pub struct PlayerMembers(Vec<Entity>);
impl PlayerMembers { pub fn entities(&self) -> &[Entity] { &self.0 } }



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



