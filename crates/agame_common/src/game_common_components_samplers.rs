
use bevy::{ecs::entity::MapEntities, platform::collections::HashMap, prelude::*};
use bevy_replicon::prelude::Replicated;
use common::common_components::*;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use tilemap_shared::{AaGlobalGenSettings, GlobalTilePos, HashablePosVec};
#[allow(unused_imports)] use bevy::prelude::*;
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
            pub fn sample_with_pos(&self, settings: &AaGlobalGenSettings, pos: GlobalTilePos) -> Option<$inner> {
                let hash_used_to_sample = pos.hash_for_weight_maps(settings);
                let rng_val = (hash_used_to_sample as f64 / u64::MAX as f64) as f32;
                self.sample_index(rng_val)
                    .and_then(|idx| self.weights.get(idx).map(|(e, _)| *e))
            }
            pub fn sample_with_rng(&self, rng: &mut impl rand::Rng) -> Option<$inner> {
                if self.weights.is_empty() { return None; }
                let rng_val = rng.random_range(0.0..=1.0);
                self.sample_index(rng_val)
                    .and_then(|idx| self.weights.get(idx).map(|(e, _)| *e))
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
        #[derive(Debug, Clone, Reflect, Default, Component)]
        #[require(EntityPrefix::new_truncated($entityprefix), Replicated, AssetScoped, TgenHotLoadingScoped)]
        pub struct $ty {
            weights: Vec<($inner, f32)>,
            cumulative_weights: Vec<f32>,
            total_weight: f32,
        }
        $crate::define_weightedsampler_impl!($ty, $inner);
    };
}

define_weightedsampler!(ColorSampler, [u8; 4], "ColorWeightedSampler");

#[derive(Component, Debug, Deserialize, Serialize, Copy, Clone, Hash, PartialEq, Eq, Reflect)]
pub struct WeightedSamplerRef(#[entities] pub Entity);

#[derive(Component, Debug, Deserialize, Serialize, Copy, Clone, Hash, PartialEq, Eq, Reflect)]
pub struct ColorSamplerRef(#[entities] pub Entity);



#[derive(Debug, Clone, Component, Default)]
#[require(EntityPrefix::new_truncated("HashPosEntWSampler"), Replicated, AssetScoped, TgenHotLoadingScoped)]
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

