#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
use serde::{Deserialize, Serialize};

use crate::common::common_components::EntityPrefix;

#[derive(Component, Debug,)]
pub struct OfSelf;


//NO ES PARA ADJUNTARSELO A ENTITIES COMÚNES (OBJETOS O BEINGS)
// ES PARA ADJUNTARSELO A ENTITIES QUE REPRESENTAN JUGADORES
#[derive(Debug, Component, Default, Serialize, Deserialize)]
#[require(Replicated, EntityPrefix::new("Player "))]
pub struct Player;

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
pub struct CreatedCharacter(pub Entity);
