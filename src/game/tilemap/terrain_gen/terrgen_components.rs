
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


#[derive(Component, Debug, Deserialize, Serialize, Clone, PartialEq, )]
pub struct PoissonDisk {
    pub min_distance: u8,
}
impl PoissonDisk {
    pub fn sample(&self, settings: &WorldGenSettings, tile_pos: GlobalTilePos) -> f32 {
        let val = tile_pos.normalized_hash_value(settings);
        // Check neighbors within min_distance — reject if any has lower hash (pseudo-distance enforcement)
        for dy in -(self.min_distance as i32)..=(self.min_distance as i32) {
            for dx in -(self.min_distance as i32)..=(self.min_distance as i32) {
                if dx == 0 && dy == 0 {
                    continue;
                }
                // Only check within circle of radius min_distance
                if dx * dx + dy * dy > (self.min_distance as i32).pow(2) {
                    continue;
                }
                let neighbor_pos = GlobalTilePos(IVec2::new(tile_pos.0.x + dx, tile_pos.0.y + dy));
                let neighbor_val = neighbor_pos.normalized_hash_value(settings);
                if neighbor_val > val {
                    return 0.0;
                }
            }
        }
        val 
    }
}
impl Default for PoissonDisk {fn default() -> Self { Self { min_distance: 1 } } }



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
            tiles_on_success: ProducedTiles::default(),
            tiles_on_failure: ProducedTiles::default(),
            on_success: NextAction::Continue,
            on_failure: NextAction::Break,
        }
    }
}
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq,)]
pub enum Operation {
    Add, Subtract, Multiply, Divide, Modulo, Log, Min, Max, Pow, Assign,
    GreaterThan(OnCompareConfig),
    LessThan(OnCompareConfig),
    GetTiles(ProducedTiles),
}

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, Copy)]
pub struct InputOperand(pub f32);

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, Copy)]
pub struct RootOpList;

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Default)]
pub enum Operand {
    Entity(Entity),
    Value(f32),
    HashPos,
    PoissonDisk(PoissonDisk),
    #[default] Zero,
}
impl From<Entity> for Operand { fn from(e: Entity) -> Self { Self::Entity(e) } }
impl From<f32> for Operand { fn from(v: f32) -> Self { Self::Value(v) } }

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone)]
pub struct OperationList {
    pub trunk: Vec<(Operand, Operation)>, pub threshold: f32,
    pub bifurcation_over: Option<Entity>, pub bifurcation_under: Option<Entity>,
}

#[derive(Component, Debug, Deserialize, Serialize, Clone, Copy, )]
#[require(InputOperand)]
pub struct OplistRef(pub Entity);
