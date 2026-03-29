use bevy::prelude::*;
use tilemap_shared::{DimensionRef, GlobalTilePos, MacrochunkPos};

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NavIslandRef {
    pub dim_ref: DimensionRef,
    pub macrochunk_pos: MacrochunkPos,
    pub island_ix: u32,
}

#[derive(Debug, Clone)]
pub struct NavIslandPortal {
    pub target: NavIslandRef,
    pub anchor: GlobalTilePos,
}

#[derive(Debug, Clone, Default)]
pub struct NavIsland {
    pub sample_points: Vec<GlobalTilePos>,
    pub portals: Vec<NavIslandPortal>,
    pub area_tiles: u32,
}

#[derive(Component, Debug, Default, Clone)]
pub struct MacroChunkNavIslands(pub Vec<NavIsland>);

impl MacroChunkNavIslands {
    pub fn new(islands: Vec<NavIsland>) -> Self {
        Self(islands)
    }
}
