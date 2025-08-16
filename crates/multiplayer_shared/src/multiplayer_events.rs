use bevy::{ecs::entity::MapEntities, prelude::*};
use common::common_components::{StrId};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Event, Serialize, Clone)]
pub struct HostStartedGame;

#[derive(Deserialize, Event, Serialize, Clone)]
pub struct SendUsername(pub StrId);




#[derive(Deserialize, Event, Serialize, Clone, MapEntities)]
pub struct NameSelected(#[entities] Entity);

#[derive(Deserialize, Event, Serialize, Clone, MapEntities)]
pub struct RaceSelected(#[entities] Entity);

#[derive(Deserialize, Event, Serialize, Clone, MapEntities)]
pub struct HeadSelected(#[entities] Entity);

#[derive(Deserialize, Event, Serialize, Clone, MapEntities)]
pub struct ClassSelected(#[entities] Entity);

#[derive(Deserialize, Event, Serialize, Clone, MapEntities)]
pub struct FollowerSelected(#[entities] Entity);
