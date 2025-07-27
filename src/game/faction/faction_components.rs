#[allow(unused_imports)] use bevy::prelude::*;
use bevy_replicon::prelude::*;
use superstate::{SuperstateInfo};

use crate::{common::common_components::EntityPrefix, game::{faction::faction_resources::FactionEntityMap, game_components::SourceDest}};
use serde::{Deserialize, Serialize};



#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, Eq, PartialEq, Hash)]
#[require(Replicated, EntityPrefix::new("Faction "))]
pub struct Faction(String);
impl Faction {
    pub fn new<S: Into<String>, B: Bundle>(cmd: &mut Commands, map: &mut FactionEntityMap, id: S, name: S, bundle: B) -> Entity {
        FactionEntityMap::insert_faction(map, cmd, Faction(id.into()), name, bundle)
    }
    pub fn id(&self) -> &String {&self.0}
}


#[derive(Component, Debug, Deserialize, Serialize, Clone, Eq, PartialEq, Hash)]
/// entity: Player
pub struct FactionOwner(#[entities] pub Entity);



#[derive(Component, Debug,)]
pub struct BelongsToSelfPlayerFaction;

//esto en cada Being y Player


#[derive(Component, Debug, Deserialize, Serialize, Clone, )]
pub struct BelongsToFaction(#[entities] pub Entity);


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
    #[entities]
    pub source: Entity,
    #[entities]
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



