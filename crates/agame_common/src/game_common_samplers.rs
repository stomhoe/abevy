
use bevy::{ecs::entity::{EntityHashMap, MapEntities}, platform::collections::HashMap, prelude::*};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use::tilemap_shared::*;
use crate::game_common_seris::NormalDistSeri;
use bevy_replicon::prelude::*;
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
        #[derive(Debug, Clone, Component, Default, )]
        #[require(common::common_components::Prefix::trunc($entityprefix), bevy_replicon::shared::replication::Replicated, common::common_components::AssetScoped, )]
        pub struct $ty {
            weights: Vec<($inner, f32)>,
            cumulative_weights: Vec<f32>,
            total_weight: f32,
        }
        $crate::define_weightedsampler_impl!($ty, $inner);
    };
}
#[derive(Debug, Clone, Component, Default, )]
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

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub struct BiomeTagWeightAtMacroChunk {
    pub biome: Entity,
    pub weight: f32,
    pub pack_count_multiplier_mean: f32,
    pub pack_count_multiplier_std_dev: f32,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct BiomePackCountMultiplierStats {
    pub mean_sum: f32,
    pub std_dev_sum: f32,
    pub samples: u32,
}
impl BiomePackCountMultiplierStats {
    pub fn add_sample(&mut self, mean: f32, std_dev: f32) {
        self.mean_sum += mean;
        self.std_dev_sum += std_dev.max(0.0);
        self.samples = self.samples.saturating_add(1);
    }

    pub fn averaged_mean(&self) -> f32 {
        if self.samples == 0 {
            return 1.0;
        }
        self.mean_sum / self.samples as f32
    }

    pub fn averaged_std_dev(&self) -> f32 {
        if self.samples == 0 {
            return 0.0;
        }
        self.std_dev_sum / self.samples as f32
    }
}

#[derive(Debug, Clone, Default)]
pub struct BiomeTagDistributionAtTimeout {
    pub biome_sampler: EntityWeightedSampler,
    pub pack_count_multiplier_stats: EntityHashMap<BiomePackCountMultiplierStats>,
}
impl BiomeTagDistributionAtTimeout {
    pub fn averaged_pack_count_multiplier_stats(&self, biome: Entity) -> BiomePackCountMultiplierStats {
        self.pack_count_multiplier_stats.get(&biome).copied().unwrap_or(BiomePackCountMultiplierStats {
            mean_sum: 1.0,
            std_dev_sum: 0.0,
            samples: 1,
        })
    }
}

#[derive(Resource, Debug, Default)]
pub struct MacroChunkBiomeTagDistributionMap(pub HashMap<(DimensionRef, MacroChunkPos), BiomeTagDistributionAtTimeout>);
impl MacroChunkBiomeTagDistributionMap {
    pub fn add_tag_weights<I>(&mut self, dim_ref: DimensionRef, chunk_pos: MacroChunkPos, tag_weights: I)
    where
        I: IntoIterator<Item = BiomeTagWeightAtMacroChunk>,
    {
        let dist = self.0.entry((dim_ref, chunk_pos)).or_default();
        let mut changed = false;
        for tag_weight in tag_weights {
            let tag = tag_weight.biome;
            let weight = tag_weight.weight;
            if !weight.is_finite() || weight <= 0.0 {
                continue;
            }
            changed = true;
            dist
                .pack_count_multiplier_stats
                .entry(tag)
                .or_default()
                .add_sample(
                    tag_weight.pack_count_multiplier_mean,
                    tag_weight.pack_count_multiplier_std_dev,
                );
            let Some((_, accumulated_weight)) = dist
                .biome_sampler
                .weights
                .iter_mut()
                .find(|(existing_tag, _)| *existing_tag == tag)
            else {
                dist.biome_sampler.weights.push((tag, weight));
                continue;
            };
            *accumulated_weight += weight;
        }
        if changed {
            rebuild_entity_weighted_sampler(&mut dist.biome_sampler);
        }
    }
}

fn rebuild_entity_weighted_sampler(sampler: &mut EntityWeightedSampler) {
    sampler.cumulative_weights.clear();
    sampler.cumulative_weights.reserve(sampler.weights.len());
    let mut acc = 0.0;
    for &(_, weight) in &sampler.weights {
        acc += weight;
        sampler.cumulative_weights.push(acc);
    }
    sampler.total_weight = acc;
}

macro_rules! define_sprite_normal_dist {
    ($dist_name:ident) => {
        ::paste::paste! {
            #[derive(Component, Default, Debug, Clone, Deserialize, Serialize)]
            pub struct $dist_name(pub CappedNormalDist);
            impl $dist_name {
                pub fn new(seri: NormalDistSeri) -> Self {
                    Self(CappedNormalDist::from_seri(seri))
                }
                pub fn sample(&self, rng: &mut impl rand::Rng) -> [<$dist_name Result>] {
                    [<$dist_name Result>](self.0.sample(rng))
                }
            }

            #[derive(Component, Debug, Default, Deserialize, Serialize, Copy, Clone, Reflect, MapEntities)]
            /// must be inserted into Being entities after sampling
            pub struct [<$dist_name Result>](pub f32);

            pub fn [<plugin_ $dist_name:snake>](app: &mut App) {
                use common::AppRegisterAndReplicateExt;
                app
                    .replicate::<$dist_name>()
                    .regrepli::<[<$dist_name Result>]>();
            }
        }
    };
}

define_sprite_normal_dist!(SpriteHoriNormalDist);
define_sprite_normal_dist!(SpriteVertNormalDist);
define_sprite_normal_dist!(SpriteGlobalNormalDist);

#[derive(Component, Debug, Default, Deserialize, Serialize, Copy, Clone, )]
pub struct ScaleHpAndStrengthWithSize;

#[derive(Component, Default, Debug, Clone, Deserialize, Serialize)]
pub struct CappedNormalDist {
    pub min: f32,
    pub max: f32,
    pub mean: f32,
    pub std_dev: f32,
}
impl CappedNormalDist {
    pub fn new(min: f32, max: f32, mean: f32, std_dev: f32) -> Self {
        let mut min = min.max(0.01);
        let mut max = max.max(0.01);
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
            error!(
                "CappedNormalDist: std_dev ({}) is larger than the range ({} - {} = {}). \
                 Most samples will be clamped.",
                std_dev, max, min, range
            );
        }
        Self {min, max, mean, std_dev,}
    }

    pub fn sample(&self, rng: &mut impl rand::Rng) -> f32 {
        use rand_distr::{Normal, Distribution};

        if self.std_dev == 0.0 || self.min == self.max {
            return self.mean.clamp(self.min, self.max);
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

        normal.sample(rng).clamp(self.min, self.max)
    }
    pub fn from_seri(seri: NormalDistSeri) -> Self {
        Self::new(seri.min, seri.max, seri.mean, seri.std_dev)
    }
}
