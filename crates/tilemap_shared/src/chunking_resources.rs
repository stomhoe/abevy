use bevy::platform::collections::HashMap;
use bevy::prelude::*;
use smallvec::SmallVec;

use crate::*;

#[derive(Resource, Default)]
pub struct LoadedChunks (pub HashMap<(DimensionRef, ChunkPos), Entity>,);

#[derive(Resource, Default)]
pub struct LoadedMacroChunks(pub HashMap<(DimensionRef, MacroChunkPos), Entity>,);

pub type SmallEntiArr = SmallVec<[Entity; 16]>;
