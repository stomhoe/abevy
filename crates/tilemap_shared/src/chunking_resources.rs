use bevy::platform::collections::{HashMap, HashSet};
use bevy::prelude::*;
use common::common_id_components::{HashId, HashIdMap};
use smallvec::SmallVec;

use crate::*;

#[derive(Resource, Default)]
pub struct LoadedChunks (pub HashMap<(DimensionRef, ChunkPos), Entity>,);

#[derive(Resource, Debug, Default)]
pub struct DiscoveredMacroChunks(pub HashSet<(DimensionRef, MacroChunkPos)>);

#[derive(Debug, Clone, Default)]
pub struct BiomeTagDistributionAtTimeout {
    pub sums: HashIdMap<f32>,
    pub total_sum: f32,
}
impl BiomeTagDistributionAtTimeout {
    pub fn add_weight(&mut self, tag: HashId, weight: f32) {
        if !weight.is_finite() || weight <= 0.0 {
            return;
        }
        let entry = self.sums.0.entry(tag).or_insert(0.0);
        *entry += weight;
        self.total_sum += weight;
    }
    pub fn add_tag_weights<I>(&mut self, tag_weights: I)
    where
        I: IntoIterator<Item = (HashId, f32)>,
    {
        for (tag, weight) in tag_weights {
            self.add_weight(tag, weight);
        }
    }
    pub fn predominant_tag(&self) -> Option<HashId> {
        self.sums
            .iter()
            .max_by(|(_, left), (_, right)| left.total_cmp(right))
            .map(|(tag, _)| *tag)
    }
    pub fn predominant_tags(&self) -> Vec<HashId> {
        self
            .predominant_tags_with_sums()
            .into_iter()
            .map(|(tag, _)| tag)
            .collect()
    }
    pub fn predominant_tags_with_sums(&self) -> Vec<(HashId, f32)> {
        let mut ordered: Vec<(HashId, f32)> = self.sums.iter().map(|(tag, sum)| (*tag, *sum)).collect();
        ordered.sort_unstable_by(|left, right| right.1.total_cmp(&left.1));
        ordered
    }
    pub fn tag_sum(&self, tag: HashId) -> f32 {
        self.sums.get_opt(tag).copied().unwrap_or_default()
    }
    pub fn tag_percentage(&self, tag: HashId) -> f32 {
        if self.total_sum <= 0.0 {
            return 0.0;
        }
        self.tag_sum(tag) / self.total_sum
    }
    pub fn unique_tag_count(&self) -> usize {
        self.sums.len()
    }
}

#[derive(Resource, Debug, Default)]
pub struct MacroChunkBiomeTagDistributionMap(pub HashMap<(DimensionRef, MacroChunkPos), BiomeTagDistributionAtTimeout>);
impl MacroChunkBiomeTagDistributionMap {
    pub fn add_tag_weights<I>(&mut self, dim_ref: DimensionRef, chunk_pos: MacroChunkPos, tag_weights: I)
    where
        I: IntoIterator<Item = (HashId, f32)>,
    {
        let dist = self.0.entry((dim_ref, chunk_pos)).or_default();
        dist.add_tag_weights(tag_weights);
    }
}

pub type ReturnedVec = SmallVec<[Entity; 16]>;

#[derive(Debug, )]
pub struct ChunkEntityMatrix {
    cells: [ReturnedVec; ChunkPos::CHUNK_AREA],
}
impl ChunkEntityMatrix {
    pub fn new() -> Self {
        Self::default()
    }
    fn index(local: UVec2) -> usize {
        let width = ChunkPos::CHUNK_SIZE.x as usize;
        (local.y as usize * width) + local.x as usize
    }
    pub fn get(&self, local: UVec2) -> &[Entity] {
        self.cells.get(Self::index(local)).map_or(&[], |cell| cell.as_slice())
    }
    pub fn push(&mut self, local: UVec2, entity: Entity) {
        if let Some(cell) = self.cells.get_mut(Self::index(local)) {
            cell.push(entity);
        }
    }
    pub fn swap_remove(&mut self, local: UVec2, entity: Entity) -> Option<()> {
        let cell = self.cells.get_mut(Self::index(local))?;
        let idx = cell.iter().position(|&e| e == entity)?;
        cell.swap_remove(idx);
        Some(())
    }
}
impl Default for ChunkEntityMatrix {
    fn default() -> Self {
        let cells = std::array::from_fn(|_| ReturnedVec::new());
        Self { cells }
    }
}
