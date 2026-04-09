#[allow(unused_imports)]
use bevy::prelude::*;

#[macro_export]
macro_rules! log_negative_weighted_sampler_items {
    ($target:expr, $sampler_label:expr, $negative_items:expr) => {{
        for item in $negative_items {
            error!(
                target: $target,
                "Weighted sampler {} encountered a negative weight for value {:?}; clamped to 0.0",
                $sampler_label,
                item
            );
        }
    }};
}

#[macro_export]
macro_rules! log_negative_weighted_sampler_indices {
    ($target:expr, $sampler_label:expr, $weights:expr, $negative_indices:expr) => {{
        for index in $negative_indices {
            let Some((item, _)) = $weights.get(index) else {
                continue;
            };
            error!(
                target: $target,
                "Weighted sampler {} encountered a negative weight for value {:?}; clamped to 0.0",
                $sampler_label,
                item
            );
        }
    }};
}

#[macro_export]
macro_rules! impl_weighted_sampler_serialization {
    ($ty:ident, $inner:ty) => {
        impl Serialize for $ty {
            fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                self.weights.serialize(serializer)
            }
        }
        impl<'de> Deserialize<'de> for $ty {
            fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                let weights: Vec<($inner, f32)> = Deserialize::deserialize(deserializer)?;
                let (sampler, negative_indices) = $ty::build_from_weights(&weights);
                for index in negative_indices {
                    let Some((item, _)) = weights.get(index) else {
                        continue;
                    };
                    warn!(
                        "Negative weight encountered in deserialized weighted sampler for value {:?}; weight clamped to 0.0.",
                        item
                    );
                }
                Ok(sampler)
            }
        }
    };
}

#[macro_export]
macro_rules! define_weightedsampler_impl {
    ($ty:ident, $inner:ty) => {
        impl $ty {
            fn build_from_weights(weights: &Vec<($inner, f32)>) -> (Self, Vec<usize>) {
                let mut weights_vec = Vec::with_capacity(weights.len());
                let mut negative_indices = Vec::new();
                for (idx, (item, weight)) in weights.iter().cloned().enumerate() {
                    if weight < 0.0 {
                        negative_indices.push(idx);
                    }
                    weights_vec.push((item, weight.max(0.0)));
                }
                let mut cumulative_weights = Vec::with_capacity(weights_vec.len());
                let mut acc = 0.0;
                for &(_, w) in &weights_vec {
                    acc += w;
                    cumulative_weights.push(acc);
                }
                let total_weight = acc;
                (
                    Self {
                        weights: weights_vec,
                        cumulative_weights,
                        total_weight,
                    },
                    negative_indices,
                )
            }

            pub fn new(weights: &Vec<($inner, f32)>) -> (Self, Vec<usize>) {
                Self::build_from_weights(weights)
            }

            pub fn insert(&mut self, item: $inner, weight: f32) -> Result<(), $inner>
            where
                $inner: Clone,
            {
                if weight < 0.0 {
                    let clamped = weight.max(0.0);
                    self.weights.push((item.clone(), clamped));
                    self.total_weight += clamped;
                    self.cumulative_weights.push(self.total_weight);
                    return Err(item);
                }
                self.weights.push((item, weight));
                self.total_weight += weight;
                self.cumulative_weights.push(self.total_weight);
                Ok(())
            }

            pub fn add_or_accumulate_weight(&mut self, item: $inner, weight: f32) -> Result<(), $inner>
            where
                $inner: PartialEq + Clone,
            {
                if weight < 0.0 {
                    let clamped = weight.max(0.0);
                    let Some((_, existing_weight)) = self.weights.iter_mut().find(|(existing_item, _)| *existing_item == item) else {
                        let result = self.insert(item.clone(), clamped);
                        return result.map_err(|_| item);
                    };
                    *existing_weight += clamped;
                    self.rebuild();
                    return Err(item);
                }
                if weight == 0.0 {
                    return Ok(());
                }
                let Some((_, existing_weight)) = self.weights.iter_mut().find(|(existing_item, _)| *existing_item == item) else {
                    return self.insert(item.clone(), weight).map_err(|_| item);
                };
                *existing_weight += weight;
                self.rebuild();
                Ok(())
            }

            fn rebuild(&mut self) {
                self.cumulative_weights.clear();
                self.cumulative_weights.reserve(self.weights.len());
                let mut acc = 0.0;
                for &(_, w) in &self.weights {
                    acc += w;
                    self.cumulative_weights.push(acc);
                }
                self.total_weight = acc;
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
            pub fn sample_n_with_rng(&self, sample_count: usize, rng: &mut impl rand::Rng, out: &mut impl Extend<$inner>) {
                for _ in 0..sample_count {
                    let Some(sample) = self.sample_with_rng(rng) else {
                        break;
                    };
                    out.extend(std::iter::once(sample));
                }
            }
            pub fn sample_with_rng_and_remove(&mut self, rng: &mut impl rand::Rng) -> Option<$inner> {
                if self.weights.is_empty() { return None; }
                let rng_val = rng.random_range(0.0..=1.0);
                let idx = self.sample_index(rng_val)?;
                let (item, weight) = self.weights.remove(idx);//don't use swap_remove, order is important
                self.total_weight -= weight;
                self.rebuild();
                Some(item)
            }
            pub fn remove(&mut self, item: &$inner) -> bool
            where
                $inner: PartialEq,
            {
                if let Some(pos) = self.weights.iter().position(|(e, _)| e == item) {
                    let (_, weight) = self.weights.remove(pos);
                    self.total_weight -= weight;
                    self.rebuild();
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
#[macro_export]
macro_rules! define_sprite_normal_dist {
    ($dist_name:ident) => {
        ::paste::paste! {
            #[derive(Component, Default, Debug, Clone, )]
            pub struct $dist_name(pub CappedNormalDist);
            impl $dist_name {
                pub fn new(seri: NormalDistSeri) -> Self {
                    Self(CappedNormalDist::from_seri(seri))
                }
                pub fn sample(&self, rng: &mut impl rand::Rng) -> [<$dist_name Result>] {
                    [<$dist_name Result>](self.0.sample(rng))
                }
            }

            #[derive(Component, Debug, Default, Deserialize, Serialize, Copy, Clone, Reflect, )]
            /// must be inserted into Being entities after sampling
            pub struct [<$dist_name Result>](pub f32);

            pub fn [<plugin_ $dist_name:snake>](app: &mut App) {
                use bevy_replicon::prelude::AppRuleExt;
                app
                    .register_type::<[<$dist_name Result>]>()
                    .replicate::<[<$dist_name Result>]>()
                ;
            }
        }
    };
}
