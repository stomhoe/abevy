use std::{hash::{DefaultHasher, Hash, Hasher}, ops::Add};

#[allow(unused_imports)] use bevy::prelude::*;
use bevy_ecs_tilemap::tiles::TilePos;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
use rand::Rng;
use serde::{Deserialize, Serialize};

#[derive(Component, Debug, Reflect, Deserialize, Serialize, Clone, )]
#[require(Replicated, )]
pub struct AcGlobalGenSettings {//TODO cambiar esto por un entity para synquear seed
    
    pub seed: i32,
    pub world_freq: f32,
}

impl Default for AcGlobalGenSettings {
    fn default() -> Self {
        Self { 
            seed: 0,
            world_freq: 1e-1,
        }
    }
}
macro_rules! impl_position_conversions {
    ($t:ty) => {
        impl Into<IVec2> for $t {
            fn into(self) -> IVec2 {
                self.0
            }
        }
        impl From<IVec2> for $t {
            fn from(ivec2: IVec2) -> Self {
                Self(ivec2)
            }
        }
    };
}
macro_rules! impl_position_ops {
    ($t:ty) => {
        impl std::ops::Add for $t {
            type Output = Self;
            fn add(self, other: Self) -> Self {
                Self(self.0 + other.0)
            }
        }
        impl std::ops::Sub for $t {
            type Output = Self;
            fn sub(self, other: Self) -> Self {
                Self(self.0 - other.0)
            }
        }
        impl std::ops::Add<IVec2> for $t {
            type Output = Self;
            fn add(self, other: IVec2) -> Self {
                Self(self.0 + other)
            }
        }
    };
}
macro_rules! impl_display_debug {
    ($t:ty, $name:expr) => {
        impl std::fmt::Display for $t {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}({}, {})", $name, self.0.x, self.0.y)
            }
        }
        impl std::fmt::Debug for $t {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}({}, {})", $name, self.0.x, self.0.y)
            }
        }
    };
}

pub trait HashablePosVec: Hash {
    fn hash_value(&self, settings: &AcGlobalGenSettings, seed: u64) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        settings.seed.hash(&mut hasher);
        seed.hash(&mut hasher);
        hasher.finish()
    }
    fn hash_true_false(&self, settings: &AcGlobalGenSettings, extra_seed: u64) -> bool {
        self.hash_value(settings, extra_seed) % 2 == 0
    }
    fn hash_for_weight_maps(&self, settings: &AcGlobalGenSettings) -> u64 {
        self.hash_value(settings, 53)
    }
    fn normalized_hash_value(&self, settings: &AcGlobalGenSettings, seed: u64) -> f64 {
        self.hash_value(settings, seed) as f64 / u64::MAX as f64
    }

    fn x(&self) -> i32;
    fn y(&self) -> i32;
}
macro_rules! impl_hashed_position {
    ($t:ty) => {
        impl HashablePosVec for $t {
            fn x(&self) -> i32 { self.0.x }
            fn y(&self) -> i32 { self.0.y }
        }
    };
}

#[derive(Component, Clone, Deserialize, Serialize, Default, Hash, PartialEq, Eq, Copy, Reflect, )]
pub struct GlobalTilePos(pub IVec2);

#[derive(Component, Clone, Deserialize, Serialize, Default, Hash, PartialEq, Eq, Copy, Reflect, Debug)]
pub struct PrevGlobalTilePos(pub GlobalTilePos);

impl_hashed_position!(GlobalTilePos);
impl_position_conversions!(GlobalTilePos);
impl_position_ops!(GlobalTilePos);

impl GlobalTilePos {
    pub fn new(x: i32, y: i32) -> Self {GlobalTilePos(IVec2::new(x, y))}

    pub fn distance(&self, other: &GlobalTilePos) -> f32 {
        (self.0.distance_squared(other.0) as f32).sqrt()
    }
    pub fn distance_squared(&self, other: &GlobalTilePos) -> u32 {
        self.0.distance_squared(other.0) as u32
    }
    pub const TILE_SIZE_PXS: UVec2 = UVec2 { x: 64, y: 64 };

    pub fn to_tilepos(&self, oplist_size: OplistSize) -> TilePos {
        let chunk_size = ChunkPos::CHUNK_SIZE.as_ivec2();
        let ivec2 = (((Into::<IVec2>::into(*self) % chunk_size) + chunk_size) % chunk_size) / oplist_size.inner().as_ivec2();
        TilePos::from(ivec2.as_uvec2())
    }
    pub fn to_chunkpos(&self) -> ChunkPos {
        ChunkPos(Into::<IVec2>::into(*self) / ChunkPos::CHUNK_SIZE.as_ivec2())
    }

    pub fn to_translation(&self, prev_transform_z: f32) -> Vec3 {
        let vec2: Vec2 = (*self).into();
        vec2.extend(prev_transform_z)
    }
    
}
impl_display_debug!(GlobalTilePos, "Gpos");

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
impl Into<Vec2> for GlobalTilePos {
    fn into(self) -> Vec2 {
        self.0.as_vec2() * GlobalTilePos::TILE_SIZE_PXS.as_vec2()
    }
}

#[derive(Component, Default, Clone, Deserialize, Serialize, Copy, Hash, PartialEq, Eq, Reflect)]
pub struct ChunkPos(pub IVec2);
impl_hashed_position!(ChunkPos);
impl_display_debug!(ChunkPos, "Cpos");
impl_position_ops!(ChunkPos);
impl_position_conversions!(ChunkPos);

impl ChunkPos {
    pub const fn new(x: i32, y: i32) -> Self { Self(IVec2::new(x, y)) }
    pub fn rand_within_region(region_pos: RegionPos, rng: &mut impl Rng) -> Self {
        let local_x = rng.random_range(0..REGION_SIZE_IN_CHUNKS.x());
        let local_y = rng.random_range(0..REGION_SIZE_IN_CHUNKS.y());
        Self(region_pos.0 * REGION_SIZE_IN_CHUNKS.0 + IVec2::new(local_x, local_y))
    }
    pub fn x(&self) -> i32 { self.0.x }
    pub fn y(&self) -> i32 { self.0.y }
    pub const CHUNK_SIZE: UVec2 = UVec2 { x: 12, y: 12 };//NORMALMENTE 12X12. 60x60?

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

    pub const fn area(&self) -> usize {
        (self.0.x * self.0.y) as usize
    }
    pub fn get_tilepositions_within_chunk(&self, oplist_size: OplistSize) -> Vec<GlobalTilePos> {
        let mut tiles = Vec::with_capacity((Self::CHUNK_SIZE.x / oplist_size.x() * Self::CHUNK_SIZE.y / oplist_size.y()) as usize);
        let chunk_origin = self.to_tilepos();
        for y in (0..Self::CHUNK_SIZE.y).step_by(oplist_size.y() as usize) {
            for x in (0..Self::CHUNK_SIZE.x).step_by(oplist_size.x() as usize) {
                let tile_pos = GlobalTilePos(IVec2::new(chunk_origin.0.x + x as i32, chunk_origin.0.y + y as i32));
                tiles.push(tile_pos);
            }
        }
        tiles
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



#[derive(Component, Debug, Deserialize, Serialize, Clone, Copy, Hash, PartialEq, Eq, Reflect)]
/// IMPORTANTE: va asociado a cada tile instance, no a la tile original
pub struct OplistSize(UVec2);

impl OplistSize {
    pub fn new([x, y]: [u32; 2]) -> Result<Self, BevyError> {
        if x <= 0 || y <= 0 {
            return Err(BevyError::from("OplistSize dimensions must be > 0"));
        }
        let max = 4;
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
impl Default for OplistSize { fn default() -> Self { Self(UVec2::ONE) } }


pub const REGION_SIZE_IN_CHUNKS: ChunkPos = ChunkPos::new(32, 32);

#[derive(Component, Clone, Deserialize, Serialize, Default, Hash, PartialEq, Eq, Copy, Reflect, )]
pub struct RegionPos(pub IVec2);
impl_hashed_position!(RegionPos);
impl_position_ops!(RegionPos);
impl_position_conversions!(RegionPos);
impl_display_debug!(RegionPos, "Rpos");

impl RegionPos {
    pub fn new(x: i32, y: i32) -> Self { Self(IVec2::new(x, y)) }
    pub fn x(&self) -> i32 { self.0.x }
    pub fn y(&self) -> i32 { self.0.y }

    pub fn chunk_bounds(&self) -> (ChunkPos, ChunkPos) {
        let min = ChunkPos(self.0 * REGION_SIZE_IN_CHUNKS.0);
        let max = ChunkPos((self.0 + IVec2::ONE) * REGION_SIZE_IN_CHUNKS.0);
        (min, max)
    }
    pub fn contains_chunkpos(&self, cp: ChunkPos) -> bool {
        let (min, max) = self.chunk_bounds();
        cp.x() >= min.x() && cp.y() >= min.y() && cp.x() < max.x() && cp.y() < max.y()
    }
    pub fn to_chunkpos(&self) -> ChunkPos {
        ChunkPos(self.0 * REGION_SIZE_IN_CHUNKS.0)
    }

}

type PDiskDistType = i64;
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Reflect, Component, )]
pub struct PoissonDisk { pub mindists_seeds: Vec<(PDiskDistType, u64)>, }
impl PoissonDisk {
    pub fn new(min_distance: u8, seed: u64) -> Result<Self, BevyError> {

        let max = 5;

        if min_distance > max {
            return Err(BevyError::from(format!("min_distance must be <= {}", max)));
        } else if min_distance == 0 {
            return Err(BevyError::from("min_distance must be > 0"));
        }
        Ok(Self { mindists_seeds: vec![(min_distance as PDiskDistType, seed)] })
    }
    pub fn multiple_tagged(mindists_tag: Vec<(Option<u8>, String)>, fallback_mindist: u8, max: u8) -> Result<Self, BevyError> {
        let mut mindists_seeds: Vec<(PDiskDistType, u64)> = Vec::with_capacity(mindists_tag.len());
        for (min_distance, tag) in mindists_tag.iter() {
            let min_distance = min_distance.unwrap_or(fallback_mindist);

            if min_distance > max {
                return Err(BevyError::from(format!("min_distance must be <= {}", max)));
            } else if min_distance == 0 {
                return Err(BevyError::from("min_distance must be > 0"));
            }
            let mut hasher = DefaultHasher::new();
            tag.hash(&mut hasher);
            let seed = hasher.finish();
            mindists_seeds.push((min_distance as PDiskDistType, seed));
        }
        Ok(Self { mindists_seeds })
    }
    pub fn is_allowed_position<T: HashablePosVec>(&self, settings: &AcGlobalGenSettings, pos: T, check_within_radius: bool, oplist_size: OplistSize) -> bool {
        self.sample(settings, pos, check_within_radius, oplist_size) > 0.0
    }

    pub fn sample<T: HashablePosVec>(&self, settings: &AcGlobalGenSettings, pos: T, 
        check_within_circle: bool, oplist_size: OplistSize, ) -> f64 {

        let mut sum = 0.0;
        for &(min_distance, seed) in self.mindists_seeds.iter() {
            let val = pos.normalized_hash_value(settings, seed);
            sum += val;
            let added_sample_distance_x = oplist_size.x() as i32 - 1;
            let added_sample_distance_y = oplist_size.y() as i32 - 1;
    
            for dy in -(min_distance as i32)..=(min_distance as i32) {
                for dx in -(min_distance as i32)..=(min_distance as i32) {
                    if dx == 0 && dy == 0 {
                        continue;
                    }
                    // Only check within circle of radius min_distance
                    if check_within_circle && dx * dx + dy * dy > (min_distance as i32).pow(2) {
                        continue;
                    }
                    // Calculate the neighbor's position by offsetting the current tile position
                    let neighbor_x = pos.x() + dx + added_sample_distance_x;
                    let neighbor_y = pos.y() + dy + added_sample_distance_y;
                    let neighbor_pos = GlobalTilePos(IVec2::new(neighbor_x, neighbor_y));
                    let neighbor_val = neighbor_pos.normalized_hash_value(settings, seed);
                    if neighbor_val > val {
                        return 0.0;
                    }
                }
            }
            
        }
        sum / (self.mindists_seeds.len() as f64)
    }
}
impl Default for PoissonDisk { fn default() -> Self { Self { mindists_seeds: vec![(1, 0)] } } }