use bevy::prelude::*;
use bevy_replicon::prelude::*;

use crate::game::{being::being_components::*, game_components::FacingDirection, multiplayer::{multiplayer_components::MpAuthority, multiplayer_events::TransformFromClient}, player::player_components::{CameraTarget, SelfPlayer}};






#[allow(unused_parens)]
pub fn on_control_change(mut cmd: Commands, query: Query<(Entity, &ControlledBy),(Changed<ControlledBy>)>,
    self_player_ent: Single<Entity, With<SelfPlayer>>,
) {
    let self_player_ent = *self_player_ent;
    for (being_ent, ControlledBy(player_ent)) in query.iter() {
        if *player_ent == self_player_ent {
            cmd.entity(being_ent).insert((ControlledBySelf, CameraTarget));
        } else {
            cmd.entity(being_ent).remove::<ControlledBySelf>();
        }
    }
}


// ----------------------> NO OLVIDARSE DE AGREGARLO AL Plugin DEL MÓDULO <-----------------------------
//                                                       ^^^^
