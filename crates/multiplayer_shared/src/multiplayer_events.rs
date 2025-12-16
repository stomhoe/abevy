use bevy::{ecs::entity::MapEntities, prelude::*};
use common::common_components::{StrId};
use serde::{Deserialize, Serialize};

#[derive(Event, Deserialize, Serialize, Clone)]
pub struct HostStartedGame;

#[derive(Event, Deserialize, Serialize, Clone)]
pub struct SendUsername(pub StrId);

#[derive(Event, Deserialize, Serialize, Clone, Copy, Default)]
pub struct StartServer { pub port: Option<u16> } 



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
