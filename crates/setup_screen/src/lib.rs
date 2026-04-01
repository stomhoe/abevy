use bevy::prelude::*;

pub mod character_creation;
pub mod lobby;
pub mod setup_screen;

#[allow(unused_parens, )]
pub fn plugin(app: &mut App) {
    app.add_plugins((
        setup_screen::plugin,
        character_creation::plugin,
    ));
}
