#[allow(unused_imports)] use bevy::prelude::*;
use bevy_replicon::prelude::Replicated;
use serde::{Deserialize, Serialize};
use bevy::{ecs::{entity::{EntityHashSet, MapEntities}, entity_disabling::Disabled}, platform::collections::{HashMap, HashSet}, prelude::*};
use tilemap_shared::{ChunkPos, GlobalTilePos, RegionPos};

use crate::{chunking_components::Chunk, chunking_resources::AaChunkRangeSettings, tile::tile_components::*};


use common::{common_components::*, };

#[derive(Message, Debug, Clone, )]
pub struct OfferChunk {
    // for determinism
    pub i: u64,
    pub region_ent: Entity,// extraer region pos de una query
    pub structured_gen_cfg_ent: Entity,
    pub start_gpos: ChunkPos, 
}

#[derive(Message, Debug, Clone, Hash, PartialEq, Eq)]
pub struct ClaimedChunks {
    pub i: u64,
    pub region_ent: Entity,
    pub structured_gen_cfg_ent: Entity,
    //TODO: chequear que cada chunkpos no se salga de su respectiva region
    pub chunks_gpos: Vec<ChunkPos>, 
    pub partition_tolerant: bool,
}
impl Default for ClaimedChunks {
    fn default() -> Self {
        Self { i: 0, region_ent: Entity::PLACEHOLDER, structured_gen_cfg_ent: Entity::PLACEHOLDER, chunks_gpos: Vec::new(), partition_tolerant: false }
    }
}


#[derive(Message, Debug, Clone, Hash, PartialEq, Eq)]
pub struct StructureBuildOrder {
    pub i: u64,
    pub structured_gen_cfg_ent: Entity,
    pub region_ent: Entity,
    //global chunk positions
    pub chunks_gpos: Vec<ChunkPos>, 
}