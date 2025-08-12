#[allow(unused_imports)] use bevy::prelude::*;
use bevy_ecs_tilemap::tiles::TilePos;
use superstate::{SuperstateInfo};
use serde::{Deserialize, Serialize};
use bevy::{ecs::entity::EntityHashSet, platform::collections::HashMap, prelude::*};

use crate::{terrain_gen::terrgen_resources::GlobalGenSettings, tile::tile_components::{GlobalTilePos, HashPosEntiWeightedSampler, Tile},};


use common::{common_components::*, };

#[derive(Component, Default)]
#[require(SuperstateInfo<ChunkInitState>, SessionScoped, )]
pub struct ChunkInitState;
impl ChunkInitState {
    pub const SIZE: UVec2 = UVec2 { x: 5, y: 5 };
}

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

#[derive(Component, Debug, Deserialize, Serialize, Clone, Hash, PartialEq, Eq, Reflect, Default, )]
pub struct ProducedTiles(#[entities] pub Vec<Entity>);

impl ProducedTiles {
    pub fn new_with_chunk_capacity() -> Self {
        let chunk_area = ChunkInitState::SIZE.element_product();
        let cap = chunk_area + chunk_area / 8;//TODO CALCULAR PROMEDIO DE TILES POR CHUNK
        ProducedTiles(Vec::with_capacity(cap as usize))
    }
    pub fn new<I>(entities: I) -> Self where I: IntoIterator, I::Item: Into<Entity>, {
        ProducedTiles(entities.into_iter().map(Into::into).collect())
    }

    pub fn produced_tiles(&self) -> &[Entity] { &self.0 }

    pub fn push(&mut self, entity: Entity) {self.0.push(entity);}

    fn insert_tile_recursive(
        &mut self,
        tiling_ent: Entity,
        cmd: &mut Commands,
        global_pos: GlobalTilePos,
        pos_within_chunk: TilePos,
        weight_maps: &Query<(&HashPosEntiWeightedSampler,), ()>,
        gen_settings: &GlobalGenSettings,
        depth: u32
    ) {
        if let Ok((wmap, )) = weight_maps.get(tiling_ent) {
            if let Some(tiling_ent) = wmap.sample(gen_settings, global_pos) {
                //info!(target: "tilemap", "Inserting tile {:?} at {:?} with pos within chunk {:?}", tiling_ent, global_pos, pos_within_chunk);

                if depth > 6 {
                    warn!("Tile insertion depth exceeded 6, stopping recursion for tile at {:?} with pos within chunk {:?}", global_pos, pos_within_chunk);
                    return;
                }

                self.insert_tile_recursive( tiling_ent, cmd, global_pos, pos_within_chunk, weight_maps, gen_settings, depth + 1);
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
        weight_maps: &Query<(&HashPosEntiWeightedSampler,), ()>,
        gen_settings: &GlobalGenSettings,
    ) {
        for tile in to_insert.0.iter().cloned() {
            self.insert_tile_recursive(tile, cmd, global_pos, pos_within_chunk, weight_maps, gen_settings, 0, );
        }
    }
}





#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, )]
pub struct PendingOperations(pub i32);





#[derive(Component, Debug, Default, Serialize, Deserialize, Reflect)]
pub struct ActivatingChunks(pub EntityHashSet,);



#[derive(Component, Default, Clone, Deserialize, Serialize, Copy, Hash, PartialEq, Eq, Reflect)]
pub struct ChunkPos(pub IVec2);

impl ChunkPos {
    pub fn new(x: i32, y: i32) -> Self { Self(IVec2::new(x, y)) }
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
        self.0.as_vec2() * Tile::PIXELS.as_vec2() * ChunkInitState::SIZE.as_vec2()
    }
    pub fn to_tilepos(&self) -> GlobalTilePos {
        GlobalTilePos(self.0 * ChunkInitState::SIZE.as_ivec2())
    }
}

impl From<GlobalTilePos> for ChunkPos {
    fn from(global_tile_pos: GlobalTilePos) -> Self {
        ChunkPos(global_tile_pos.0.div_euclid(ChunkInitState::SIZE.as_ivec2()))
    }
}

impl From<Vec2> for ChunkPos {
    fn from(pixel_pos: Vec2) -> Self {
        ChunkPos(pixel_pos.as_ivec2().div_euclid(Tile::PIXELS.as_ivec2() * ChunkInitState::SIZE.as_ivec2()))
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


