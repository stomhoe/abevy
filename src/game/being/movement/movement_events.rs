use bevy::ecs::entity::MapEntities;
#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;

use crate::game::being::movement::movement_components::InputMoveVector;


#[derive(serde::Deserialize, Event, serde::Serialize, Clone, MapEntities)]
pub struct SendMoveInput { 
    #[entities] pub being_ent: Entity,
    pub vec: InputMoveVector,
}

#[derive(serde::Deserialize, Event, serde::Serialize, Clone, MapEntities)]
pub struct TransformFromServer { #[entities] pub being: Entity, pub trans: Transform, pub interpolate: bool }
impl TransformFromServer {
    pub fn new(being: Entity, trans: Transform, interpolate: bool) -> Self {
        TransformFromServer { being, trans, interpolate }
    }
}



// #[derive(serde::Deserialize, Event, serde::Serialize, Clone)]
// pub struct AnimStateFromServer { pub being: Entity, pub trans: Transform, pub interpolate: bool }