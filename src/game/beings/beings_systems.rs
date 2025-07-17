use bevy::{ecs::system::command, prelude::*};
use bevy_replicon::prelude::*;

use crate::game::{beings::beings_components::{Being, ControlledBy, ControlledBySelf, InputMoveDirection}, game_components::Nid, game_resources::NidEntityMap, multiplayer::{multiplayer_components::MpAuthority, multiplayer_events::TransformFromClient}, player::player_components::SelfPlayer};



pub fn handle_movement(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &InputMoveDirection, &mut Transform), With<MpAuthority>>,
) {
    for (ent, input_move_direction, mut transform) in query.iter_mut() {
        let speed = 1000.0;
        let delta = time.delta_secs();
        let movement = input_move_direction.0 * speed * delta;
        transform.translation += movement;

        commands.client_trigger(
            TransformFromClient {
                entity: ent,
                transf: transform.clone(),
                time: std::time::SystemTime::now(),
            }
        );
    }
}



