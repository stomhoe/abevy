
#[allow(unused_imports)] use bevy::prelude::*;
use bevy_replicon::prelude::*;
use fastnoise_lite::FastNoiseLite;

use serde::{Deserialize, Serialize};
use std::hash::{Hasher, Hash};
use std::collections::hash_map::DefaultHasher;

use crate::{common::common_components::EntityPrefix, game::tilemap::{terrain_gen::terrgen_resources::WorldGenSettings, tile::tile_components::GlobalTilePos}, AppState};
use serde::ser::SerializeStruct;


#[derive(Component, Default)]
#[require(EntityPrefix::new("Noise"), Replicated, StateScoped::<AppState>, )]
pub struct TgenNoise { pub noise: FastNoiseLite, pub offset: IVec2, }

impl TgenNoise {
    pub fn new(mut noise: FastNoiseLite) -> Self {
        // Hash the noise and offset to generate a seed
        let mut hasher = DefaultHasher::new();
        noise.seed.hash(&mut hasher);
        noise.frequency.to_bits().hash(&mut hasher);
        (noise.noise_type as u8).hash(&mut hasher);
        (noise.rotation_type_3d as u8).hash(&mut hasher);
        (noise.fractal_type as u8).hash(&mut hasher);
        noise.octaves.hash(&mut hasher);
        noise.lacunarity.to_bits().hash(&mut hasher);
        noise.gain.to_bits().hash(&mut hasher);
        noise.weighted_strength.to_bits().hash(&mut hasher);
        noise.ping_pong_strength.to_bits().hash(&mut hasher);
        (noise.cellular_distance_function as u8).hash(&mut hasher);
        (noise.cellular_return_type as u8).hash(&mut hasher);
        noise.cellular_jitter_modifier.to_bits().hash(&mut hasher);
        (noise.domain_warp_type as u8).hash(&mut hasher);
        noise.domain_warp_amp.to_bits().hash(&mut hasher);

        noise.seed = (hasher.finish() & 0xFFFF_FFFF) as i32;

        Self { noise, offset: IVec2::ZERO }
    }

    pub fn set_offset(&mut self, offset: IVec2) { self.offset = offset; }

    pub fn get_val(&self, tile_pos: GlobalTilePos) -> f32 {
        (self.noise.get_noise_2d(
            (tile_pos.0.x + self.offset.x) as f32,
            (tile_pos.0.y + self.offset.y) as f32
        ) + 1.0) / 2.0 // Normalizar a [0, 1]
    }
}

impl Serialize for TgenNoise {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where S: serde::Serializer {
        let mut state = serializer.serialize_struct("TgenNoise", 2)?;
        // Manually serialize all public fields of FastNoiseLite
        let noise = &self.noise;
        state.serialize_field("noise_seed", &noise.seed)?;
        state.serialize_field("noise_frequency", &noise.frequency)?;
        state.serialize_field("noise_type", &(noise.noise_type as u8))?;
        state.serialize_field("rotation_type_3d", &(noise.rotation_type_3d as u8))?;
        state.serialize_field("fractal_type", &(noise.fractal_type as u8))?;
        state.serialize_field("octaves", &(noise.octaves as u8))?;
        state.serialize_field("lacunarity", &noise.lacunarity)?;
        state.serialize_field("gain", &noise.gain)?;
        state.serialize_field("weighted_strength", &noise.weighted_strength)?;
        state.serialize_field("ping_pong_strength", &noise.ping_pong_strength)?;
        state.serialize_field("cellular_distance_function", &(noise.cellular_distance_function as u8))?;
        state.serialize_field("cellular_return_type", &(noise.cellular_return_type as u8))?;
        state.serialize_field("cellular_jitter_modifier", &noise.cellular_jitter_modifier)?;
        state.serialize_field("domain_warp_type", &(noise.domain_warp_type as u8))?;
        state.serialize_field("domain_warp_amp", &noise.domain_warp_amp)?;
        state.serialize_field("offset", &self.offset)?;
        state.end()
    }
}
impl<'de> Deserialize<'de> for TgenNoise {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: serde::Deserializer<'de> {
        #[derive(Deserialize)]
        struct TgenNoiseHelper {
            noise_seed: i32,
            noise_frequency: f32,
            noise_type: u8,
            rotation_type_3d: u8,
            fractal_type: u8,
            octaves: u8,
            lacunarity: f32,
            gain: f32,
            weighted_strength: f32,
            ping_pong_strength: f32,
            cellular_distance_function: u8,
            cellular_return_type: u8,
            cellular_jitter_modifier: f32,
            domain_warp_type: u8,
            domain_warp_amp: f32,
            offset: IVec2,
        }
        let helper = TgenNoiseHelper::deserialize(deserializer)?;

        let mut noise = FastNoiseLite::default();
        noise.seed = helper.noise_seed;
        noise.frequency = helper.noise_frequency;
        noise.noise_type = unsafe { std::mem::transmute(helper.noise_type) };
        noise.rotation_type_3d = unsafe { std::mem::transmute(helper.rotation_type_3d) };
        noise.fractal_type = unsafe { std::mem::transmute(helper.fractal_type) };
        noise.octaves = helper.octaves as i32;
        noise.lacunarity = helper.lacunarity;
        noise.gain = helper.gain;
        noise.weighted_strength = helper.weighted_strength;
        noise.ping_pong_strength = helper.ping_pong_strength;
        noise.cellular_distance_function = unsafe { std::mem::transmute(helper.cellular_distance_function) };
        noise.cellular_return_type = unsafe { std::mem::transmute(helper.cellular_return_type) };
        noise.cellular_jitter_modifier = helper.cellular_jitter_modifier;
        noise.domain_warp_type = unsafe { std::mem::transmute(helper.domain_warp_type) };
        noise.domain_warp_amp = helper.domain_warp_amp;

        Ok(TgenNoise { noise, offset: helper.offset, })
    }
}


#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, )]
pub struct PoissonDisk { pub min_distance: u8, pub seed: u64, }
impl PoissonDisk {
    pub fn new(min_distance: u8, seed: u64) -> Self { Self { min_distance, seed } }
    pub fn sample(&self, settings: &WorldGenSettings, tile_pos: GlobalTilePos) -> f32 {

        let val = tile_pos.normalized_hash_value(settings, self.seed);
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
                let neighbor_val = neighbor_pos.normalized_hash_value(settings, self.seed);
                if neighbor_val > val {
                    return 0.0;
                }
            }
        }
        val 
    }
}
impl Default for PoissonDisk { fn default() -> Self { Self { min_distance: 1, seed: 0 } } }



#[derive(Component, Debug, Deserialize, Serialize, Clone, Copy)]
pub struct ChunkRef(pub Entity);


#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, )]
pub enum Operation {
    Add, Subtract, Multiply, Divide, Modulo, Log, Min, Max, Pow, Assign, Mean,
}

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, Copy)]
pub struct InputOperand(pub f32);

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, Copy)]
pub struct RootOpList;


#[derive(Component, Debug, Default, Deserialize, Serialize, Clone)]
#[require(Replicated, StateScoped::<AppState>, EntityPrefix::new("OpList"), )]
pub struct OperationList {
    pub trunk: Vec<(Operand, Operation)>, pub threshold: f32,
    #[entities] pub bifurcation_over: Option<Entity>, 
    #[entities] pub bifurcation_under: Option<Entity>,
}


#[derive(Component, Debug, Deserialize, Serialize, Clone, PartialEq, )]
pub enum Operand {
    Value(f32),
    Entities(#[entities] Vec<Entity>),
    HashPos,
    PoissonDisk(PoissonDisk),
}
impl Operand {
    pub fn new_poisson_disk(min_distance: u8, seed: u64) -> Self {
        Self::PoissonDisk(PoissonDisk::new(min_distance, seed))
    }
}
impl Default for Operand { fn default() -> Self { Self::Value(0.0) } }
impl From<Entity> for Operand { fn from(e: Entity) -> Self { Self::Entities(vec![e]) } }
impl From<f32> for Operand { fn from(v: f32) -> Self { Self::Value(v) } }


#[derive(Component, Debug, Clone, Copy, )]
#[require(InputOperand)]
pub struct OplistRef(pub Entity);
