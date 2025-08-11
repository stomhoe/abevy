#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
use common::{common_components::EntityPrefix, common_states::AppState};
use serde::{Deserialize, Serialize};

use crate::being_components::{HumanControlled, PlayerDirectControllable};


#[derive(Component, Debug,)]
pub struct OfSelf;


//NO ES PARA ADJUNTARSELO A ENTITIES COMÚNES (OBJETOS O BEINGS)
// ES PARA ADJUNTARSELO A ENTITIES QUE REPRESENTAN JUGADORES
#[derive(Debug, Component, Default, Serialize, Deserialize)]
#[require(Replicated, EntityPrefix::new("Player"), StateScoped::<AppState>(AppState::StatefulGameSession))]
pub struct Player;

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, )]
pub struct TrustedForUnaCosa;

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, )]
pub struct TrustedForOtracosa;

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, )]
pub struct TrustedMovement;

#[derive(Debug, Component, Default, Serialize, Deserialize)]
#[require(Player)]
pub struct HostPlayer;


          
#[derive(Component, Debug, Deserialize, Serialize, Copy, Clone, Hash, PartialEq, Eq, Reflect)]
#[relationship(relationship_target = CreatedCharacters)]
#[require(PlayerDirectControllable, HumanControlled(true))]
pub struct CharacterCreatedBy {
    #[relationship] #[entities] pub player: Entity,
}

#[derive(Component, Debug, Reflect)]
#[relationship_target(relationship = CharacterCreatedBy)]
pub struct CreatedCharacters(Vec<Entity>);
impl CreatedCharacters { pub fn entities(&self) -> &[Entity] { &self.0 } }



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

// ---------------------------> NO OLVIDARSE DE INICIALIZARLO EN EL Plugin DEL MÓDULO <-----------------------
