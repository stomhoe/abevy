use std::{hash::{DefaultHasher, Hash, Hasher}, i32};

#[allow(unused_imports)] use bevy::prelude::*;
use bevy_ecs_tilemap::tiles::TilePos;
use common::common_components::{HashId, StrId};
use rand::{Rng, seq::SliceRandom};
use serde::{Deserialize, Serialize};

use crate::{*, tilemap_shared::GlobalGenSettings};

pub trait HashablePosVec: Hash {
    fn hash_value(&self, settings: &GlobalGenSettings, dimension_hash: HashId, seed: u64) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        settings.seed.hash(&mut hasher);
        dimension_hash.hash(&mut hasher);
        seed.hash(&mut hasher);
        hasher.finish()
    }
    fn hash_true_false(&self, settings: &GlobalGenSettings, dimension_hash: HashId, extra_seed: u64) -> bool {
        self.hash_value(settings, dimension_hash, extra_seed) % 2 == 0
    }
    fn hash_for_weight_maps(&self, settings: &GlobalGenSettings, dimension_hash: HashId) -> u64 {
        self.hash_value(settings, dimension_hash, 0)
    }
    fn normalized_hash_value(&self, settings: &GlobalGenSettings, dimension_hash: HashId, seed: u64) -> f64 {
        self.hash_value(settings, dimension_hash, seed) as f64 / u64::MAX as f64
    }
    fn x(&self) -> i32;
    fn y(&self) -> i32;
}

#[derive(Component, Clone, Deserialize, Serialize, Default, Hash, PartialEq, Eq, Copy, Reflect, )]
pub struct GlobalTilePos(pub IVec2);
impl_basic_funcs!(GlobalTilePos);
impl_hashed_position!(GlobalTilePos);
impl_position_conversions!(GlobalTilePos);
impl_position_ops!(GlobalTilePos);
impl_display_debug!(GlobalTilePos, "Global pos","Gpos");

#[derive(Component, Clone, Deserialize, Serialize, Hash, PartialEq, Eq, Copy, Reflect, Debug)]
pub struct PrevGlobalTilePos(pub GlobalTilePos);
impl PrevGlobalTilePos {
    pub const PLACEHOLDER_I32_MAX: PrevGlobalTilePos = PrevGlobalTilePos(GlobalTilePos::new(i32::MAX, i32::MAX));
}

impl Default for PrevGlobalTilePos {
    fn default() -> Self {
        Self::PLACEHOLDER_I32_MAX
    }
}


impl GlobalTilePos {
    pub const TILE_SIZE_PXS: UVec2 = UVec2 { x: 32, y: 32 };

    pub fn to_tilepos(&self, /*size_in_tiles: SizeInTiles*/) -> TilePos {
        let chunk_size = ChunkPos::CHUNK_SIZE.as_ivec2();
        let ivec2 = (((Into::<IVec2>::into(*self) % chunk_size) + chunk_size) % chunk_size) /*/ size_in_tiles.inner().as_ivec2()*/;
        TilePos::from(ivec2.as_uvec2())
    }
    pub fn to_chunkpos(&self) -> ChunkPos {
        ChunkPos(Into::<IVec2>::into(*self).div_euclid(ChunkPos::CHUNK_SIZE.as_ivec2()))
    }

    pub fn to_translation(&self, prev_transform_z: f32) -> Vec3 {
        self.to_pixelpos().extend(prev_transform_z)
    }
    pub fn to_pixelpos(&self) -> Vec2 {
        self.0.as_vec2() * GlobalTilePos::TILE_SIZE_PXS.as_vec2()
    }
    impl_adjacent_position_methods!();
}
impl From<ChunkPos> for GlobalTilePos {
    fn from(chunk_pos: ChunkPos) -> Self {
        GlobalTilePos(chunk_pos.0 * ChunkPos::CHUNK_SIZE.as_ivec2())
    }
}
impl From<Vec2> for GlobalTilePos {
    fn from(pixelpos: Vec2) -> Self {
        GlobalTilePos(pixelpos.div_euclid(GlobalTilePos::TILE_SIZE_PXS.as_vec2()).as_ivec2())
    }
}
impl From<(i8, i8)> for GlobalTilePos {
    fn from(gpos: (i8, i8)) -> Self {
        GlobalTilePos(IVec2::new(gpos.0 as i32, gpos.1 as i32))
    }
}
impl Into<Vec2> for GlobalTilePos {
    fn into(self) -> Vec2 {
        self.to_pixelpos()
    }
}

#[derive(Component, Default, Clone, Deserialize, Serialize, Copy, Hash, PartialEq, Eq, )]
pub struct ChunkPos(pub IVec2);
impl_basic_funcs!(ChunkPos);
impl_hashed_position!(ChunkPos);
impl_display_debug!(ChunkPos, "Chunk pos", "Cpos");
impl_position_ops!(ChunkPos);
impl_position_conversions!(ChunkPos);

impl ChunkPos {
    pub fn rand_within_region(region_pos: RegionPos, rng: &mut impl Rng) -> Self {
        let local_x = rng.random_range(0..REGION_SIZE_IN_CHUNKS.x());
        let local_y = rng.random_range(0..REGION_SIZE_IN_CHUNKS.y());
        Self(region_pos.0 * REGION_SIZE_IN_CHUNKS.0 + IVec2::new(local_x, local_y))
    }
    pub const CHUNK_SIZE: UVec2 = UVec2::splat(30);//may change later. fed
    pub const CHUNK_AREA: usize = (Self::CHUNK_SIZE.x * Self::CHUNK_SIZE.y) as usize;

    pub fn to_pixelpos(&self) -> Vec2 {
        self.0.as_vec2() * GlobalTilePos::TILE_SIZE_PXS.as_vec2() * Self::CHUNK_SIZE.as_vec2()
    }
    pub fn to_tilepos(&self) -> GlobalTilePos {
        GlobalTilePos(self.0 * Self::CHUNK_SIZE.as_ivec2())
    }
    pub fn to_region_pos(&self) -> RegionPos {
        RegionPos(self.0.div_euclid(REGION_SIZE_IN_CHUNKS.0))
    }

    pub fn chunk_pos_from_flat_index_within_region(index: usize, region_pos: RegionPos) -> Self {
        let x = (index as i32 % REGION_SIZE_IN_CHUNKS.x()) as i32;
        let y = (index as i32 / REGION_SIZE_IN_CHUNKS.x()) as i32;
        ChunkPos(IVec2::new(x, y)) + region_pos.to_chunkpos()
    }
    pub fn flat_index_within_region(&self, region_pos: RegionPos) -> usize {
        let local_pos = *self - region_pos.to_chunkpos();
        (local_pos.0.y * REGION_SIZE_IN_CHUNKS.x() + local_pos.0.x) as usize
    }

    pub fn get_tilepositions_within_chunk(&self) -> Vec<GlobalTilePos> {
        let width = Self::CHUNK_SIZE.x as usize;
        let height = Self::CHUNK_SIZE.y as usize;
        let mut tiles = Vec::with_capacity(width * height);
        let chunk_origin = self.to_tilepos();
        for y in 0..Self::CHUNK_SIZE.y {
            for x in 0..Self::CHUNK_SIZE.x {
                let tile_pos = GlobalTilePos(IVec2::new(
                    chunk_origin.0.x + x as i32,
                    chunk_origin.0.y + y as i32,
                ));
                tiles.push(tile_pos);
            }
        }
        tiles
    }
    pub fn is_tilepos_within_chunk(&self, tile_pos: GlobalTilePos) -> Result<(), BevyError> {
        let chunk_origin = self.to_tilepos();
        let chunk_ocorner = GlobalTilePos(IVec2::new(
            chunk_origin.0.x + Self::CHUNK_SIZE.x as i32 - 1,
            chunk_origin.0.y + Self::CHUNK_SIZE.y as i32 - 1,
        ));
        let is_within = tile_pos.0.x >= chunk_origin.0.x &&
        tile_pos.0.x <= chunk_ocorner.0.x &&
        tile_pos.0.y >= chunk_origin.0.y &&
        tile_pos.0.y <= chunk_ocorner.0.y;
        if is_within {
            Ok(())
        } else {
            let dx = if tile_pos.0.x < chunk_origin.0.x {
                chunk_origin.0.x - tile_pos.0.x
            } else {
                tile_pos.0.x - chunk_ocorner.0.x
            };
            let dy = if tile_pos.0.y < chunk_origin.0.y {
                chunk_origin.0.y - tile_pos.0.y
            } else {
                tile_pos.0.y - chunk_ocorner.0.y
            };
            Err(BevyError::from(format!("{} out of chunk bounds ({},{}): {} tiles away (dx: {}, dy: {})", tile_pos, chunk_origin, chunk_ocorner, dx.max(dy), dx, dy)))
        }
    }
}

impl From<GlobalTilePos> for ChunkPos {
    fn from(global_tile_pos: GlobalTilePos) -> Self {
        ChunkPos(global_tile_pos.0.div_euclid(Self::CHUNK_SIZE.as_ivec2()))
    }
}

impl From<Vec2> for ChunkPos {
    fn from(pixel_pos: Vec2) -> Self {
        ChunkPos(pixel_pos.as_ivec2().div_euclid(GlobalTilePos::TILE_SIZE_PXS.as_ivec2() * Self::CHUNK_SIZE.as_ivec2()))
    }
}

impl From<Vec3> for ChunkPos {
    fn from(translation: Vec3) -> Self {
        ChunkPos::from(translation.xy())
    }
}

#[derive(Component, Debug, Deserialize, Serialize, Clone, Copy, PartialEq, Eq, )]
pub struct OplistSize(UVec2);
impl OplistSize {
    pub fn new([x, y]: [u32; 2]) -> Result<Self, BevyError> {
        if x <= 0 || y <= 0 {
            return Err(BevyError::from("OplistSize dimensions must be > 0"));
        }
        let max = 6;
        if x > max || y > max {
            return Err(BevyError::from(format!("OplistSize dimensions must be <= {}", max)));
        }
        Ok(Self(UVec2::new(x, y)))
    }
    pub fn x(&self) -> u32 { self.0.x }
    pub fn y(&self) -> u32 { self.0.y }
    pub fn inner(&self) -> UVec2 { self.0 }
    pub fn size(&self) -> usize { (self.x() * self.y()) as usize }
}
impl std::ops::Div<TilePos> for OplistSize {
    type Output = Self;
    fn div(self, rhs: TilePos) -> Self {
        let rhs: UVec2 = rhs.into();
        Self(self.0 / rhs)
    }
}
impl std::ops::Mul<TilePos> for OplistSize {
    type Output = Self;
    fn mul(self, rhs: TilePos) -> Self {
        let rhs: UVec2 = rhs.into();
        Self(self.0 * rhs)
    }
}

impl std::cmp::PartialOrd for OplistSize {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl std::cmp::Ord for OplistSize {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.size().cmp(&other.size())
    }
}

impl Default for OplistSize { fn default() -> Self { Self(UVec2::ONE) } }

pub const REGION_SIZE_IN_CHUNKS: ChunkPos = ChunkPos::new(256, 256);

#[derive(Component, Clone, Deserialize, Serialize, Default, Hash, PartialEq, Eq, Copy, )]
pub struct RegionPos(pub IVec2);
impl_basic_funcs!(RegionPos);
impl_hashed_position!(RegionPos);
impl_position_ops!(RegionPos);
impl_position_conversions!(RegionPos);
impl_display_debug!(RegionPos, "Region pos", "Rpos");

impl RegionPos {
    pub fn chunk_bounds(&self) -> (ChunkPos, ChunkPos) {
        let min = ChunkPos(self.0 * REGION_SIZE_IN_CHUNKS.0);
        let max = ChunkPos((self.0 + IVec2::ONE) * REGION_SIZE_IN_CHUNKS.0);
        (min, max)
    }
    pub fn contains_chunkpos(&self, cp: ChunkPos) -> bool {
        let (min, max) = self.chunk_bounds();
        cp.x() >= min.x() && cp.y() >= min.y() && cp.x() < max.x() && cp.y() < max.y()
    }
    pub fn all_chunk_positions(&self) -> Vec<ChunkPos> {
        let mut chunks = Vec::with_capacity(REGION_SIZE_IN_CHUNKS.area_usize());
        let (min, max) = self.chunk_bounds();
        for y in min.0.y..max.0.y {
            for x in min.0.x..max.0.x {
                chunks.push(ChunkPos::new(x, y));
            }
        }
        chunks
    }
    pub fn all_chunk_positions_shuffled(&self, rng: &mut impl Rng) -> Vec<ChunkPos> {
        let mut chunks = self.all_chunk_positions();
        chunks.shuffle(rng);
        chunks
    }
    pub fn to_chunkpos(&self) -> ChunkPos {
        ChunkPos(self.0 * REGION_SIZE_IN_CHUNKS.0)
    }

}

pub mod prelude {
    pub use super::{
        ChunkPos, GlobalTilePos, HashablePosVec, OplistSize, PrevGlobalTilePos, RegionPos,
        REGION_SIZE_IN_CHUNKS,
    };
}


#[derive(Component, Debug, Deserialize, Serialize, Copy, Clone, Reflect, PartialEq, Eq, Hash)]
pub struct SizeInTiles(pub UVec2);
impl SizeInTiles{
    pub fn new(str_id: &StrId, size_in_tiles: Option<(u32, u32)>, is_spritetile: bool) -> Self {
        let (mut x, mut y) = size_in_tiles.unwrap_or((1, 1));
        if x == 0 {
            error!("{}: TileOccupancy width must be greater than 0", str_id);
            x = 1;
        }
        if y == 0 {
            error!("{}: TileOccupancy height must be greater than 0", str_id);
            y = 1;
        }
        Self(UVec2::new(x, y))
    }
    pub fn inner(&self) -> UVec2 {
        self.0
    }
    pub fn to_pixel_size(&self) -> Vec2 {
       (self.0 * GlobalTilePos::TILE_SIZE_PXS).as_vec2()
    }
    pub fn tiles_per_chunk(&self) -> UVec2 {
        ChunkPos::CHUNK_SIZE / self.0
    }
    pub fn tilemap_size(&self) -> bevy_ecs_tilemap::map::TilemapSize {
        bevy_ecs_tilemap::map::TilemapSize::from(self.tiles_per_chunk())
    }
    pub fn render_chunk_size(&self) -> UVec2 {
        self.tiles_per_chunk() * 2
    }
    pub fn x(&self) -> usize {
        self.0.x as usize
    }
    pub fn y(&self) -> usize {
        self.0.y as usize
    }
}
impl Default for SizeInTiles {
    fn default() -> Self {
        Self(UVec2::ONE)
    }
}
