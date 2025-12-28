#[allow(unused_imports)] use bevy::prelude::*;
use bevy_ecs_tilemap::tiles::TilePos;
use bevy_replicon::prelude::Replicated;
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
pub struct Chunk {#[entities]#[relationship]pub region: Entity,}

#[derive(Component, Debug, Reflect)]
#[relationship_target(relationship = Chunk)]
pub struct ChunksInRegion(Vec<Entity>);
impl ChunksInRegion { pub fn entities(&self) -> &[Entity] { &self.0 } }


/*

*/
#[derive(Component, Debug, Deserialize, Serialize, Copy, Clone, Hash, PartialEq, Eq, Reflect, MapEntities)]
#[relationship(relationship_target = AcceptedFilters)]
pub struct WhitelistedFilterOf {
    #[relationship] #[entities]
    pub structured_gen_cfg: Entity,
}
impl WhitelistedFilterOf{
    pub fn new(structured_gen_cfg: Entity) -> Self {
        Self { structured_gen_cfg }
    }
}

#[derive(Component, Debug, Reflect)]
#[relationship_target(relationship = WhitelistedFilterOf)]
pub struct AcceptedFilters(Vec<Entity>);
impl AcceptedFilters { pub fn entities(&self) -> &[Entity] { &self.0 } }