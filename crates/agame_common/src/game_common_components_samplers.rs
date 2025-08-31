
use bevy::{ecs::entity::MapEntities, platform::collections::HashMap, prelude::*};
use bevy_replicon::prelude::Replicated;
use common::common_components::*;
use rand::Rng;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use tilemap_shared::{AaGlobalGenSettings, GlobalTilePos, HashablePosVec};
use std::time::Duration;
#[allow(unused_imports)] use bevy::prelude::*;
use splines::{Interpolation, Key, Spline};



pub type ColorSampler = WeightedSampler<[u8; 4]>;



#[derive(Component, Debug, Deserialize, Serialize, Copy, Clone, Hash, PartialEq, Eq, Reflect)]
pub struct WeightedSamplerRef(#[entities] pub Entity);

#[derive(Component, Debug, Deserialize, Serialize, Copy, Clone, Hash, PartialEq, Eq, Reflect)]
pub struct ColorSamplerRef(#[entities] pub Entity);


#[derive(Debug, Clone, Component, Default)]
#[require(EntityPrefix::new("HashPosEntWSampler"), Replicated, AssetScoped, TgenScoped)]
pub struct EntiWeightedSampler {
    #[entities]entities: Vec<Entity>, weights: Vec<f32>,
    cumulative_weights: Vec<f32>, total_weight: f32,
}
impl EntiWeightedSampler {
    //PROBLEMA, PUEDE Q LAS ENTITIES DE ESTE HASHMAP NO SE GUARDEN EN EL MISMO ORDEN ENTRE CADA CARGADA, POR LO Q HAY Q GUARDARLO EN LA SAVE TMB
    pub fn new(weights_map: &HashMap<Entity, f32>) -> Self {
        let mut entities = Vec::with_capacity(weights_map.len());
        let mut weights = Vec::with_capacity(weights_map.len());
        for (&entity, &weight) in weights_map.iter() {
            entities.push(entity);
            weights.push(weight); 
        }
        let mut cumulative_weights = Vec::with_capacity(weights.len());
        let mut acc = 0.0;
        for &w in &weights {
            acc += w;
            cumulative_weights.push(acc);
        }
        let total_weight = acc;
        Self {
            entities,
            weights,
            cumulative_weights,
            total_weight,
        }
    }
    fn sample_index(&self, rng_val: f32) -> Option<usize> {
        if self.entities.is_empty() {
            return None;
        }
        let mut rng_val = rng_val;
        if rng_val >= 1.0 { rng_val = 0.999_999; }
        let target = rng_val * self.total_weight;
        match self.cumulative_weights.binary_search_by(|w| w.partial_cmp(&target).unwrap()) {
            Ok(idx) | Err(idx) => Some(idx),
        }
    }

    pub fn sample_with_pos(&self, settings: &AaGlobalGenSettings, pos: GlobalTilePos) -> Option<Entity> {
        let hash_used_to_sample = pos.hash_for_weight_maps(settings);
        let rng_val = (hash_used_to_sample as f64 / u64::MAX as f64) as f32;
        self.sample_index(rng_val)
            .and_then(|idx| self.entities.get(idx).map(|e| *e))
    }
    pub fn sample_with_rng(&self, rng: &mut impl Rng) -> Option<Entity> {
        if self.entities.is_empty() {return None;}
        let rng_val = rng.random_range(0.0..=1.0);
        self.sample_index(rng_val)
            .and_then(|idx| self.entities.get(idx).map(|e| *e))
    }
}
impl Serialize for EntiWeightedSampler {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        (&self.entities, &self.weights).serialize(serializer)
    }
}
impl<'de> Deserialize<'de> for EntiWeightedSampler {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let (entities, weights): (Vec<Entity>, Vec<f32>) = Deserialize::deserialize(deserializer)?;
        // Recompute cumulative_weights and total_weight
        let mut cumulative_weights = Vec::with_capacity(weights.len());
        let mut acc = 0.0;
        for &w in &weights {
            acc += w;
            cumulative_weights.push(acc);
        }
        let total_weight = acc;
        Ok(EntiWeightedSampler {
            entities,
            weights,
            cumulative_weights,
            total_weight,
        })
    }
}

#[derive(Debug, Clone, Component, )]
#[require(EntityPrefix::new("WSampler"), Replicated, AssetScoped, )]
pub struct WeightedSampler<T: Clone + Serialize> {
    choices_and_weights: Vec<(T, f32)>, cumulative_weights: Vec<f32>, total_weight: f32,
}
impl<T: Clone + Serialize> WeightedSampler<T> {
    pub fn new(weights_map: &HashMap<T, f32>) -> Self {
        let mut choices_and_weights = Vec::with_capacity(weights_map.len());
        for (choice, weight) in weights_map.iter() {
            choices_and_weights.push((choice.clone(), *weight));
        }
        let mut cumulative_weights = Vec::with_capacity(choices_and_weights.len());
        let mut acc = 0.0;
        for &(_, w) in &choices_and_weights {
            acc += w;
            cumulative_weights.push(acc);
        }
        let total_weight = acc;
        Self {
            choices_and_weights,
            cumulative_weights,
            total_weight,
        }
    }
    fn sample_index(&self, rng_val: f32) -> Option<usize> {
        if self.choices_and_weights.is_empty() {
            return None;
        }
        let mut rng_val = rng_val;
        if rng_val >= 1.0 { rng_val = 0.999_999; }
        let target = rng_val * self.total_weight;
        match self.cumulative_weights.binary_search_by(|w| w.partial_cmp(&target).unwrap()) {
            Ok(idx) | Err(idx) => Some(idx),
        }
    }
    pub fn sample_with_pos(&self, settings: &AaGlobalGenSettings, pos: GlobalTilePos) -> Option<T> {
        let hash_used_to_sample = pos.hash_for_weight_maps(settings);
        let rng_val = (hash_used_to_sample as f64 / u64::MAX as f64) as f32;
        self.sample_index(rng_val)
            .and_then(|idx| self.choices_and_weights.get(idx).map(|(choice, _)| choice.clone()))
    }
    pub fn sample_with_rng(&self, rng: &mut impl Rng) -> Option<T> {
        let rng_val = rng.random_range(0.0..=1.0);
        self.sample_index(rng_val)
            .and_then(|idx| self.choices_and_weights.get(idx).map(|(choice, _)| choice.clone()))
    }
}

impl MapEntities for WeightedSampler<Entity> {
    fn map_entities<E: bevy::ecs::entity::EntityMapper>(&mut self, entity_mapper: &mut E) {
        for (ent, _) in &mut self.choices_and_weights {
            *ent = entity_mapper.get_mapped(*ent);
        }
    }
}
impl<T: Clone + Serialize + for<'de> Deserialize<'de>> Default for WeightedSampler<T> {
    fn default() -> Self { Self { choices_and_weights: Vec::new(), cumulative_weights: Vec::new(), total_weight: 0.0 } }
}

impl<T: Clone + Serialize + for<'de> Deserialize<'de>> Serialize for WeightedSampler<T> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        (&self.choices_and_weights).serialize(serializer)
    }
}
impl<'de, T> Deserialize<'de> for WeightedSampler<T>
where
    T: Clone + Serialize + Deserialize<'de>,
{
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let choices_and_weights: Vec<(T, f32)> = Deserialize::deserialize(deserializer)?;
        let mut cumulative_weights = Vec::with_capacity(choices_and_weights.len());
        let mut acc = 0.0;
        for &(_, w) in &choices_and_weights {
            acc += w;
            cumulative_weights.push(acc);
        }
        let total_weight = acc;
        Ok(WeightedSampler {
            choices_and_weights,
            cumulative_weights,
            total_weight,
        })
    }
}