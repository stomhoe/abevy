use std::default;

use bevy::platform::collections::HashSet;
#[allow(unused_imports)] use bevy::prelude::*;


#[derive(Component, Debug, Default, )]
pub struct ActivatesChunks(pub HashSet<Entity>,);


#[derive(Component, Debug,)]
#[require(Visibility::Hidden)]
pub struct Chunk{
}
impl default::Default for Chunk {
    fn default() -> Self {
        Chunk {  }
    }
}

