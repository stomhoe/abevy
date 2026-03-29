use std::hash::{DefaultHasher, Hash, Hasher};

#[allow(unused_imports)] use bevy::prelude::*;
use bevy_ecs_tilemap::tiles::TilePos;
use common::common_components::{HashId, StrId};
use rand::{Rng, seq::SliceRandom};
use serde::{Deserialize, Serialize};

use crate::{tilemap_shared::GlobalGenSettings, *};

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

#[derive(Component, Clone, Deserialize, Serialize, Hash, PartialEq, Eq, Copy, Debug)]
pub struct PrevPos {
    pub dim: DimensionRef,
    pub gpos: GlobalTilePos,
}


impl GlobalTilePos {
    pub const TILE_SIZE_PXS: UVec2 = UVec2 { x: 32, y: 32 };
    pub const ONE: Self = Self(IVec2::ONE);

    pub fn to_tilepos(&self) -> TilePos {
        let chunk_size = ChunkPos::CHUNK_SIZE.as_ivec2();
        let ivec2 = ((Into::<IVec2>::into(*self) % chunk_size) + chunk_size) % chunk_size;
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
    pub fn taxicab_tile_distance(&self, other: GlobalTilePos) -> f32 {
        let delta = (other.0 - self.0).abs();
        (delta.x + delta.y) as f32
    }
    pub fn direct_chase_dir(
        &self,
        target_pos: GlobalTilePos,
        stop_distance: f32,
    ) -> Option<Vec2> {
        if self.taxicab_tile_distance(target_pos) <= stop_distance.max(0.0) {
            return None;
        }
        let delta = target_pos.0 - self.0;
        let step = if delta == IVec2::ZERO {
            IVec2::ZERO
        } else if delta.x.abs() >= delta.y.abs() {
            IVec2::new(delta.x.signum(), 0)
        } else {
            IVec2::new(0, delta.y.signum())
        };
        if step == IVec2::ZERO {
            return None;
        }
        Some(step.as_vec2())
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
pub struct MacrochunkPos(pub IVec2);
impl_basic_funcs!(MacrochunkPos);
impl_hashed_position!(MacrochunkPos);
impl_display_debug!(MacrochunkPos, "Macrochunk pos", "Mpos");
impl_position_ops!(MacrochunkPos);
impl_position_conversions!(MacrochunkPos);


impl MacrochunkPos {
    pub const SIZE_IN_CHUNKS: ChunkPos = ChunkPos::splat(8);
    pub fn chunk_bounds(&self) -> (ChunkPos, ChunkPos) {
        let min = ChunkPos(self.0 * Self::SIZE_IN_CHUNKS.0);
        let max = ChunkPos((self.0 + IVec2::ONE) * Self::SIZE_IN_CHUNKS.0);
        (min, max)
    }
    pub fn to_chunkpos(&self) -> ChunkPos {
        ChunkPos(self.0 * Self::SIZE_IN_CHUNKS.0)
    }
    pub fn contains_chunkpos(&self, cp: ChunkPos) -> bool {
        let (min, max) = self.chunk_bounds();
        cp.x() >= min.x() && cp.y() >= min.y() && cp.x() < max.x() && cp.y() < max.y()
    }
    pub fn sample_macro_chunk_positions<'a>(&self, n_points: usize, out: &'a mut Vec<GlobalTilePos>) -> &'a [GlobalTilePos] {
        let (min_chunk, max_chunk_excl) = self.chunk_bounds();
        let chunk_count = (max_chunk_excl.x() - min_chunk.x()) as usize * (max_chunk_excl.y() - min_chunk.y()) as usize;
        out.reserve(chunk_count * n_points * n_points);
        for y in min_chunk.y()..max_chunk_excl.y() {
            for x in min_chunk.x()..max_chunk_excl.x() {
                ChunkPos::new(x, y).get_n_equally_spaced_sample_points(n_points, out);
            }
        }
        out.as_slice()
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
    pub fn to_macrochunk_pos(&self) -> MacrochunkPos {
        MacrochunkPos(self.0.div_euclid(MacrochunkPos::SIZE_IN_CHUNKS.0))
    }
    pub fn bit_index_in_chunk(&self, gpos: GlobalTilePos) -> Option<usize> {
        let local = gpos.0 - self.to_tilepos().0;
        if local.x < 0 || local.y < 0 {
            return None;
        }
        let x = local.x as usize;
        let y = local.y as usize;
        if x >= Self::CHUNK_SIZE.x as usize || y >= Self::CHUNK_SIZE.y as usize {
            return None;
        }
        Some(y * Self::CHUNK_SIZE.x as usize + x)
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

    pub fn clamp_gpos_to_chunk(&self, gpos: GlobalTilePos) -> GlobalTilePos {
        let min_tile = self.to_tilepos().0;
        let max_tile = min_tile + Self::CHUNK_SIZE.as_ivec2() - IVec2::ONE;
        GlobalTilePos(IVec2::new(
            gpos.0.x.clamp(min_tile.x, max_tile.x),
            gpos.0.y.clamp(min_tile.y, max_tile.y),
        ))
    }

    pub fn random_gpos_within(&self, rng: &mut impl Rng) -> GlobalTilePos {
        let chunk_origin = self.to_tilepos().0;
        GlobalTilePos(IVec2::new(
            chunk_origin.x + rng.random_range(0..Self::CHUNK_SIZE.x as i32),
            chunk_origin.y + rng.random_range(0..Self::CHUNK_SIZE.y as i32),
        ))
    }
    pub fn get_n_equally_spaced_sample_points<'a>(&self, n: usize, out: &'a mut Vec<GlobalTilePos>) -> &'a [GlobalTilePos] {
        if n == 0 {
            return out.as_slice();
        }
        let sample_steps = n as i32 + 1;
        let chunk_origin = self.to_tilepos().0;
        out.reserve(n * n);
        for sample_y in 0..n {
            let y = (Self::CHUNK_SIZE.y as i32 * (sample_y as i32 + 1)) / sample_steps;
            for sample_x in 0..n {
                let x = (Self::CHUNK_SIZE.x as i32 * (sample_x as i32 + 1)) / sample_steps;
                out.push(GlobalTilePos(chunk_origin + IVec2::new(x, y)));
            }
        }
        out.as_slice()
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



#[derive(Component, Deserialize, Serialize, Clone, Copy, PartialEq, Eq, )]
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
impl_display_debug!(OplistSize, "OplistSize", "OplistSize");

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

//don't increase further
pub const REGION_SIZE_IN_CHUNKS: ChunkPos = ChunkPos::splat(64);

#[derive(Component, Clone, Deserialize, Serialize, Default, Hash, PartialEq, Eq, Copy, )]
pub struct RegionPos(pub IVec2);
impl_basic_funcs!(RegionPos);
impl_hashed_position!(RegionPos);
impl_position_ops!(RegionPos);
impl_position_conversions!(RegionPos);
impl_display_debug!(RegionPos, "Region pos", "Rpos");

impl RegionPos {
    pub fn rand_within_region(&self, rng: &mut impl Rng) -> ChunkPos {
        let local_x = rng.random_range(0..REGION_SIZE_IN_CHUNKS.x());
        let local_y = rng.random_range(0..REGION_SIZE_IN_CHUNKS.y());
        ChunkPos(self.0 * REGION_SIZE_IN_CHUNKS.0 + IVec2::new(local_x, local_y))
    }
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



#[derive(Component, Debug, Deserialize, Serialize, Copy, Clone, PartialEq, Eq, Hash)]
pub struct SizeInTiles(pub UVec2);
impl SizeInTiles{
    pub fn new(str_id: &StrId, size_in_tiles: Option<(u32, u32)>, ) -> Self {
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
    pub fn tiles_per_chunk(&self) -> Result<UVec2, ()> {
        let raw = ChunkPos::CHUNK_SIZE / self.0;
        if raw.x == 0 || raw.y == 0 {
            return Err(());
        }
        Ok(raw)
    }
    pub fn tilemap_size(&self) -> bevy_ecs_tilemap::map::TilemapSize {
        bevy_ecs_tilemap::map::TilemapSize::from(self.tiles_per_chunk().unwrap_or(UVec2::ONE))
    }
    pub fn render_chunk_size(&self) -> UVec2 {
        self.tiles_per_chunk().unwrap_or(UVec2::ONE) * 2
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
