#[allow(unused_imports)] use bevy::prelude::*;
use bevy_ecs_tilemap::tiles::TilePos;
use debug_unwraps::{DebugUnwrapErrExt, DebugUnwrapExt};
use game_common::{game_common_components_samplers::EntityWeightedSampler};
use serde::{Deserialize, Serialize};
use bevy::{ecs::{entity::{EntityHashSet, MapEntities}, entity_disabling::Disabled}, platform::collections::{HashMap, HashSet}, prelude::*};

use crate::{chunking_resources::AaChunkRangeSettings, tile::tile_components::*};
use ::tilemap_shared::*;


use common::{common_components::*, };


#[derive(Component, Debug, Default, Deserialize, Serialize, Copy, Clone, Reflect)]
pub struct ComponentName;


/*
           .replicate::<Chunk>()
           .register_type::<Chunk>()
           .register_type::<ActiveChunks>()
*/
#[derive(Component, Debug, Deserialize, Serialize, Copy, Clone, Hash, PartialEq, Eq, Reflect, MapEntities)]
#[relationship(relationship_target = ChunksInRegion)]
pub struct Chunk {
    #[relationship]
    pub region: Entity,
}

#[derive(Component, Debug, Reflect)]
#[relationship_target(relationship = Chunk)]
pub struct ChunksInRegion(Vec<Entity>);
impl ChunksInRegion { pub fn entities(&self) -> &[Entity] { &self.0 } }