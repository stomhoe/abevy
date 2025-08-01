use bevy::platform::collections::HashSet;
#[allow(unused_imports)] use bevy::prelude::*;
use bevy_ecs_tilemap::tiles::TilePos;
use serde::{Deserialize, Serialize};

#[derive(Component, Debug, Default, Serialize, Deserialize)]
pub struct ActivatesChunks(pub HashSet<Entity>,);


use superstate::{SuperstateInfo};

use crate::game::{game_utils::WeightedMap, tilemap::{chunking_resources::CHUNK_SIZE, tile::{tile_components::GlobalTilePos, tile_constants::TILE_SIZE_PXS}}};

#[derive(Component, Default)]
#[require(SuperstateInfo<ChunkInitState>)]
pub struct ChunkInitState;

#[derive(Component, Debug, Default, )]
#[require(ChunkInitState)]
#[require(Visibility::Hidden)]
pub struct UninitializedChunk;

#[derive(Component, Debug)]
#[require(ChunkInitState)]
pub struct TilesReady;

#[derive(Component, Debug, Default, )]
#[require(ChunkInitState)]
pub struct LayersReady;

#[derive(Component, Debug, Default,)]
#[require(ChunkInitState)]
pub struct InitializedChunk;

#[derive(Component, Debug, Deserialize, Serialize, Clone, Hash, PartialEq, Eq)]
pub struct ProducedTiles(pub Vec<Entity>);
impl Default for ProducedTiles {
    fn default() -> Self {
        let chunk_area = (CHUNK_SIZE.x as usize) * (CHUNK_SIZE.y as usize);
        let cap = chunk_area + chunk_area / 10;
        ProducedTiles(Vec::with_capacity(cap))
    }
}
impl ProducedTiles {
    pub fn insert_cloned_with_pos(&self, cmd: &mut Commands, destination: &mut Self, global_pos: GlobalTilePos, pos_within_chunk: TilePos, ) {
        for tile in self.0.iter().cloned() {
            let tile = cmd.entity(tile).clone_and_spawn().insert((global_pos, pos_within_chunk)).id();
            destination.0.push(tile);
        }
    }
}


#[derive(Component, Default, Clone, Deserialize, Serialize, Copy, Hash, PartialEq, Eq)]
pub struct ChunkPos(pub IVec2);

impl ChunkPos {
    pub fn new(x: i32, y: i32) -> Self {
        Self(IVec2::new(x, y))
    }
    pub fn x(&self) -> i32 { self.0.x }
    pub fn y(&self) -> i32 { self.0.y }
}

impl std::fmt::Debug for ChunkPos {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ChunkPos({}, {})", self.0.x, self.0.y)
    }
}

impl std::fmt::Display for ChunkPos {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.0.x, self.0.y)
    }
}

impl ChunkPos {
    pub fn to_pixelpos(&self) -> Vec2 {
        self.0.as_vec2() * TILE_SIZE_PXS.as_vec2() * CHUNK_SIZE.as_vec2()
    }
    pub fn to_tilepos(&self) -> GlobalTilePos {
        GlobalTilePos(self.0 * CHUNK_SIZE.as_ivec2())
    }
}

impl From<GlobalTilePos> for ChunkPos {
    fn from(global_tile_pos: GlobalTilePos) -> Self {
        ChunkPos(global_tile_pos.0.div_euclid(CHUNK_SIZE.as_ivec2()))
    }
}

impl From<Vec2> for ChunkPos {
    fn from(pixel_pos: Vec2) -> Self {

            
        ChunkPos(pixel_pos.as_ivec2().div_euclid(TILE_SIZE_PXS.as_ivec2() * CHUNK_SIZE.as_ivec2()))
    }
}

impl std::ops::Add for ChunkPos {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output { ChunkPos(self.0 + rhs.0) }
}
impl std::ops::Sub for ChunkPos {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output { ChunkPos(self.0 - rhs.0) }
}


#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, )]
pub struct PendingOperations(pub i32);
