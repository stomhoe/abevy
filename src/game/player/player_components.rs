#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{common::common_components::EntityPrefix, game::being::being_components::ControlledBy, AppState};

#[derive(Component, Debug,)]
pub struct OfSelf;


//NO ES PARA ADJUNTARSELO A ENTITIES COMÚNES (OBJETOS O BEINGS)
// ES PARA ADJUNTARSELO A ENTITIES QUE REPRESENTAN JUGADORES
#[derive(Debug, Component, Default, Serialize, Deserialize)]
#[require(Replicated, EntityPrefix::new("Player "), StateScoped::<AppState>(AppState::StatefulGameSession))]
pub struct Player;

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, )]
pub struct TrustedAlgo;

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, )]
pub struct TrustedMovement;

#[derive(Debug, Component, Default, Serialize, Deserialize)]
#[require(Replicated, Player)]
pub struct HostPlayer;


// impl Default for Player {
//     fn default() -> Self {
//         Self { 
//             //id: ClientId::from(rand::random::<u64>()),
//             display_name:,
//         }
//     }
// }

#[derive(Component, Default)] 
#[require(Transform)]
pub struct CameraTarget;


#[derive(Component, Debug, Deserialize, Serialize, Clone, )]
pub struct CreatedCharacter(#[entities] pub Entity);


#[derive(Component)]
#[relationship_target(relationship = ControlledBy)]
pub struct Controls(Vec<Entity>);
impl Controls {
    pub fn controls(&self) -> &[Entity] {
        &self.0
    }
}