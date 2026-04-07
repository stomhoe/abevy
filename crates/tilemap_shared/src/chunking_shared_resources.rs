use bevy::prelude::*;
use bevy::ecs::entity::EntityHashMap;
use bevy::platform::collections::HashMap;
use common::log_targets::TILEMAP_LOAD;
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;

use crate::*;

#[derive(Resource, Clone, Default)]
pub struct LoadedChunks(pub HashMap<(DimensionRef, ChunkPos), Entity>,);

#[derive(Resource, Clone, Default)]
pub struct LoadedMacroChunks(pub HashMap<(DimensionRef, MacrochunkPos), Entity>,);

#[derive(Resource, Clone, Default)]
pub struct BeingsInCpos {
    pub by_chunk: HashMap<(DimensionRef, ChunkPos), SmallVec<[Entity; 8]>>,
    pub by_being: EntityHashMap<(DimensionRef, ChunkPos)>,
}

impl BeingsInCpos {
    pub fn set_being_chunk(
        &mut self,
        being_ent: Entity,
        dim_ref: DimensionRef,
        chunk_pos: ChunkPos,
    ) -> Option<(DimensionRef, ChunkPos)> {
        let new_key = (dim_ref, chunk_pos);
        let old_key = self.by_being.insert(being_ent, new_key);
        if old_key == Some(new_key) {
            self.insert_into_chunk(new_key, being_ent);
            return old_key;
        }
        if let Some(old_key) = old_key {
            self.remove_from_chunk(old_key, being_ent);
        }
        self.insert_into_chunk(new_key, being_ent);
        old_key
    }

    pub fn remove_being(&mut self, being_ent: Entity) -> Option<(DimensionRef, ChunkPos)> {
        let Some(old_key) = self.by_being.remove(&being_ent) else {
            return None;
        };
        self.remove_from_chunk(old_key, being_ent);
        Some(old_key)
    }

    pub fn beings_in_chunk(&self, dim_ref: DimensionRef, chunk_pos: ChunkPos) -> Option<&[Entity]> {
        self.by_chunk.get(&(dim_ref, chunk_pos)).map(SmallVec::as_slice)
    }

    fn insert_into_chunk(&mut self, key: (DimensionRef, ChunkPos), being_ent: Entity) {
        let beings = self.by_chunk.entry(key).or_default();
        if !beings.contains(&being_ent) {
            beings.push(being_ent);
        }
    }

    fn remove_from_chunk(&mut self, key: (DimensionRef, ChunkPos), being_ent: Entity) {
        let Some(beings) = self.by_chunk.get_mut(&key) else {
            return;
        };
        if let Some(idx) = beings.iter().position(|&ent| ent == being_ent) {
            beings.swap_remove(idx);
        }
        if beings.is_empty() {
            self.by_chunk.remove(&key);
        }
    }
}

pub type EntiSmallVec = SmallVec<[Entity; 4]>;

#[derive(Resource, Component, Clone, Copy, Debug, )]
#[require(ActivatingChunks)]
pub struct LoadChunksAround {
    pub chunk_visib_max_dist: f32,
    /// range in which already generated chunks are kept active
    pub chunk_active_max_dist: f32,
    /// half side of square in which chunks get generated (not shown)
    pub discovery_range: u8,
}

impl Default for LoadChunksAround {
    fn default() -> Self {
        TWO_CHUNK_RANGE_SETTINGS
    }
}

impl LoadChunksAround {
    pub fn approximate_number_of_tiles(&self, chunk_count: usize) -> usize {
        chunk_count * ChunkPos::CHUNK_SIZE.element_product() as usize
    }

    pub fn approximate_number_of_chunks(&self, multiplier: f32) -> usize {
        let multiplier = multiplier.max(0.);
        let cnt = self.discovery_range as i32;
        (((cnt * 2) - 1).pow(2) as f32 * multiplier) as usize
    }

    pub fn out_of_active_range(&self, center: &GlobalTransform, other: ChunkPos) -> bool {
        let center_pos = center.translation().xy();
        let other_pos = other.to_pixelpos();
        center_pos.distance(other_pos) > self.chunk_active_max_dist
    }

    pub fn out_of_visible_range(&self, center: &GlobalTransform, other: ChunkPos) -> bool {
        let center_pos = center.translation().xy();
        let other_pos = other.to_pixelpos();
        center_pos.distance(other_pos) > self.chunk_visib_max_dist
    }

    pub fn out_of_discovery_range(&self, center: ChunkPos, other: ChunkPos) -> bool {
        let range = self.discovery_range as i32;
        (other.0.x - center.0.x).abs() >= range || (other.0.y - center.0.y).abs() >= range
    }

    pub fn is_one_chunk(&self) -> bool {
        self.discovery_range == 1
    }
}
pub const ONE_CHUNK_RANGE_SETTINGS: LoadChunksAround = LoadChunksAround {
    chunk_visib_max_dist: 1000.0, chunk_active_max_dist: 250.0,
    discovery_range: 1,
};
pub const TWO_CHUNK_RANGE_SETTINGS: LoadChunksAround = LoadChunksAround {
    chunk_visib_max_dist: 2000.0, chunk_active_max_dist: 100.0,
    discovery_range: 2,
};
pub const NORMAL_CHUNK_RANGE_SETTINGS: LoadChunksAround = LoadChunksAround {
    chunk_visib_max_dist: 6000.0, chunk_active_max_dist: 6000.0,
    discovery_range: 4,
};
pub const EXTRA_RANGE_SETTINGS: LoadChunksAround = LoadChunksAround {
    chunk_visib_max_dist: 14000.0, chunk_active_max_dist: 14000.0,
    discovery_range: 4,
};

#[derive(Deserialize, Asset, TypePath, Clone, Debug)]
pub struct ChunkingSettingsSeri {
    pub id: String,
    #[serde(default = "default_chunk_visib_max_dist")]
    pub chunk_visib_max_dist: f32,
    #[serde(default = "default_chunk_active_max_dist")]
    pub chunk_active_max_dist: f32,
    #[serde(default = "default_discovery_range")]
    pub discovery_range: u8,
}

impl ChunkingSettingsSeri {
    pub fn to_settings(&self) -> LoadChunksAround {
        LoadChunksAround {
            chunk_visib_max_dist: self.chunk_visib_max_dist,
            chunk_active_max_dist: self.chunk_active_max_dist,
            discovery_range: self.discovery_range.max(1),
        }
    }
}

fn default_chunk_visib_max_dist() -> f32 {
    ONE_CHUNK_RANGE_SETTINGS.chunk_visib_max_dist
}

fn default_chunk_active_max_dist() -> f32 {
    ONE_CHUNK_RANGE_SETTINGS.chunk_active_max_dist
}

fn default_discovery_range() -> u8 {
    ONE_CHUNK_RANGE_SETTINGS.discovery_range
}

pub fn load_chunking_settings(mut settings: ResMut<LoadChunksAround>) {
    let db = match common::def_db::DefDatabase::<ChunkingSettingsSeri>::load_from_assets_dir_with_type(
        stringify!(ChunkingSettingsSeri),
        &["chunking.settings.ron"],
        |_| "chunking_settings",
    ) {
        Ok(db) => db,
        Err(err) => {
            error!(target: TILEMAP_LOAD, "Failed loading ChunkingSettingsSeri defs: {err:#}");
            return;
        }
    };
    let Some(first) = db.into_records().into_iter().next() else {
        return;
    };
    *settings = first.value.to_settings();
}
