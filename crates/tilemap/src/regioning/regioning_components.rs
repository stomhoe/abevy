#[allow(unused_imports)] use bevy::prelude::*;
use bevy_replicon::prelude::Replicated;
use serde::{Deserialize, Serialize};
use bevy::{ecs::{entity::{EntityHashMap, EntityHashSet, MapEntities}, entity_disabling::Disabled}, platform::collections::{HashMap, HashSet}, prelude::*};
use tilemap_shared::ChunkPos;

use crate::{chunking_components::Chunk, chunking_resources::AaChunkRangeSettings, regioning::regioning_messages::ClaimedChunks, tile::tile_components::*};


use common::{common_components::*, };

#[derive(Component, Debug, Default, Deserialize, Serialize, Copy, Clone, Reflect)]
#[require(SessionScoped, TgenHotLoadingScoped, RegionStructures)]
pub struct Region;

#[derive(Component, Debug, Reflect)]
#[relationship_target(relationship = Chunk)]
pub struct ChunksActiveInRegion(Vec<Entity>);
impl ChunksActiveInRegion { pub fn entities(&self) -> &[Entity] { &self.0 } }

#[derive(Component, Debug, Deserialize, Serialize, Clone, Reflect)]
#[require(Replicated)]
pub struct StructuredGenConfig{
    pub structure_id: String,
    pub max_per_region: u8,
}
impl Default for StructuredGenConfig {
    fn default() -> Self {
        Self { structure_id: String::new(), max_per_region: 255 }
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
pub struct StructuredGenConfigWeightedMap;

pub struct PlannedStructures{}//TODO put something inside


#[derive(Component, Debug, Clone)]
pub struct RegionStructures { 
    pub processed_up_to_i: usize,
    pub vec: [Option<ClaimedChunks>; MAX_CLAIMS],
    pub struct_gen_counts: EntityHashMap<u32>,
    pub occupied_chunks_grid: [[Option<Entity>; 32]; 32],
}

impl Default for RegionStructures {
    fn default() -> Self {
        Self { 
            vec: [(); MAX_CLAIMS].map(|_| None),
            processed_up_to_i: 0,
            struct_gen_counts: EntityHashMap::default(),
            occupied_chunks_grid: [[None; 32]; 32],
        }
    }
}

pub const MAX_CLAIMS: usize = 1024;
