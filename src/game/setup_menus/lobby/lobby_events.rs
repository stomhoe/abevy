#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;


#[derive(serde::Deserialize, Event, serde::Serialize, Clone)]
pub struct HostStartedGame;




// No olvidarse de agregarlo al Plugin del módulo
// 

// #[derive(serde::Deserialize, Event, serde::Serialize, Clone)]
// pub struct ConnectedEvent;
