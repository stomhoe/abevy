use bevy::{platform::collections::HashMap, prelude::*};
use bevy_inspector_egui::prelude::*;
use dimension_shared::DimensionRef
;
use tilemap_shared::ChunkPos;


#[derive(Resource, Reflect, InspectorOptions, Default)]
#[reflect(Resource, Default, InspectorOptions)]
pub struct LoadedChunks (pub HashMap<(DimensionRef, ChunkPos), Entity>,);

#[derive(Resource, Reflect, InspectorOptions)]
#[reflect(Resource, Default, InspectorOptions)]
pub struct AaChunkRangeSettings {
    #[inspector(min = 0., max = 10000.)]
    pub chunk_visib_max_dist: f32,
    #[inspector(min = 0., max = 10000.)]
    pub chunk_active_max_dist: f32,
    #[inspector(min = 1, max = 15)]
    pub chunk_show_range: u8,
}
impl Default for AaChunkRangeSettings {
    fn default() -> Self {
        DEBUG_CHUNK_RANGE_SETTINGS
    }
}

impl AaChunkRangeSettings {
    pub fn approximate_number_of_tiles(&self) -> usize {
        let ret = self.approximate_number_of_chunks() * ChunkPos::CHUNK_SIZE.element_product() as usize;
        //info!("Approximate number of tiles per chunk range settings: {}", ret);
        ret
    }
    pub fn approximate_number_of_chunks(&self) -> usize {
        let cnt = self.chunk_show_range as i32;
        ((cnt * 2) - 1).pow(2) as usize
    }
}


pub const DEBUG_CHUNK_RANGE_SETTINGS: AaChunkRangeSettings = AaChunkRangeSettings {
    chunk_visib_max_dist: 1000.0,
    chunk_active_max_dist: 250.0,
    chunk_show_range: 1,
};

pub const NORMAL_CHUNK_RANGE_SETTINGS: AaChunkRangeSettings = AaChunkRangeSettings {
    chunk_visib_max_dist: 6000.0,
    chunk_active_max_dist: 6000.0,
    chunk_show_range: 4,
};

pub const EXTRA_RANGE_SETTINGS: AaChunkRangeSettings = AaChunkRangeSettings {
    chunk_visib_max_dist: 14000.0,
    chunk_active_max_dist: 14000.0,
    chunk_show_range: 8,
};