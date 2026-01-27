use bevy::{ecs::entity::MapEntities, prelude::*};
use common::common_components::{StrId};
use serde::{Deserialize, Serialize};

#[derive(Event, Clone, Copy, Default)]
pub struct AttemptHostServer { pub port: Option<u16> } 

#[derive(Event, )]
pub struct StartServerFailed { pub reason: BevyError, } 

#[derive(Event, Clone, Copy, Default)]
pub struct JoinServer { } 

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
