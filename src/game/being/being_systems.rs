use bevy::prelude::*;
use bevy_replicon::prelude::*;

use crate::game::{being::being_components::*, game_components::{FacingDirection, LocalCpu}, multiplayer::{multiplayer_components::MpAuthority, }, player::player_components::{CameraTarget, OfSelf, Player}};






#[allow(unused_parens)]
pub fn on_control_change(mut cmd: Commands, query: Query<(Entity, &ControlledBy),(Changed<ControlledBy>)>,
    self_player_ent: Single<Entity, (With<OfSelf>, With<Player>)>,
) {
    let self_player_ent = *self_player_ent;
    for (being_ent, &ControlledBy { player: controlling_ent }) in query.iter() {
        if controlling_ent == self_player_ent {
            cmd.entity(being_ent).insert_if_new((ControlledLocally, CameraTarget));
        } else {
            cmd.entity(being_ent).remove::<ControlledLocally>();
        }
    }
}


