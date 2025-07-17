#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;


#[derive(serde::Deserialize, Event, serde::Serialize, Clone)]
pub struct NameSelected (Entity);

#[derive(serde::Deserialize, Event, serde::Serialize, Clone)]
pub struct RaceSelected (Entity);


#[derive(serde::Deserialize, Event, serde::Serialize, Clone)]
pub struct HeadSelected (Entity);

#[derive(serde::Deserialize, Event, serde::Serialize, Clone)]
pub struct ClassSelected (Entity);

#[derive(serde::Deserialize, Event, serde::Serialize, Clone)]
pub struct FollowerSelected (Entity);



// No olvidarse de agregarlo al Plugin del módulo
// .add_client_trigger::<RaceSelected>(Channel::Ordered)