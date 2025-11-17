use bevy::{ecs::entity::MapEntities, prelude::*};
use common::common_components::{StrId};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Message, Serialize, Clone)]
pub struct HostStartedGame;

#[derive(Deserialize, Message, Serialize, Clone)]
pub struct SendUsername(pub StrId);




#[derive(Deserialize, Message, Serialize, Clone, MapEntities)]
pub struct NameSelected(#[entities] Entity);

#[derive(Deserialize, Message, Serialize, Clone, MapEntities)]
pub struct RaceSelected(#[entities] Entity);

#[derive(Deserialize, Message, Serialize, Clone, MapEntities)]
pub struct HeadSelected(#[entities] Entity);

#[derive(Deserialize, Message, Serialize, Clone, MapEntities)]
pub struct ClassSelected(#[entities] Entity);

#[derive(Deserialize, Message, Serialize, Clone, MapEntities)]
pub struct FollowerSelected(#[entities] Entity);
