use bevy::platform::collections::HashSet;
#[allow(unused_imports)] use bevy::prelude::*;
use bevy_ecs_tilemap::tiles::TilePos;
use serde::{Deserialize, Serialize};

#[derive(Component, Debug, Default, Serialize, Deserialize)]
pub struct ActivatesChunks(pub HashSet<Entity>,);


use superstate::{SuperstateInfo};

use crate::game::{game_utils::WeightedMap, tilemap::{chunking_resources::CHUNK_SIZE, terrain_gen::terrgen_resources::WorldGenSettings, tile::{tile_components::{GlobalTilePos, TileWeightedSampler}, tile_utils::TILE_SIZE_PXS}}};

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
pub struct ProducedTiles(#[entities] Vec<Entity>);
impl Default for ProducedTiles {
    fn default() -> Self {
        ProducedTiles(Vec::new())
    }
}
impl ProducedTiles {
    pub fn new_with_chunk_capacity() -> Self {
        let chunk_area = (CHUNK_SIZE.x as usize) * (CHUNK_SIZE.y as usize);
        let cap = chunk_area + chunk_area / 8;//TODO CALCULAR PROMEDIO DE TILES POR CHUNK
        ProducedTiles(Vec::with_capacity(cap))
    }
    pub fn new<I>(entities: I) -> Self where I: IntoIterator, I::Item: Into<Entity>, {
        ProducedTiles(entities.into_iter().map(Into::into).collect())
    }

    pub fn produced_tiles(&self) -> &[Entity] { &self.0 }
    pub fn drain(&mut self) -> Vec<Entity> { std::mem::take(&mut self.0) }

    pub fn push(&mut self, entity: Entity) {self.0.push(entity);}

    fn insert_tile_recursive(
        &mut self,
        tiling_ent: Entity,
        cmd: &mut Commands,
        global_pos: GlobalTilePos,
        pos_within_chunk: TilePos,
        weight_maps: &Query<(&TileWeightedSampler,), ()>,
        gen_settings: &WorldGenSettings,
    ) {
        if let Ok((wmap, )) = weight_maps.get(tiling_ent) {
            if let Some(tiling_ent) = wmap.sample(gen_settings, global_pos) {
                info!(target: "tilemap", "Inserting tile {:?} at {:?} with pos within chunk {:?}", tiling_ent, global_pos, pos_within_chunk);
                self.insert_tile_recursive(
                    tiling_ent, cmd, global_pos, pos_within_chunk, weight_maps, gen_settings
                );
            }
        } else {
            let tile_ent = cmd.entity(tiling_ent).clone_and_spawn().insert((global_pos, pos_within_chunk)).id();
            self.0.push(tile_ent);
        }
    }

    pub fn insert_clonespawned_with_pos(
        &mut self,
        to_insert: &ProducedTiles,
        cmd: &mut Commands,
        global_pos: GlobalTilePos,
        pos_within_chunk: TilePos,
        weight_maps: &Query<(&TileWeightedSampler,), ()>,
        gen_settings: &WorldGenSettings,
    ) {
        for tile in to_insert.0.iter().cloned() {
            self.insert_tile_recursive(tile, cmd, global_pos, pos_within_chunk, weight_maps, gen_settings, );
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
