#[allow(unused_imports)] use bevy::prelude::*;
use common::common_components::StrId;



#[derive(Resource)]
pub struct KeyboardInputMappings {
    pub move_up: KeyCode,
    pub move_down: KeyCode,
    pub move_left: KeyCode,
    pub move_right: KeyCode,
    pub duck: KeyCode,
    pub jump_or_fly: KeyCode,
    pub attack: KeyCode,
    pub interact: KeyCode,
    pub inventory: KeyCode,
    pub pause: KeyCode,
}
impl Default for KeyboardInputMappings {
    fn default() -> Self {
        KeyboardInputMappings {
            move_up: KeyCode::KeyW,
            move_down: KeyCode::KeyS,
            move_left: KeyCode::KeyA,
            move_right: KeyCode::KeyD,
            duck: KeyCode::KeyC,
            jump_or_fly: KeyCode::Space,
            attack: KeyCode::ControlLeft,
            interact: KeyCode::KeyE,
            inventory: KeyCode::KeyI,
            pause: KeyCode::Escape,
        }
    }
}

#[derive(Resource)]//HAY Q TENERLOS SEGREGADOS POR SI SE QUIERE VOLVER AL TECLADO SIN HABERLO SOBRESCRITO TODO
pub struct GamepadInputMappings {
    // pub move_up: GamepadButton,
    // pub move_down: GamepadButton,
    // pub move_left: GamepadButton,
    // pub move_right: GamepadButton,
    pub jump: GamepadButton,
    pub attack: GamepadButton,
    pub interact: GamepadButton,
    pub inventory: GamepadButton,
    pub pause: GamepadButton,
    
}

#[derive(Resource, Debug, )]
pub struct PlayerData { pub username: StrId, }
impl Default for PlayerData {
    fn default() -> Self {
        let username = StrId::new(format!("Player-{}", nano_id::base64::<6>())).expect("Failed to create StrId for playerdata");
        Self { username }
    }
}