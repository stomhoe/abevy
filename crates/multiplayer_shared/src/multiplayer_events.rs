use bevy::{ecs::entity::MapEntities, prelude::*};
use common::components::DisplayName;
use game::movement_components::InputMoveVector;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Event, Serialize, Clone)]
pub struct HostStartedGame;

#[derive(Deserialize, Event, Serialize, Clone)]
pub struct SendPlayerName(pub DisplayName);

#[derive(Deserialize, Event, Serialize, Clone, Component, MapEntities)]
pub struct MoveStateUpdated {
    #[entities]
    pub being_ent: Entity,
    pub moving: bool,
}

#[derive(Deserialize, Event, Serialize, Clone, MapEntities)]
pub struct SendMoveInput {
    #[entities]
    pub being_ent: Entity,
    pub vec: InputMoveVector,
}

#[derive(Deserialize, Event, Serialize, Clone, MapEntities)]
pub struct TransformFromServer {
    #[entities]
    pub being: Entity,
    pub trans: Transform,
    pub interpolate: bool,
}
impl TransformFromServer {
    pub fn new(being: Entity, trans: Transform, interpolate: bool) -> Self {
        TransformFromServer { being, trans, interpolate }
    }
}

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
