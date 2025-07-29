#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;

use crate::game::being::movement::movement_components::InputMoveVector;


#[derive(serde::Deserialize, Event, serde::Serialize, Clone)]
pub struct SendMoveInput { 
    pub vec: InputMoveVector,
    pub being_ent: Option<Entity>,
}

#[derive(serde::Deserialize, Event, serde::Serialize, Clone)]
pub struct TransformFromServer { pub being: Entity, pub trans: Transform, pub interpolate: bool }
impl TransformFromServer {
    pub fn new(being: Entity, trans: Transform, interpolate: bool) -> Self {
        TransformFromServer { being, trans, interpolate }
    }
}