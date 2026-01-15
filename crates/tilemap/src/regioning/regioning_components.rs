#[allow(unused_imports)] use bevy::prelude::*;
use bevy_replicon::prelude::Replicated;
use game_common::game_common_components::{Direction, EntityZeroRef};
use serde::{Deserialize, Serialize};
use bevy::{ecs::{entity::{EntityHashMap, EntityHashSet, MapEntities}, entity_disabling::Disabled}, platform::collections::{HashMap, HashSet, hash_map::Entry}, prelude::*};
use tilemap_shared::{ChunkPos, GlobalTilePos, REGION_SIZE_IN_CHUNKS, RegionPos};

use crate::{chunking_components::Chunk, chunking_resources::AaChunkRangeSettings, regioning::regioning_messages::ClaimedChunks, tile::tile_components::*};


use common::{common_components::*, };

#[derive(Component, Debug, Default, Deserialize, Serialize, Copy, Clone, Reflect)]
#[require(SessionScoped, TgenHotLoadingScoped, RegionStructures, RegionPlannedTiles )]
pub struct Region;

#[derive(Component, Debug, Reflect)]
#[relationship_target(relationship = Chunk)]
pub struct ChunksActiveInRegion(Vec<Entity>);
impl ChunksActiveInRegion { pub fn entities(&self) -> &[Entity] { &self.0 } }

#[derive(Component, Debug, Deserialize, Serialize, Clone, Reflect)]
#[require(Replicated, EntityPrefix::new_truncated("StructureGenCfg"))]
pub struct StructuredGenConfig{
    pub structure_id: StrId,
    pub max_per_region: u32,
    pub args: Vec<String>,
}
impl Default for StructuredGenConfig {
    fn default() -> Self {
        Self { structure_id: StrId::default(), max_per_region: 1024, args: Vec::new()  }
    }
}

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


#[derive(Component, Debug, Default, Deserialize, Serialize, Copy, Clone, Reflect)]
#[require(Replicated, EntityPrefix::new_truncated("StructGenCfgWMap"))]
pub struct StructuredGenCfgsWeightedMap;


#[derive(Debug, Reflect)]
pub struct StrGenGrid{
    occupied_chunks_grid: [[Option<Entity>; 32]; 32],
    occupied_chunks_count: u32,
}
impl Default for StrGenGrid {
    fn default() -> Self {
        Self {
            occupied_chunks_grid: [[None; 32]; 32],
            occupied_chunks_count: 0,
        }
    }
}
pub enum ChunkOccupyError {
    AlreadyOccupied(Entity),
    OutOfRegionBounds(Direction),
}

impl StrGenGrid {
    pub fn is_occupied(&self, global_chunk_pos: ChunkPos, region_pos: RegionPos) -> bool {
        let local_chunk_pos = global_chunk_pos - region_pos.to_chunkpos();
        let x = local_chunk_pos.0.x as usize;
        let y = local_chunk_pos.0.y as usize;
        self.occupied_chunks_grid[y][x].is_some()
    }
    pub fn is_available(&self, global_chunk_pos: ChunkPos, region_pos: RegionPos) -> bool {
        !self.is_occupied(global_chunk_pos, region_pos)
    }
    pub fn occupy(&mut self, global_chunk_pos: ChunkPos, region_pos: RegionPos, struct_gen_cfg_ent: Entity) -> Result<(), ChunkOccupyError> {
        let local_chunk_pos = global_chunk_pos - region_pos.to_chunkpos();
        let x = local_chunk_pos.0.x as usize;
        let y = local_chunk_pos.0.y as usize;
        match (local_chunk_pos.0.x < 0, local_chunk_pos.0.x >= REGION_SIZE_IN_CHUNKS.x(), local_chunk_pos.0.y < 0, local_chunk_pos.0.y >= REGION_SIZE_IN_CHUNKS.y()) {
            (true, _, _, _) => return Err(ChunkOccupyError::OutOfRegionBounds(Direction::West)),
            (_, true, _, _) => return Err(ChunkOccupyError::OutOfRegionBounds(Direction::East)),
            (_, _, true, _) => return Err(ChunkOccupyError::OutOfRegionBounds(Direction::South)),
            (_, _, _, true) => return Err(ChunkOccupyError::OutOfRegionBounds(Direction::North)),
            _ => {}
        }
        
        match self.occupied_chunks_grid[y][x] {
            None => {
                self.occupied_chunks_grid[y][x] = Some(struct_gen_cfg_ent);
                self.occupied_chunks_count += 1;
                Ok(())
            }
            Some(entity) => Err(ChunkOccupyError::AlreadyOccupied(entity)),
        }
    }
    pub fn free(&mut self, global_chunk_pos: ChunkPos, region_pos: RegionPos) {
        let local_chunk_pos = global_chunk_pos - region_pos.to_chunkpos();
        let x = local_chunk_pos.0.x as usize;
        let y = local_chunk_pos.0.y as usize;
        if self.occupied_chunks_grid[y][x].is_some() {
            self.occupied_chunks_grid[y][x] = None;
            self.occupied_chunks_count -= 1;
        }
    }
    pub fn occupied_count(&self) -> u32 {
        self.occupied_chunks_count
    }
}

#[derive(Component, Debug, Reflect)]
pub struct RegionStructures { 
    pub processed_up_to_i: usize,
    pub claims: [Option<ClaimedChunks>; MAX_CLAIMS],
    pub struct_gen_counts: EntityHashMap<u32>,
    pub strgen_grid: StrGenGrid,
}

impl Default for RegionStructures {
    fn default() -> Self {
        Self { 
            claims: [(); MAX_CLAIMS].map(|_| None),
            processed_up_to_i: 0,
            struct_gen_counts: EntityHashMap::default(),
            strgen_grid: StrGenGrid::default(),
        }
    }
}

pub type TilesFromBuilder = Vec<(GlobalTilePos, EntityZeroRef, Option<DeleteOthersExceptZLevels>)>;

#[derive(Component, Debug, Reflect, Default)]
//a region's component, doesn't need dimension
pub struct RegionPlannedTiles { 
    map: HashMap<ChunkPos, Vec<(GlobalTilePos, EntityZeroRef, Option<DeleteOthersExceptZLevels>)>>,
    chunks_pending_build: HashSet<ChunkPos>,
}
impl RegionPlannedTiles {
    pub fn new(chunk_positions: &[ChunkPos]) -> Self {
        Self {
            map: HashMap::new(),
            chunks_pending_build: chunk_positions.iter().copied().collect(),
        }
    }

    pub fn extend_pending_chunks(&mut self, chunk_positions: &[ChunkPos]) {
        self.chunks_pending_build.extend(chunk_positions.iter().copied());
    }

    fn validate_chunk_pending(&self, chunk_pos: ChunkPos) -> Result<(), BevyError> {
        if !self.chunks_pending_build.contains(&chunk_pos) {
            return Err(BevyError::from(format!(
                "ChunkPos {:?} is not pending a build order",
                chunk_pos
            )));
        }
        Ok(())
    }

    pub fn add_planned_tiles(
        &mut self,
        chunk_pos: ChunkPos,
        tile_data: Vec<(GlobalTilePos, EntityZeroRef, Option<DeleteOthersExceptZLevels>)>,
    ) -> Result<bool, BevyError> {
        self.validate_chunk_pending(chunk_pos)?;

        for (tile_pos, _, _) in &tile_data {
            chunk_pos.is_tilepos_within_chunk(*tile_pos)?;
        }

        self.map.entry(chunk_pos).or_insert_with(Vec::new).extend(tile_data);
        self.chunks_pending_build.remove(&chunk_pos);
        Ok(self.chunks_pending_build.is_empty())
    }

    pub fn get(
        &self,
        chunk_pos: &ChunkPos,
    ) -> Option<&Vec<(GlobalTilePos, EntityZeroRef, Option<DeleteOthersExceptZLevels>)>> {
        self.map.get(chunk_pos)
    }
}

pub const MAX_CLAIMS: usize = 1024;


#[derive(Component, Debug, Default, Deserialize, Serialize, Copy, Clone, Reflect)]
pub struct RegionPlanningFinished;