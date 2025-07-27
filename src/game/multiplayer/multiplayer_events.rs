use std::time::SystemTime;

#[allow(unused_imports)] use bevy::prelude::*;

use crate::common::common_components::DisplayName;




#[derive(serde::Deserialize, Event, serde::Serialize, Clone)]
pub struct TransformFromClient { pub entity: Entity, pub transf: Transform, pub time: SystemTime }

#[derive(serde::Deserialize, Event, serde::Serialize, Clone)]
pub struct TransformFromServer { pub entity: Entity, pub trans: Transform, pub time: SystemTime }


impl From<TransformFromClient> for TransformFromServer {
    fn from(client: TransformFromClient) -> Self {
        TransformFromServer {
            entity: client.entity,
            trans: client.transf,
            time: client.time,
        }
    }
}


 
 #[derive(serde::Deserialize, Event, serde::Serialize, Clone)]
pub struct SendPlayerName (pub DisplayName);