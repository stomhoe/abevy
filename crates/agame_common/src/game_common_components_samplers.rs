
use bevy::{ecs::entity::MapEntities, platform::collections::HashMap, prelude::*};
use bevy_replicon::prelude::Replicated;
use common::common_components::*;
use rand::Rng;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use tilemap_shared::{AaGlobalGenSettings, GlobalTilePos, HashablePosVec};
#[allow(unused_imports)] use bevy::prelude::*;


#[derive(Component, Debug, Deserialize, Serialize, Reflect)]
#[require(EntityPrefix::new_truncated("ColorWSampler"), AssetScoped, Replicated)]
pub struct ColorSampler(pub WeightedSampler<[u8; 4]>);
impl ColorSampler {
    pub fn new(weights: &Vec<([u8; 4], f32)>) -> Self {
        let weighted_sampler = WeightedSampler::new(weights);
        Self(weighted_sampler)
    }
}

#[derive(Component, Debug, Deserialize, Serialize, Copy, Clone, Hash, PartialEq, Eq, Reflect)]
pub struct WeightedSamplerRef(#[entities] pub Entity);

#[derive(Component, Debug, Deserialize, Serialize, Copy, Clone, Hash, PartialEq, Eq, Reflect)]
pub struct ColorSamplerRef(#[entities] pub Entity);


#[derive(Debug, Clone, Component, Default)]
#[require(EntityPrefix::new_truncated("HashPosEntWSampler"), Replicated, AssetScoped, TgenHotLoadingScoped)]
#[component(map_entities)]
pub struct EntityWeightedSampler {
    entities_weights: Vec<(Entity, f32)>,
    cumulative_weights: Vec<f32>, total_weight: f32,
}
impl EntityWeightedSampler {
    //PROBLEMA, PUEDE Q LAS ENTITIES DE ESTE HASHMAP NO SE GUARDEN EN EL MISMO ORDEN ENTRE CADA CARGADA, POR LO Q HAY Q GUARDARLO EN LA SAVE TMB
    pub fn new(weights_map: &Vec<(Entity, f32)>) -> Self {
        let mut entities_weights = Vec::with_capacity(weights_map.len());
        for (entity, weight) in weights_map.iter().cloned() {
            entities_weights.push((entity, weight));
        }
        let mut cumulative_weights = Vec::with_capacity(entities_weights.len());
        let mut acc = 0.0;
        for &(_, w) in &entities_weights {
            acc += w;
            cumulative_weights.push(acc);
        }
        let total_weight = acc;
        Self {
            entities_weights,
            cumulative_weights,
            total_weight,
        }
    }
    fn sample_index(&self, rng_val: f32) -> Option<usize> {
        if self.entities_weights.is_empty() {
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
            .and_then(|idx| self.entities_weights.get(idx).map(|(e, _)| *e))
    }
    pub fn sample_with_rng(&self, rng: &mut impl Rng) -> Option<Entity> {
        if self.entities_weights.is_empty() {return None;}
        let rng_val = rng.random_range(0.0..=1.0);
        self.sample_index(rng_val)
            .and_then(|idx| self.entities_weights.get(idx).map(|(e, _)| *e))
    }
}
impl Serialize for EntityWeightedSampler {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.entities_weights.serialize(serializer)
    }
}
impl<'de> Deserialize<'de> for EntityWeightedSampler {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let entities_weights: Vec<(Entity, f32)> = Deserialize::deserialize(deserializer)?;
        let mut cumulative_weights = Vec::with_capacity(entities_weights.len());
        let mut acc = 0.0;
        for &(_, w) in &entities_weights {
            acc += w;
            cumulative_weights.push(acc);
        }
        let total_weight = acc;
        Ok(EntityWeightedSampler {
            entities_weights,
            cumulative_weights,
            total_weight,
        })
    }
}
impl MapEntities for EntityWeightedSampler {
    fn map_entities<E: EntityMapper>(&mut self, entity_mapper: &mut E) {
        for (ent, _) in &mut self.entities_weights {
            *ent = entity_mapper.get_mapped(*ent);
        }
    }
}

#[derive(Debug, Clone, Reflect, Default)]
pub struct WeightedSampler<T: Clone + Serialize + Eq + std::hash::Hash + std::fmt::Debug> {
    choices_and_weights: Vec<(T, f32)>, cumulative_weights: Vec<f32>, total_weight: f32,
}
impl<T: Clone + Serialize + Eq + std::hash::Hash + std::fmt::Debug> WeightedSampler<T> {
    pub fn new(weights: &Vec<(T, f32)>) -> Self {
        let mut choices_and_weights = Vec::with_capacity(weights.len());
        let mut seen = std::collections::HashSet::new();
        for (choice, weight) in weights.iter() {
            if *weight < 0.0 {
                error!("Negative weight ({}) for choice {:?}, skipping.", weight, choice);
                continue;
            }
            if !seen.insert(choice) {
                error!("Duplicate choice ({:?}) found, skipping.", choice);
                continue;
            }
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

impl<T: Clone + Serialize + Eq + std::hash::Hash + std::fmt::Debug + for<'de> Deserialize<'de>> Serialize for WeightedSampler<T> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        (&self.choices_and_weights).serialize(serializer)
    }
}
impl<'de, T> Deserialize<'de> for WeightedSampler<T>
where
    T: Clone + Serialize + Eq + std::hash::Hash + std::fmt::Debug + Deserialize<'de>,
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