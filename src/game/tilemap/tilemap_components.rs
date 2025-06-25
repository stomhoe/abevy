use std::default;

use bevy::platform::collections::HashSet;
#[allow(unused_imports)] use bevy::prelude::*;


#[derive(Component, Debug, Default, )]
pub struct ActivatesChunks(pub HashSet<Entity>,);




use superstate::{SuperstateInfo};

#[derive(Component, Default)]
#[require(SuperstateInfo<ChunkState>)]
pub struct ChunkState;

#[derive(Component)]
#[require(ChunkState)]
pub struct InitializedChunk;

#[derive(Component)]
#[require(ChunkState)]
#[require(Visibility::Hidden)]
pub struct UninitializedChunk;


#[derive(Component, Debug, Default, )]
pub struct ChunkPos(pub IVec2);