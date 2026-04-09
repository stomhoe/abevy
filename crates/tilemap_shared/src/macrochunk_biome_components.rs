use bevy::{platform::collections::HashMap, prelude::*};
use common::common_id_components::{HashId, HashIdMap};
use rand_distr::{Distribution, Normal};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use crate::{
    tilemap_positioning::ChunkPos,
    tilemap_shared_samplers::HashIdWeightedSampler,
};



#[derive(Debug, Clone, Copy, Default)]
pub struct BiomePackCountAvgedNormDists {
	pub mean_sum: f32,
	pub std_dev_sum: f32,
	pub samples: u32,
}

impl BiomePackCountAvgedNormDists {
	pub fn add_sample(&mut self, mean: f32, std_dev: f32) {
		self.mean_sum += mean;
		self.std_dev_sum += std_dev.max(0.0);
		self.samples = self.samples.saturating_add(1);
	}

	pub fn sample_pack_count_int_multiplier(self, rng: &mut impl rand::Rng) -> usize {
		let mean = self.averaged_mean().max(0.0);
		let std_dev = self.averaged_std_dev().max(0.0);
		if std_dev <= f32::EPSILON {
			return self.sample_fractional_pack_count(mean, rng);
		}
		let Ok(dist) = Normal::new(mean, std_dev.max(0.01)) else {
			return self.sample_fractional_pack_count(mean, rng);
		};
		self.sample_fractional_pack_count(dist.sample(rng).max(0.0), rng)
	}

	fn averaged_mean(&self) -> f32 {
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

	fn sample_fractional_pack_count(self, sampled_multiplier: f32, rng: &mut impl rand::Rng) -> usize {
		if sampled_multiplier <= 0.0 {
			return 0;
		}
		let guaranteed_packs: usize = sampled_multiplier.floor() as usize;
		let extra_pack_probability: f32 = sampled_multiplier.fract();
		guaranteed_packs + usize::from(rng.random::<f32>() < extra_pack_probability)
	}
}

#[derive(Component, Debug, Clone, Default)]
//TODO borrar cuano ya deje de ser usado
pub struct BiomeDistribution {
	pub produced_biome_sampler: HashIdWeightedSampler,
	pub pack_count_multiplier_avged_norm_dists_per_biome: HashIdMap<BiomePackCountAvgedNormDists>,
	pub accumulated_chunk_weights_per_biome: HashIdMap<HashMap<ChunkPos, f32>>,
}
impl BiomeDistribution {
    pub fn sample_biome_hash_id(
        &self,
        rng: &mut impl rand::Rng,
    ) -> Option<HashId> {
        self.produced_biome_sampler.sample_with_rng(rng)
    }
}

#[derive(Component, Debug, Clone, Copy, Default)]
pub struct MacrochunkPendingBiomeSamples(pub u32);

impl BiomeDistribution {
	pub fn add_tag_weights_in_chunk<I>(&mut self, chunk_pos: ChunkPos, tag_weights: I)
	where
		I: IntoIterator<Item = BiomeTagWeightAtMacrochunk>,
	{
		for tag_weight in tag_weights {
			let tag = tag_weight.biome;
			let weight = tag_weight.weight;
			if !weight.is_finite() || weight <= 0.0 {
				continue;
			}
			self
				.pack_count_multiplier_avged_norm_dists_per_biome
				.0
				.entry(tag)
				.or_default()
				.add_sample(
					tag_weight.pack_count_multiplier_mean,
					tag_weight.pack_count_multiplier_std_dev,
				);
			if let Err(negative_item) = self.produced_biome_sampler.add_or_accumulate_weight(tag, weight) {
				crate::log_negative_weighted_sampler_items!("macrochunk_biome", tag, vec![negative_item]);
			}
			let chunk_weights = self
				.accumulated_chunk_weights_per_biome
				.0
				.entry(tag)
				.or_default();
			let entry = chunk_weights.entry(chunk_pos).or_default();
			*entry += weight;
		}
	}

	pub fn sorted_chunk_candidates_for_biome(&self, biome_hash: HashId) -> Vec<ChunkPos> {
		let Some(cpos_weight_list) = self.accumulated_chunk_weights_per_biome.get_opt(biome_hash) else {
			return Vec::new();
		};
		let mut sorted = cpos_weight_list
			.iter()
			.map(|(chunk_pos, weight)| (*chunk_pos, *weight))
			.collect::<Vec<_>>();
		sorted.sort_by(|lhs, rhs| rhs.1.partial_cmp(&lhs.1).unwrap_or(Ordering::Equal));
		sorted.into_iter().map(|(chunk_pos, _)| chunk_pos).collect()
	}

	pub fn averaged_pack_count_multiplier_stats(&self, biome: HashId) -> BiomePackCountAvgedNormDists {
		self.pack_count_multiplier_avged_norm_dists_per_biome.get_opt(biome).copied().unwrap_or(BiomePackCountAvgedNormDists {
			mean_sum: 1.0,
			std_dev_sum: 0.0,
			samples: 1,
		})
	}
}


#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub struct BiomeTagWeightAtMacrochunk {
	pub biome: HashId,
	pub weight: f32,
	pub pack_count_multiplier_mean: f32,
	pub pack_count_multiplier_std_dev: f32,
}
