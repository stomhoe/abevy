#[allow(unused_imports)] use bevy::platform::collections::HashMap;
use bevy::{prelude::*};

use crate::{ChunkPos, DimensionRef};

#[derive(Message, Debug, Clone, )]
pub struct ChunkLoaded {
    pub dimension: DimensionRef,
    pub chunk_pos: ChunkPos,
}
