use bevy::prelude::*;

use crate::game::setup_menus::lobby::LobbyPlugin;

mod lobby;
pub struct SetupMenusPlugin;
#[allow(unused_parens)]
impl Plugin for SetupMenusPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(LobbyPlugin)
        ;
    }
}