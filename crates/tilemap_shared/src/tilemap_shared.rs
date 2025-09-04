use std::{hash::{DefaultHasher, Hash, Hasher}, ops::Add};

use bevy::math::U8Vec4;
use bevy::platform::collections::HashMap;
#[allow(unused_imports)] use bevy::prelude::*;
use bevy_ecs_tilemap::tiles::TilePos;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Resource, Debug, Reflect)]
#[reflect(Resource, Default)]
pub struct AaGlobalGenSettings {
    
    pub seed: i32,
    pub world_freq: f32,
}

impl Default for AaGlobalGenSettings {
    fn default() -> Self {
        Self { 
            seed: 0,
            world_freq: 1e-1,
        }
    }
}

pub trait HashablePosVec: Hash {
    fn hash_value(&self, settings: &AaGlobalGenSettings, seed: u64) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        settings.seed.hash(&mut hasher);
        seed.hash(&mut hasher);
        hasher.finish()
    }
    fn hash_true_false(&self, settings: &AaGlobalGenSettings, extra_seed: u64) -> bool {
        self.hash_value(settings, extra_seed) % 2 == 0
    }
    fn hash_for_weight_maps(&self, settings: &AaGlobalGenSettings) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        settings.seed.hash(&mut hasher);
        49.hash(&mut hasher);
        hasher.finish()
    }
    fn normalized_hash_value(&self, settings: &AaGlobalGenSettings, seed: u64) -> f32 {
        self.hash_value(settings, seed) as f32 / u64::MAX as f32
    }

    fn x(&self) -> i32;
    fn y(&self) -> i32;
}

// Macro to implement HashedPosition for types that derive Hash and have x() and y()
macro_rules! impl_hashed_position {
    ($t:ty) => {
        impl HashablePosVec for $t {
            fn x(&self) -> i32 { self.x() }
            fn y(&self) -> i32 { self.y() }
        }
    };
}

#[derive(Component, Clone, Deserialize, Serialize, Default, Hash, PartialEq, Eq, Copy, Reflect, )]
pub struct GlobalTilePos(pub IVec2);

impl_hashed_position!(GlobalTilePos);

impl GlobalTilePos {
    pub fn new(x: i32, y: i32) -> Self {GlobalTilePos(IVec2::new(x, y))}

    pub fn get_pos_within_chunk(self, chunk_pos: ChunkPos, oplist_size: UVec2) -> TilePos {
        let pos_within_chunk = (self.0 - chunk_pos.to_tilepos().0) / oplist_size.as_ivec2();
        TilePos::from(pos_within_chunk.as_uvec2())
    }
    pub fn x(&self) -> i32 { self.0.x } pub fn y(&self) -> i32 { self.0.y }

    pub fn distance(&self, other: &GlobalTilePos) -> f32 {
        (self.0.distance_squared(other.0) as f32).sqrt()
    }
    pub fn distance_squared(&self, other: &GlobalTilePos) -> u32 {
        self.0.distance_squared(other.0) as u32
    }

    pub const TYPE_NAME: &'static str = "Gpos";
    pub const TILE_SIZE_PXS: UVec2 = UVec2 { x: 64, y: 64 };

    pub fn to_tilepos(&self, oplist_size: OplistSize) -> TilePos {
        let chunk_size = ChunkPos::CHUNK_SIZE.as_ivec2();
        let ivec2 = (((Into::<IVec2>::into(*self) % chunk_size) + chunk_size) % chunk_size) / oplist_size.inner().as_ivec2();
        TilePos::from(ivec2.as_uvec2())
    }
}
impl Add<IVec2> for GlobalTilePos {
    type Output = Self;
    fn add(self, rhs: IVec2) -> Self::Output {
        GlobalTilePos(self.0 + rhs)
    }
}

impl std::fmt::Display for GlobalTilePos {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}({}, {})", Self::TYPE_NAME, self.0.x, self.0.y)
    }
}
impl std::fmt::Debug for GlobalTilePos {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}({}, {})", Self::TYPE_NAME, self.0.x, self.0.y)
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
impl Into<IVec2> for GlobalTilePos {
    fn into(self) -> IVec2 {
        self.0
    }
}


impl From<IVec2> for GlobalTilePos {
    fn from(ivec2: IVec2) -> Self {
        GlobalTilePos(ivec2)
    }
}
impl std::ops::Add for GlobalTilePos {type Output = Self; fn add(self, other: Self) -> Self {GlobalTilePos(self.0 + other.0)}}
impl std::ops::Sub for GlobalTilePos {type Output = Self; fn sub(self, other: Self) -> Self {GlobalTilePos(self.0 - other.0)}}

#[derive(Component, Default, Clone, Deserialize, Serialize, Copy, Hash, PartialEq, Eq, Reflect)]
pub struct ChunkPos(pub IVec2);
impl_hashed_position!(ChunkPos);

impl ChunkPos {
    pub fn new(x: i32, y: i32) -> Self { Self(IVec2::new(x, y)) }
    pub fn x(&self) -> i32 { self.0.x }
    pub fn y(&self) -> i32 { self.0.y }
    pub const CHUNK_SIZE: UVec2 = UVec2 { x: 12, y: 12 };//NORMALMENTE 12X12
    pub fn hash_value(&self, settings: &AaGlobalGenSettings, seed: u64) -> u64 {
        let mut hasher = DefaultHasher::new();
        // Mix coordinates with a unique constant and the seed
        self.hash(&mut hasher);
        settings.seed.hash(&mut hasher);
        seed.hash(&mut hasher);
        hasher.finish()
    }
    pub fn normalized_hash_value(&self, settings: &AaGlobalGenSettings, seed: u64) -> f32 {
        self.hash_value(settings, seed) as f32 / u64::MAX as f32
    }
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
        self.0.as_vec2() * GlobalTilePos::TILE_SIZE_PXS.as_vec2() * Self::CHUNK_SIZE.as_vec2()
    }
    pub fn to_tilepos(&self) -> GlobalTilePos {
        GlobalTilePos(self.0 * Self::CHUNK_SIZE.as_ivec2())
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

impl std::ops::Add for ChunkPos {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output { ChunkPos(self.0 + rhs.0) }
}
impl std::ops::Sub for ChunkPos {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output { ChunkPos(self.0 - rhs.0) }
}



#[derive(Component, Debug, Deserialize, Serialize, Clone, Copy, Hash, PartialEq, Eq, Reflect)]
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
impl Default for OplistSize { fn default() -> Self { Self(UVec2::ONE) } }