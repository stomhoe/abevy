
use bevy::{math::{U8Vec2, U8Vec4}, platform::collections::HashMap};
#[allow(unused_imports)] use bevy::prelude::*;
use bevy_ecs_tilemap::tiles::{TileColor, TileFlip, TileTextureIndex, TileVisible};
use fastnoise_lite::FastNoiseLite;

use superstate::{SuperstateInfo};
use serde::{Deserialize, Serialize};
use std::hash::{Hasher, Hash};
use std::collections::hash_map::DefaultHasher;

use crate::{common::common_components::EntityPrefix, game::{game_utils::WeightedMap, tilemap::{chunking_components::ProducedTiles, terrain_gen::terrgen_resources::WorldGenSettings, tile::tile_components::GlobalTilePos}, }};

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, Hash, PartialEq, Eq)]
pub struct TileId(u64);

impl TileId {
    pub fn new<S: AsRef<str>>(id: S) -> Self {
        let mut hasher = DefaultHasher::new();
        id.as_ref().hash(&mut hasher);
        Self(hasher.finish())
    }

    pub fn id(&self) -> u64 {
        self.0
    }
}

#[derive(Component, Default)]
#[require(EntityPrefix::new("Noise"))]
pub struct TgenNoise {noise: FastNoiseLite, offset: IVec2,}
impl TgenNoise {
    pub fn new(noise: FastNoiseLite,) -> Self {
        Self::new_with_offset(noise, IVec2::ZERO) 
    }

    pub fn new_with_offset(mut noise: FastNoiseLite, offset: IVec2, ) -> Self {
        // Hash the noise and offset to generate a seed
        let mut hasher = DefaultHasher::new();
        noise.seed.hash(&mut hasher);
        noise.frequency.to_bits().hash(&mut hasher);
        (noise.noise_type as i32).hash(&mut hasher);
        (noise.rotation_type_3d as i32).hash(&mut hasher);
        (noise.fractal_type as i32).hash(&mut hasher);
        noise.octaves.hash(&mut hasher);
        noise.lacunarity.to_bits().hash(&mut hasher);
        noise.gain.to_bits().hash(&mut hasher);
        noise.weighted_strength.to_bits().hash(&mut hasher);
        noise.ping_pong_strength.to_bits().hash(&mut hasher);
        (noise.cellular_distance_function as i32).hash(&mut hasher);
        (noise.cellular_return_type as i32).hash(&mut hasher);
        noise.cellular_jitter_modifier.to_bits().hash(&mut hasher);
        (noise.domain_warp_type as i32).hash(&mut hasher);
        noise.domain_warp_amp.to_bits().hash(&mut hasher);
        offset.hash(&mut hasher);

        noise.seed = (hasher.finish() & 0xFFFF_FFFF) as i32 ;

        Self { noise, offset }
    }
    pub fn get_val(&self, tile_pos: GlobalTilePos) -> f32 {
        (self.noise.get_noise_2d(
            (tile_pos.0.x + self.offset.x) as f32, 
            (tile_pos.0.y + self.offset.y) as f32
        ) + 1.0) / 2.0 // Normalizar a [0, 1]
    }
}

impl std::hash::Hash for TgenNoise {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        // Hash FastNoiseLite fields that are public and relevant
        self.noise.seed.hash(state);
        self.noise.frequency.to_bits().hash(state);
        (self.noise.noise_type as i32).hash(state);
        (self.noise.rotation_type_3d as i32).hash(state);

        (self.noise.fractal_type as i32).hash(state);
        self.noise.octaves.hash(state);
        self.noise.lacunarity.to_bits().hash(state);
        self.noise.gain.to_bits().hash(state);
        self.noise.weighted_strength.to_bits().hash(state);
        self.noise.ping_pong_strength.to_bits().hash(state);

        (self.noise.cellular_distance_function as i32).hash(state);
        (self.noise.cellular_return_type as i32).hash(state);
        self.noise.cellular_jitter_modifier.to_bits().hash(state);

        (self.noise.domain_warp_type as i32).hash(state);
        self.noise.domain_warp_amp.to_bits().hash(state);

        // Hash the offset
        self.offset.hash(state);
    }
}

#[derive(Component, Debug, Deserialize, Serialize, Clone, )]
#[require(EntityPrefix::new("HashPosComp"))]
pub struct TgenHashPos;
impl TgenHashPos {
    pub fn get_val(&self, tile_pos: GlobalTilePos, settings: &WorldGenSettings) -> f32 {
       let mut hasher = std::collections::hash_map::DefaultHasher::new();
        tile_pos.0.hash(&mut hasher);
        settings.seed.hash(&mut hasher);
        let hash_val = hasher.finish();
        (hash_val as f64 / u64::MAX as f64) as f32
    }
}

#[derive(Component, Debug, Deserialize, Serialize, Clone, Copy)]
pub struct ChunkRef(pub Entity);


#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Default, )]
pub enum NextAction {
    #[default]
    Continue,
    Break,
    OverwriteAcc(f32),
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, )]
pub struct OnCompareConfig {
    pub tiles_on_success: ProducedTiles,
    pub tiles_on_failure: ProducedTiles,
    pub on_success: NextAction,
    pub on_failure: NextAction,
}
impl Default for OnCompareConfig {
    fn default() -> Self {
        Self {
            tiles_on_success: ProducedTiles(vec![]),
            tiles_on_failure: ProducedTiles(vec![]),
            on_success: NextAction::Continue,
            on_failure: NextAction::Break,
        }
    }
}
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq,)]
pub enum Operation {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    Log,
    Min,
    Max,
    Pow,
    Assign,
    GreaterThan(OnCompareConfig),
    LessThan(OnCompareConfig),
    GetTiles(ProducedTiles),
}

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, Copy)]
pub struct InputOperand(pub f32);

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, Copy)]
pub struct RootOpList;

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone)]
pub struct OperationList {
    pub trunk: Vec<(Entity, Operation)>,
    pub threshold: f32,
    pub bifurcation_over: Option<Entity>,
    pub bifurcation_under: Option<Entity>,
}

#[derive(Component, Debug, Deserialize, Serialize, Clone, Copy, )]
#[require(InputOperand)]
pub struct OplistRef(pub Entity);
