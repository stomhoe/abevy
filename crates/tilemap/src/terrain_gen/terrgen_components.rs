
use bevy::ecs::entity::MapEntities;
#[allow(unused_imports)] use bevy::prelude::*;
use bevy_replicon::prelude::*;
use fastnoise_lite::FastNoiseLite;

use noiz::DynamicConfigurableSampleable;
use serde::{Deserialize, Serialize};
use std::hash::{Hasher, Hash};
use std::collections::hash_map::DefaultHasher;

use crate::terrain_gen::terrgen_resources::GlobalGenSettings;
use crate::tile::tile_components::*;

use {common::common_components::*, };
use strum_macros::{AsRefStr, Display, };
use std::ops::{Index, IndexMut};

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, Hash, PartialEq, Reflect)]
#[require(Replicated, SessionScoped, AssetScoped, TgenScoped, )]
pub struct TerrGen;

#[derive(Component, Default, Reflect, Serialize, Deserialize, )]
#[require(TerrGen, EntityPrefix::new("Noise"), )]
pub struct FnlNoise { pub noise: FastNoiseLite, pub offset: IVec2, innate_freq: f32 }

impl FnlNoise {
    pub const FREQUENCY_MIN: f32 = 1e-10;

    pub fn new(mut noise: FastNoiseLite, id: StrId) -> Self {
        // Hash the noise and offset to generate a seed
        let mut hasher = DefaultHasher::new();
        id.hash(&mut hasher);
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

        let innate_freq = noise.frequency * 
        match noise.noise_type {
            fastnoise_lite::NoiseType::Cellular => 0.9,
            fastnoise_lite::NoiseType::Perlin => 2.0,
            fastnoise_lite::NoiseType::ValueCubic => 2.6,
            fastnoise_lite::NoiseType::Value => 2.7,
            _ => 1.,
        };

        Self {innate_freq, noise, offset: IVec2::ZERO, }
    }
    pub fn adjust2world_settings(&mut self, settings: &GlobalGenSettings){
        self.noise.seed += settings.seed;
        if self.innate_freq * settings.world_freq < FnlNoise::FREQUENCY_MIN {
            warn!("World scale is too small (< {})", FnlNoise::FREQUENCY_MIN);
        }

        self.noise.frequency = self.innate_freq * settings.world_freq;
    }
    pub fn set_offset(&mut self, offset: IVec2) { self.offset = offset; }

    pub fn sample(&self, tile_pos: GlobalTilePos, settings: u64) -> f32 {
        match settings {
            0 => self.get_val_0_1(tile_pos),
            1 => self.get_opo_val(tile_pos),
            2 => self.get_val_neg1_1(tile_pos),
            _ => {
                error!("Unknown noise settings: {}", settings);
                0.0
            }
        }
    }

    fn get_val_0_1(&self, tile_pos: GlobalTilePos) -> f32 {
        (self.noise.get_noise_2d(
            (tile_pos.0.x + self.offset.x) as f32,
            (tile_pos.0.y + self.offset.y) as f32
        ) + 1.0) / 2.0 // Normalizar a [0, 1]
    }
    fn get_val_neg1_1(&self, tile_pos: GlobalTilePos) -> f32 {
        self.noise.get_noise_2d(
            (tile_pos.0.x + self.offset.x) as f32,
            (tile_pos.0.y + self.offset.y) as f32
        )
    }
    fn get_opo_val(&self, tile_pos: GlobalTilePos) -> f32 {
        1. - self.noise.get_noise_2d(
            (tile_pos.0.x + self.offset.x) as f32,
            (tile_pos.0.y + self.offset.y) as f32
        )
    }
}

//            .replicate::<NoizRef>()
#[derive(Component, )]
pub struct Noiz(pub Box<dyn DynamicConfigurableSampleable<Vec2, f32> + Send + Sync >);


#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Reflect, )]
pub struct PoissonDisk { pub min_distance: u8, pub seed: u64, }
impl PoissonDisk {
    pub fn new(min_distance: u8, seed: u64) -> Result<Self, BevyError> { 
        if min_distance > 5 {
            return Err(BevyError::from("min_distance must be <= 5"));
        } else if min_distance == 0 {
            return Err(BevyError::from("min_distance must be > 0"));
        }
        Ok(Self { min_distance, seed }) 
    }
    pub fn sample(&self, settings: &GlobalGenSettings, tile_pos: GlobalTilePos, oplist_size: OplistSize) -> f32 {

        let val = tile_pos.normalized_hash_value(settings, self.seed);
        let added_sample_distance_x = oplist_size.x() as i32;
        let added_sample_distance_y = oplist_size.y() as i32;

        for dy in -(self.min_distance as i32)..=(self.min_distance as i32) {
            for dx in -(self.min_distance as i32)..=(self.min_distance as i32) {
                if dx == 0 && dy == 0 {
                    continue;
                }
                // Only check within circle of radius min_distance
                if dx * dx + dy * dy > (self.min_distance as i32).pow(2) {
                    continue;
                }
                // Calculate the neighbor's position by offsetting the current tile position
                let neighbor_x = tile_pos.0.x + dx + added_sample_distance_x;
                let neighbor_y = tile_pos.0.y + dy + added_sample_distance_y;
                let neighbor_pos = GlobalTilePos(IVec2::new(neighbor_x, neighbor_y));
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



#[derive(Component, Debug, Deserialize, Serialize, Clone, Copy, Reflect)]
pub struct ChunkRef(pub Entity);





#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, Reflect)]
#[require(EntityPrefix::new("OpList"), Replicated, SessionScoped, AssetScoped, TgenScoped)]
pub struct OperationList {

    pub trunk: Vec<(Operation, Vec<Operand>, u8)>,
    pub split: f32,
    pub bifurcation_over: Option<Entity>,
    pub bifurcation_under: Option<Entity>,
    pub tiles_over: Vec<Entity>,
    pub tiles_under: Vec<Entity>,
}

impl MapEntities for OperationList {
    fn map_entities<E: EntityMapper>(&mut self, entity_mapper: &mut E) {
        for (_, operands, _) in self.trunk.iter_mut() {
            for operand in operands.iter_mut() {
                if let Operand::Entity(ent, _) = operand {
                    *ent = entity_mapper.get_mapped(*ent);
                }
            }
        }
        if let Some(entity) = self.bifurcation_over.as_mut() {
            *entity = entity_mapper.get_mapped(*entity);
        }
        if let Some(entity) = self.bifurcation_under.as_mut() {
            *entity = entity_mapper.get_mapped(*entity);
        }
        for entity in self.tiles_over.iter_mut() {
            *entity = entity_mapper.get_mapped(*entity);
        }
        for entity in self.tiles_under.iter_mut() {
            *entity = entity_mapper.get_mapped(*entity);
        }
    }
}

#[derive(Component, Debug, Deserialize, Serialize, Clone, Copy, Hash, PartialEq, Eq, Reflect)]
#[require(EntityPrefix::new("MainComponentName"), )]
pub struct OplistSize(UVec2);

impl OplistSize {
    pub fn new([x, y]: [u32; 2]) -> Result<Self, BevyError> {
        if x == 0 || y == 0 {
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

// #[derive(Component, Debug, Default, Deserialize, Serialize, Clone, Copy)]
// pub struct RootOpList;

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, Copy)]
pub struct VariablesArray(pub [f32; Self::SIZE as usize]);

impl VariablesArray {
    pub const SIZE: u8 = 16;
}

impl Index<u8> for VariablesArray {
    type Output = f32;
    fn index(&self, index: u8) -> &Self::Output {
        unsafe { self.0.get_unchecked(index as usize) }
    }
}

impl IndexMut<u8> for VariablesArray {
    fn index_mut(&mut self, index: u8) -> &mut Self::Output {
        unsafe { self.0.get_unchecked_mut(index as usize) }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq, Reflect, AsRefStr, Display, )]
pub enum Operation {
    Add, Subtract, Multiply, MultiplyOpo, Divide, Modulo, Log, Min, Max, Pow, Assign, Mean, Abs, MultiplyNormalized, MultiplyNormalizedAbs, ClearArray,
}



#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Reflect, )]
pub enum Operand {
    StackArray(u8),
    Value(f32),
    Entity(Entity, u64,),
    HashPos(u64),
    PoissonDisk(PoissonDisk),
}
impl Operand {
    pub fn new_poisson_disk(min_distance: u8, seed: u64) -> Result<Self, BevyError> {
        PoissonDisk::new(min_distance, seed).map(Self::PoissonDisk)
    }
}
impl Default for Operand { fn default() -> Self { Self::Value(0.0) } }
impl From<f32> for Operand { fn from(v: f32) -> Self { Self::Value(v) } }


#[derive(Component, Debug, Clone, Copy, Reflect)]
#[require(VariablesArray)]
pub struct OplistRef(pub Entity);
