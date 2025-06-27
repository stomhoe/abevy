
use bevy::{math::U8Vec2, platform::collections::HashMap};
#[allow(unused_imports)] use bevy::prelude::*;
use bevy_ecs_tilemap::tiles::{TileColor, TileFlip, TileTextureIndex, TileVisible};
use fastnoise_lite::FastNoiseLite;
use rand::Rng;
use rand_pcg::Pcg64;
use superstate::{SuperstateInfo};

use crate::game::tilemap::{tile_imgs::*};

#[derive(Component, Default, )]
pub struct FnlComp(pub FastNoiseLite);

#[derive(Component, Debug, Default, )]
pub struct Thresholds(pub Vec<(f32, Entity)>); //usar menor igual valor -> entidad. Entidad-> tiledist?


#[derive(Component, Debug, Default, )]
pub struct Tree();

//ES COMPONENT PORQ PUEDE HABER UNO PARA ARBUSTOS, OTRO PARA ARBOLES, ETC
//VA EN UNA ENTIDAD PROPIA ASI ES QUERYABLE. AGREGAR MARKER COMPONENTS PARA DISTINTOS TIPOS DE VEGETACIÓN
#[derive(Component, Debug, Default, )]
pub struct TileWeightedMap(
    pub HashMap<Entity, f32>, 
);
impl TileWeightedMap {
    pub fn new() -> Self {
        Self(HashMap::new())
    }
    pub fn insert(&mut self, entity: Entity, weight: f32) {
        self.0.insert(entity, weight);
    }
    pub fn extract_random(&self, rng: &mut Pcg64) -> Option<Entity> {
       
        if self.0.is_empty() {
            return None;
        }

        let total_weight: f32 = self.0.values().sum();
        let mut random_value = rng.random_range(0.0..total_weight);

        for (entity, weight) in &self.0 {
            if random_value < *weight {
                return Some(*entity);
            }
            random_value -= weight;
        }
        None

    }
    pub fn extract_random_with_other_maps(&self, rng: &mut Pcg64, others: Vec<TileWeightedMap>) -> Option<Entity> {
        // Combine all maps into a single HashMap<Entity, f32>
        let mut combined = self.0.clone();
        for map in others {
            for (entity, weight) in map.0 {
                *combined.entry(entity).or_insert(0.0) += weight;
            }
        }

        if combined.is_empty() {
            return None;
        }

        let total_weight: f32 = combined.values().sum();
        let mut random_value = rng.random_range(0.0..total_weight);

        for (entity, weight) in &combined {
            if random_value < *weight {
                return Some(*entity);
            }
            random_value -= weight;
        }
        None
    }

}


#[derive(Component, Debug, Default, Hash, PartialEq, Eq, Clone, Copy)]
pub enum UsedShader{
    #[default]
    None,
    Grass
}