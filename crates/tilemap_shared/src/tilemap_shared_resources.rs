use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use bevy::platform::collections::HashMap;
use common::common_components::HashId;
use smallvec::SmallVec;

use crate::{ChunkPos, DimensionRef, GlobalTilePos, HashIdToTexIndex, SizeInTiles, Tilemaps};

#[derive(Resource, Default)]
pub struct LoadedChunks (pub HashMap<(DimensionRef, ChunkPos), Entity>,);

#[derive(Debug, Clone, Default)]
pub struct BiomeTagDistribution {
    pub sums: HashMap<HashId, f32>,
    pub total_sum: f32,
}
impl BiomeTagDistribution {
    pub fn add_weight(&mut self, tag: HashId, weight: f32) {
        if !weight.is_finite() || weight <= 0.0 {
            return;
        }
        let entry = self.sums.entry(tag).or_insert(0.0);
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
        self.sums.get(&tag).copied().unwrap_or_default()
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
pub struct ChunkBiomeTagDistributionMap(pub HashMap<(DimensionRef, ChunkPos), BiomeTagDistribution>);
impl ChunkBiomeTagDistributionMap {
    pub fn add_tag_weights<I>(&mut self, dim_ref: DimensionRef, chunk_pos: ChunkPos, tag_weights: I)
    where
        I: IntoIterator<Item = (HashId, f32)>,
    {
        let dist = self.0.entry((dim_ref, chunk_pos)).or_default();
        dist.add_tag_weights(tag_weights);
    }
    pub fn predominant_tag(&self, dim_ref: DimensionRef, chunk_pos: ChunkPos) -> Option<HashId> {
        self.0
            .get(&(dim_ref, chunk_pos))
            .and_then(BiomeTagDistribution::predominant_tag)
    }
    pub fn predominant_tags(&self, dim_ref: DimensionRef, chunk_pos: ChunkPos) -> Vec<HashId> {
        self.0
            .get(&(dim_ref, chunk_pos))
            .map(BiomeTagDistribution::predominant_tags)
            .unwrap_or_default()
    }
    pub fn predominant_tags_with_sums(&self, dim_ref: DimensionRef, chunk_pos: ChunkPos) -> Vec<(HashId, f32)> {
        self.0
            .get(&(dim_ref, chunk_pos))
            .map(BiomeTagDistribution::predominant_tags_with_sums)
            .unwrap_or_default()
    }
    pub fn tag_sum(&self, dim_ref: DimensionRef, chunk_pos: ChunkPos, tag: HashId) -> f32 {
        self.0
            .get(&(dim_ref, chunk_pos))
            .map(|dist| dist.tag_sum(tag))
            .unwrap_or_default()
    }
    pub fn tag_percentage(&self, dim_ref: DimensionRef, chunk_pos: ChunkPos, tag: HashId) -> f32 {
        self.0
            .get(&(dim_ref, chunk_pos))
            .map(|dist| dist.tag_percentage(tag))
            .unwrap_or_default()
    }
    pub fn distribution(&self, dim_ref: DimensionRef, chunk_pos: ChunkPos) -> Option<&BiomeTagDistribution> {
        self.0.get(&(dim_ref, chunk_pos))
    }
}

pub type ReturnedVec = SmallVec<[Entity; 16]>;

#[derive(Debug, )]
pub struct ChunkEntityMatrix {
    cells: Box<[ReturnedVec; ChunkPos::CHUNK_AREA]>,
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
        Self {
            cells: Box::new(cells),
        }
    }
}

#[derive(Resource, Debug, Default)]
pub struct SpriteTilesAtGpos  {
    pub map: HashMap<(DimensionRef, ChunkPos), ChunkEntityMatrix>,
}
impl SpriteTilesAtGpos {
    fn chunk_and_local(gpos: GlobalTilePos) -> (ChunkPos, UVec2) {
        let chunk = gpos.to_chunkpos();
        let chunk_origin = chunk.to_tilepos();
        let local = (gpos.0 - chunk_origin.0).as_uvec2();
        (chunk, local)
    }
    pub fn tiles_at_pos(&self, dim_ref: DimensionRef, gpos: GlobalTilePos) -> &[Entity] {
        let (chunk, local) = Self::chunk_and_local(gpos);
        self.map
            .get(&(dim_ref, chunk))
            .map(|matrix| matrix.get(local))
            .unwrap_or(&[])
    }
    pub fn remove_tile(&mut self, dim_ref: DimensionRef, gpos: GlobalTilePos, tile_ent: Entity, size: SizeInTiles) {
        let size = size.inner();
        for y in 0..size.y {
            for x in 0..size.x {
                let curr_gpos = GlobalTilePos(gpos.0 + IVec2::new(x as i32, y as i32));
                let (chunk, local) = Self::chunk_and_local(curr_gpos);
                if let Some(matrix) = self.map.get_mut(&(dim_ref, chunk)) {
                    matrix.swap_remove(local, tile_ent);
                }
            }
        }
    }
    pub fn reserve_capacity(&mut self, additional: usize) {
        self.map.reserve(additional);
    }
    pub fn insert(&mut self, entity: Entity, dimension_ref: DimensionRef, gpos: GlobalTilePos, size: SizeInTiles) {
        let size = size.inner();
        for y in 0..size.y {
            for x in 0..size.x {
                let curr_gpos = GlobalTilePos(gpos.0 + IVec2::new(x as i32, y as i32));
                let (chunk, local) = Self::chunk_and_local(curr_gpos);
                let matrix = self
                    .map
                    .entry((dimension_ref, chunk))
                    .or_insert_with(ChunkEntityMatrix::new);
                matrix.push(local, entity);
            }
        }
    }
}

#[derive(Resource, Debug, Default)]
pub struct BeingsAtGpos {
    pub map: HashMap<(DimensionRef, ChunkPos), ChunkEntityMatrix>,
}
impl BeingsAtGpos {
    fn chunk_and_local(gpos: GlobalTilePos) -> (ChunkPos, UVec2) {
        let chunk = gpos.to_chunkpos();
        let chunk_origin = chunk.to_tilepos();
        let local = (gpos.0 - chunk_origin.0).as_uvec2();
        (chunk, local)
    }
    pub fn beings_at_pos(&self, dim_ref: DimensionRef, gpos: GlobalTilePos) -> &[Entity] {
        let (chunk, local) = Self::chunk_and_local(gpos);
        self.map
            .get(&(dim_ref, chunk))
            .map(|matrix| matrix.get(local))
            .unwrap_or(&[])
    }
    pub fn remove_being(&mut self, dim_ref: DimensionRef, gpos: GlobalTilePos, being_ent: Entity) {
        let (chunk, local) = Self::chunk_and_local(gpos);
        let Some(matrix) = self.map.get_mut(&(dim_ref, chunk)) else {
            return;
        };
        matrix.swap_remove(local, being_ent);
    }
    pub fn insert_being(&mut self, dim_ref: DimensionRef, gpos: GlobalTilePos, being_ent: Entity) {
        let (chunk, local) = Self::chunk_and_local(gpos);
        let matrix = self
            .map
            .entry((dim_ref, chunk))
            .or_insert_with(ChunkEntityMatrix::new);
        matrix.push(local, being_ent);
    }
}

#[derive(SystemParam)]
#[allow(unused_parens, )]
/// system which uses this must be put .in_set(PreChunkDespawnReaders)
pub struct TileGatheringParamSet<'w, 's> {
    spritetiles_at_gpos: Res<'w, SpriteTilesAtGpos>,
    loaded_chunks: Res<'w, LoadedChunks>,
    chunk_children: Query<'w, 's, &'static Tilemaps>,
    pub tilemap_query: Query<'w, 's, (&'static mut TileStorage, &'static HashIdToTexIndex),>,
}
impl<'w, 's> TileGatheringParamSet<'w, 's> {
    pub fn gather_tiles_at(&self, vec_to_drain: &mut impl Extend<Entity>, dim: DimensionRef, gpos: GlobalTilePos) {
        let chunk_pos = gpos.to_chunkpos();
        vec_to_drain.extend(self.spritetiles_at_gpos.tiles_at_pos(dim, gpos).iter().copied());
        let Some(&chunk_ent) = self.loaded_chunks.0.get(&(dim, chunk_pos)) else {
            return;
        };
        if let Ok(tilemaps) = self.chunk_children.get(chunk_ent){
            for &tmap_ent in tilemaps.entities() {
                let Ok((storage, ..)) = self.tilemap_query.get(tmap_ent) else {
                    continue;
                };
                let tpos = gpos.to_tilepos();

                if let Some(tile_ent) = storage.get(&tpos) {
                    vec_to_drain.extend(std::iter::once(tile_ent));
                }
            }
        }
    }
}
