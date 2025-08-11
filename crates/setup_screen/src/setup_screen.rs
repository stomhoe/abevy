use bevy::prelude::*;

use crate::lobby;

pub fn plugin(app: &mut App) {
    app
       .add_plugins(lobby::plugin)
    ;
}