use bevy::prelude::*;

#[derive(Component, Debug,)]
pub struct Name(String);

#[derive(Component, Debug,)]
pub struct Description(String);

#[derive(Component, Debug,)]
pub struct Sid(String);



#[derive(Component, Debug,)]
pub struct GameZindex(pub f32);