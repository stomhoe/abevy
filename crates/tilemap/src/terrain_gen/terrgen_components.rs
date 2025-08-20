
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

