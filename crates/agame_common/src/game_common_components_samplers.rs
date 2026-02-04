
use bevy::{ecs::entity::MapEntities, prelude::*};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use::tilemap_shared::*;
#[macro_export]
macro_rules! define_weightedsampler_impl {
    ($ty:ident, $inner:ty) => {
        impl $ty {
            pub fn new(weights: &Vec<($inner, f32)>) -> Self {
                let mut weights_vec = Vec::with_capacity(weights.len());
                for (item, weight) in weights.iter().cloned() {
                    if weight < 0.0 {
                        error!(
                            "Negative weight ({}) encountered in WeightedSampler for value {:?}, skipping entry.",
                            weight, item
                        );
                        continue;
                    }
                    weights_vec.push((item, weight));
                }
                let mut cumulative_weights = Vec::with_capacity(weights_vec.len());
                let mut acc = 0.0;
                for &(_, w) in &weights_vec {
                    acc += w;
                    cumulative_weights.push(acc);
                }
                let total_weight = acc;
                Self {
                    weights: weights_vec,
                    cumulative_weights,
                    total_weight,
                }
            }
            pub fn insert(&mut self, item: $inner, weight: f32) {
                if weight < 0.0 {
                    warn!(
                        "Negative weight ({}) encountered in WeightedSampler for value {:?}.",
                        weight, item
                    );
                }
                let weight = weight.max(0.0);
                self.weights.push((item, weight));
                self.total_weight += weight;
                self.cumulative_weights.push(self.total_weight);
            }

            fn sample_index(&self, rng_val: f32) -> Option<usize> {
                if self.weights.is_empty() {
                    return None;
                }
                let mut rng_val = rng_val;
                if rng_val >= 1.0 { rng_val = 0.999_999; }
                let target = rng_val * self.total_weight;
                match self.cumulative_weights.binary_search_by(|w| w.partial_cmp(&target).unwrap()) {
                    Ok(idx) | Err(idx) => Some(idx),
                }
            }
            pub fn sample_with_pos(&self, pos: GlobalTilePos, settings: &GlobalGenSettings, dimension_hash: common::common_id_components::HashId) -> Option<$inner> {
                let hash_used_to_sample = pos.hash_for_weight_maps(settings, dimension_hash);
                let rng_val = (hash_used_to_sample as f64 / u64::MAX as f64) as f32;
                self.sample_index(rng_val)
                    .and_then(|idx| self.weights.get(idx).map(|(e, _)| e.clone()))
            }
            pub fn sample_with_rng(&self, rng: &mut impl rand::Rng) -> Option<$inner> {
                if self.weights.is_empty() { return None; }
                let rng_val = rng.random_range(0.0..=1.0);
                self.sample_index(rng_val)
                    .and_then(|idx| self.weights.get(idx).map(|(e, _)| e.clone()))
            }
            pub fn sample_with_rng_and_remove(&mut self, rng: &mut impl rand::Rng) -> Option<$inner> {
                if self.weights.is_empty() { return None; }
                let rng_val = rng.random_range(0.0..=1.0);
                let idx = self.sample_index(rng_val)?;
                let (item, weight) = self.weights.remove(idx);//don't use swap_remove, order is important
                self.total_weight -= weight;
                self.cumulative_weights.clear();
                let mut acc = 0.0;
                for &(_, w) in &self.weights {
                    acc += w;
                    self.cumulative_weights.push(acc);
                }
                Some(item)
            }
            pub fn remove(&mut self, item: &$inner) -> bool
            where
                $inner: PartialEq,
            {
                if let Some(pos) = self.weights.iter().position(|(e, _)| e == item) {
                    let (_, weight) = self.weights.remove(pos);
                    self.total_weight -= weight;
                    self.cumulative_weights.clear();
                    let mut acc = 0.0;
                    for &(_, w) in &self.weights {
                        acc += w;
                        self.cumulative_weights.push(acc);
                    }
                    true
                } else {
                    false
                }
            }
        }
        impl std::ops::Deref for $ty {
            type Target = Vec<($inner, f32)>;
            fn deref(&self) -> &Self::Target {
                &self.weights
            }
        }
        impl std::ops::DerefMut for $ty {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.weights
            }
        }
        impl Serialize for $ty {
            fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                self.weights.serialize(serializer)
            }
        }
        impl<'de> Deserialize<'de> for $ty {
            fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                let weights: Vec<($inner, f32)> = Deserialize::deserialize(deserializer)?;
                let mut cumulative_weights = Vec::with_capacity(weights.len());
                let mut acc = 0.0;
                for &(_, w) in &weights {
                    acc += w;
                    cumulative_weights.push(acc);
                }
                let total_weight = acc;
                Ok($ty {
                    weights,
                    cumulative_weights,
                    total_weight,
                })
            }
        }
    };
}
#[macro_export]
macro_rules! define_weightedsampler {
    ($ty:ident, $inner:ty, $entityprefix:expr) => {
        use common::common_components::{Prefix, AssetScoped};
        #[derive(Debug, Clone, Reflect, Component)]
        #[require(Prefix::trunc($entityprefix), bevy_replicon::shared::replication::Replicated, AssetScoped, )]
        pub struct $ty {
            weights: Vec<($inner, f32)>,
            cumulative_weights: Vec<f32>,
            total_weight: f32,
        }
        $crate::define_weightedsampler_impl!($ty, $inner);
    };
}
#[derive(Debug, Clone, Component, Default, Reflect)]
#[component(map_entities)]
pub struct EntityWeightedSampler {
    weights: Vec<(Entity, f32)>,
    cumulative_weights: Vec<f32>,
    total_weight: f32,
}
define_weightedsampler_impl!(EntityWeightedSampler, Entity);
impl MapEntities for EntityWeightedSampler {
    fn map_entities<E: EntityMapper>(&mut self, entity_mapper: &mut E) {
        for (ent, _) in &mut self.weights {
            *ent = entity_mapper.get_mapped(*ent);
        }
    }
}
#[derive(Component, Debug, Deserialize, Serialize, Copy, Clone, Hash, PartialEq, Eq, Reflect, MapEntities)]
pub struct WeightedSamplerRef(#[entities] pub Entity);


define_weightedsampler!(StringWeightedSampler, String, "StringWeightedSampler");            