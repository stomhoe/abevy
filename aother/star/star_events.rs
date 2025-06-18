use bevy::prelude::*;

#[derive(Event)]
pub struct GameOver{
    pub score: i32
}