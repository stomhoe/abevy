use bevy::{ecs::entity::{EntityHashMap, MapEntities}, prelude::*};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use crate::*;

#[derive(Asset, TypePath, Debug, Clone, Deserialize, )]
///https://homepage.divms.uiowa.edu/~mbognar/applets/normal.html
pub struct NormalDistSeri {
    pub min_dev: f32,
    pub max_dev: f32,
    pub mean: f32,
    pub std_dev: f32,
}
impl NormalDistSeri {
    fn sentinel() -> Self {
        Self {
            min_dev: f32::NAN,
            max_dev: f32::NAN,
            mean: f32::NAN,
            std_dev: f32::NAN,
        }
    }
    pub fn is_sentinel(&self) -> bool {
        self.min_dev.is_nan() && self.max_dev.is_nan() && self.mean.is_nan() && self.std_dev.is_nan()
    }
}
impl Default for NormalDistSeri {
    fn default() -> Self {
        Self::sentinel()
    }
}

impl From<CappedNormalDist> for NormalDistSeri {
    fn from(value: CappedNormalDist) -> Self {
        Self {
            min_dev: value.min_dev,
            max_dev: value.max_dev,
            mean: value.mean,
            std_dev: value.std_dev,
        }
    }
}

#[derive(Debug, Clone, Component, Default, )]
#[component(map_entities)]
pub struct EntityWeightedSampler {
    weights: Vec<(Entity, f32)>,
    cumulative_weights: Vec<f32>,
    total_weight: f32,
}
define_weightedsampler_impl!(EntityWeightedSampler, Entity);
impl_weighted_sampler_serialization!(EntityWeightedSampler, Entity);
impl MapEntities for EntityWeightedSampler {
    fn map_entities<E: EntityMapper>(&mut self, entity_mapper: &mut E) {
        for (ent, _) in &mut self.weights {
            *ent = entity_mapper.get_mapped(*ent);
        }
    }
}

#[derive(Debug, Clone, Component, Default)]
#[component(map_entities)]
pub struct EntityCountMapWeightedSampler {
    weights: Vec<(EntityHashMap<u32>, f32)>,
    cumulative_weights: Vec<f32>,
    total_weight: f32,
}
define_weightedsampler_impl!(EntityCountMapWeightedSampler, EntityHashMap<u32>);
impl MapEntities for EntityCountMapWeightedSampler {
    fn map_entities<E: EntityMapper>(&mut self, entity_mapper: &mut E) {
        for (count_map, _) in &mut self.weights {
            let old_map = std::mem::take(count_map);
            let mut remapped = EntityHashMap::with_capacity(old_map.len());
            for (ent, count) in old_map {
                remapped.insert(entity_mapper.get_mapped(ent), count);
            }
            *count_map = remapped;
        }
    }
}

#[derive(Debug, Clone, Component, Default, )]
pub struct GlobalTilePosWeightedSampler {
    weights: Vec<(GlobalTilePos, f32)>,
    cumulative_weights: Vec<f32>,
    total_weight: f32,
}
define_weightedsampler_impl!(GlobalTilePosWeightedSampler, GlobalTilePos);

define_weightedsampler!(StringWeightedSampler, String, "StringWeightedSampler");



define_sprite_normal_dist!(SpriteHoriNormalDist);
define_sprite_normal_dist!(SpriteVertNormalDist);
define_sprite_normal_dist!(SpriteGlobalNormalDist);

#[derive(Component, Debug, Default, Deserialize, Serialize, Copy, Clone, )]
pub struct ScaleHpAndStrengthWithSampledSize;

#[derive(Component, Default, Debug, Clone, )]
pub struct CappedNormalDist {
    pub min_dev: f32,
    pub max_dev: f32,
    pub mean: f32,
    pub std_dev: f32,
}
impl CappedNormalDist {
    pub fn new(min_dev: f32, max_dev: f32, mean: f32, std_dev: f32) -> Self {
        let min_dev = min_dev.max(0.0);
        let max_dev = max_dev.max(0.0);
        let mut min = mean - min_dev;
        let mut max = mean + max_dev;
        let mut mean = mean;
        let std_dev = std_dev.max(0.0);
        if min > max {
            error!(
                "CappedNormalDist: min ({}) is greater than max ({}). Swapping values.",
                min, max
            );
            std::mem::swap(&mut min, &mut max);
        }
        if mean < min || mean > max {
            error!(
                "CappedNormalDist: mean ({}) is outside the range [{}, {}]. Clamping to range.",
                mean, min, max
            );
            mean = mean.clamp(min, max);
        }
        let range = max - min;
        if std_dev > range {
            warn!(
                "CappedNormalDist: std_dev ({}) is larger than the range ({} - {} = {}). \
                 Most samples will be clamped.",
                std_dev, max, min, range
            );
        }
        Self {min_dev, max_dev, mean, std_dev,}
    }

    pub fn sample(&self, rng: &mut impl rand::Rng) -> f32 {
        use rand_distr::{Normal, Distribution};

        let min = self.mean - self.min_dev;
        let max = self.mean + self.max_dev;
        if self.std_dev == 0.0 || min == max {
            return self.mean.clamp(min, max);
        }

        let normal = match Normal::new(self.mean, self.std_dev) {
            Ok(dist) => dist,
            Err(e) => {
                error!(
                    "CappedNormalDist: Failed to create normal distribution with mean={}, std_dev={}: {}. \
                     Using fallback distribution.",
                    self.mean, self.std_dev, e
                );
                Normal::new(self.mean, 0.1)
                    .unwrap_or_else(|_| {
                        error!(
                            "CappedNormalDist: Fallback distribution also failed. Returning mean value."
                        );
                        Normal::new(self.mean, 0.01)
                            .expect("Final fallback should always work")
                    })
            }
        };

        normal.sample(rng).clamp(min, max)
    }
    pub fn from_seri(seri: NormalDistSeri) -> Self {
        Self::new(seri.min_dev, seri.max_dev, seri.mean, seri.std_dev)
    }
}
