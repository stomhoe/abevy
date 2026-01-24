#[allow(unused_imports)] use bevy::prelude::*;
use dimension_shared::DimensionRef;
use game_common::game_common_components::EntityZeroRef;
use tilemap_shared::{ChunkPos, GlobalTilePos, RegionPos};

use crate::{tile::tile_components::*};



#[derive(Message, Debug, Clone, )]
pub struct OfferChunk {
    // for determinism
    pub i: u64,
    pub region_ent: Entity,// extraer region pos de una query
    pub structured_gen_cfg_ent: Entity,
    pub start_pos: ChunkPos, 
}

#[derive(Message, Debug, Clone, Hash, PartialEq, Eq, Reflect)]
pub struct ChunksClaim {
    pub i: u64,
    pub region_ent: Entity,
    pub sgc_ent: Entity,
    //TODO: chequear que cada chunkpos no se salga de su respectiva region
    pub chunks_gpos: Vec<ChunkPos>, 
    pub partition_tolerant: bool,
}
impl Default for ChunksClaim {
    fn default() -> Self {
        Self { i: 0, region_ent: Entity::PLACEHOLDER, sgc_ent: Entity::PLACEHOLDER, chunks_gpos: Vec::new(), partition_tolerant: false }
    }
}


#[derive(Message, Debug, Clone, Hash, PartialEq, Eq)]
pub struct StructurePrepareTilesOrder {
    pub i: u64,
    pub structured_gen_cfg_ent: Entity,
    pub region_pos: RegionPos,
    pub dimension_ref: DimensionRef,
    //sucessfully claimed global chunk positions
    pub chunks_gpos: Vec<ChunkPos>, 
}


#[derive(Message, Debug, )]
pub struct StructureBuildCompliance {
    pub structure_gen_cfg_ent: Entity,
    pub dimension_ref: DimensionRef,
    pub chunk_pos: ChunkPos,
    pub tiles: Vec<(GlobalTilePos, EntityZeroRef, Option<DeleteOtherTiles>)>,

}


#[derive(Message, Debug, )]
pub struct RecheckRegion(pub Entity);