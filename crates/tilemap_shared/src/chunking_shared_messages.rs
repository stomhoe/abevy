#[allow(unused_imports)] use bevy::platform::collections::HashMap;
use bevy::{prelude::*};

use crate::{ChunkPos, DimensionRef};

#[derive(Message, Debug, Clone, Copy)]
pub struct UpdateActivatedChunkPos {
    pub being_ent: Entity,
}

#[derive(Message, Debug, Clone, Copy)]
pub struct ChunkWithBeingsWantsDespawn {
    pub chunk_ent: Entity,
}

#[derive(Message, Debug, Clone, Copy)]
pub struct ChunkBeingsChanged {
    pub dim: DimensionRef,
    pub cpos: ChunkPos,
}

#[derive(Debug, Message, Clone, Copy)]
pub struct CheckIfChunkShouldDespawn(pub Entity);

#[derive(Debug, Message, Clone, Copy, Default)]
pub struct ForceAllChunksDespawn;

#[derive(Debug, Message, Clone, Copy)]
pub struct MakeChunkDespawn {
    pub chunk_ent: Entity,
    pub reschedule_if_beings_present: bool,
}

impl MakeChunkDespawn {
    pub fn default(chunk_ent: Entity) -> Self {
        Self { chunk_ent, reschedule_if_beings_present: true }
    }
    pub fn new_no_delegate_if_beings(chunk_ent: Entity) -> Self {
        Self { chunk_ent, reschedule_if_beings_present: false }
    }
}
