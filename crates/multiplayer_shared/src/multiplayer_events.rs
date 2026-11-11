use bevy::{ecs::entity::MapEntities, prelude::*};
use common::common_components::{HashId, StrId};
use serde::{Deserialize, Serialize};

#[derive(Event, Clone, Copy, Default)]
pub struct AttemptHostServer { pub port: Option<u16> }

#[derive(Event, )]
pub struct StartServerFailed { pub reason: BevyError, }

#[derive(Event, Clone, Copy, Default)]
pub struct JoinServer{ }

#[derive(Event, Deserialize, Serialize, Clone)]
pub struct HostStartedGameplay;

#[derive(Event, Deserialize, Serialize, Clone)]
pub struct SendUsername(pub StrId);


#[derive(Message, Deserialize, Serialize, Clone, MapEntities)]
pub struct NameSelected(#[entities] Entity);

#[derive(Message, Deserialize, Serialize, Clone, MapEntities)]
pub struct RaceSelected(#[entities] Entity);

#[derive(Message, Deserialize, Serialize, Clone, MapEntities)]
pub struct HeadSelected(#[entities] Entity);

#[derive(Message, Deserialize, Serialize, Clone, MapEntities)]
pub struct ClassSelected(#[entities] Entity);

#[derive(Message, Deserialize, Serialize, Clone, MapEntities)]
pub struct FollowerSelected(#[entities] Entity);

#[derive(Message, Deserialize, Serialize, Clone, Debug)]
pub struct CharacterCreationRequest {
    pub race: HashId,
    pub class: HashId,
    pub sprites: Vec<(HashId, HashId)>,
    pub pets: Vec<HashId>,
}

#[derive(Message, Deserialize, Serialize, Clone, Copy, Debug)]
pub struct CharacterCreationRejected(pub CharacterCreationRejection);

#[derive(Message, Deserialize, Serialize, Clone, Debug)]
pub struct CharacterCreationAccepted(pub CharacterCreationRequest);

#[derive(Deserialize, Serialize, Clone, Copy, Debug)]
pub enum CharacterCreationRejection {
    UnknownRace,
    RaceNotSelectable,
    UnknownClass,
    ClassNotSelectable,
    IncompatibleRace,
    InvalidSpriteSelection,
    InvalidPetSelection,
}

