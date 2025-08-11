use bevy::app::App;

use crate::player::KeyboardInputMappings;




 pub fn plugin(app: &mut App) {
    app
    .init_resource::<KeyboardInputMappings>()

    ;
}