use std::collections::HashMap;
use bevy::prelude::*;

use tilemap_shared::{ChunkPos, DimensionRef};

#[derive(Resource, Debug, Default)]
pub struct FrozenBgSimulatedBeingsMap(pub HashMap<(DimensionRef, ChunkPos), Vec<Entity>>);
