use bevy::{platform::collections::HashMap, prelude::*};
use bevy_inspector_egui::prelude::*;
use game_common::game_common_components::DimensionRef;
use tilemap_shared::ChunkPos;


#[derive(Default, Debug, Resource)]
pub struct LoadedChunks (pub HashMap<(DimensionRef, ChunkPos), Entity>,);
//pub struct LoadedChunks (pub HashMap<ChunkPos, Entity>,);

#[derive(Resource, Reflect, InspectorOptions)]
#[reflect(Resource, Default, InspectorOptions)]
pub struct AaChunkRangeSettings {
    #[inspector(min = 0., max = 10000.)]
    pub chunk_visib_max_dist: f32,
    #[inspector(min = 0., max = 10000.)]
    pub chunk_active_max_dist: f32,
    #[inspector(min = 0, max = 6)]
    pub chunk_show_range: u8,
}
impl Default for AaChunkRangeSettings {
    fn default() -> Self {
        DEBUG_CHUNK_RANGE_SETTINGS
    }
}


pub const DEBUG_CHUNK_RANGE_SETTINGS: AaChunkRangeSettings = AaChunkRangeSettings {
    chunk_visib_max_dist: 300.0,
    chunk_active_max_dist: 300.0,
    chunk_show_range: 0,
};

pub const NORMAL_CHUNK_RANGE_SETTINGS: AaChunkRangeSettings = AaChunkRangeSettings {
    chunk_visib_max_dist: 300.0,
    chunk_active_max_dist: 300.0,
    chunk_show_range: 2,
};