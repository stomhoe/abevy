use bevy::prelude::*;
use smallvec::SmallVec;
use crate::tile::TileIndex;
use ::tilemap_shared::*;
use serde::{Serialize, Deserialize};

pub use ::tilemap_shared::{
	BiomeDistribution,
	BiomePackCountAvgedNormDists,
	BiomeTagWeightAtMacroChunk,
	MacroChunkBiomePendingSampleState,
};

#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct MacroChunkTileIndices {
	cells: Vec<Self::Cell>,
}

impl Default for MacroChunkTileIndices {
	fn default() -> Self {
		let cells: Vec<Self::Cell> = std::iter::repeat_with(|| SmallVec::new())
			.take(Self::CELL_COUNT)
			.collect();
		Self { cells }
	}
}

impl MacroChunkTileIndices {
	pub type Cell = SmallVec<[TileIndex; Self::INLINE_CAP]>;
	const INLINE_CAP: usize = 4;//min 4(pointer size)
	const WIDTH: usize = (MacroChunkPos::SIZE_IN_CHUNKS.0.x * ChunkPos::CHUNK_SIZE.x as i32) as usize;
	const HEIGHT: usize = (MacroChunkPos::SIZE_IN_CHUNKS.0.y * ChunkPos::CHUNK_SIZE.y as i32) as usize;
	const CELL_COUNT: usize = Self::WIDTH * Self::HEIGHT;

	fn flat_index_from_gpos(&self, anchor_gpos: GlobalTilePos, gpos: GlobalTilePos) -> Option<usize> {
		let local = gpos.0 - anchor_gpos.0;
		if local.x < 0 || local.y < 0 {
			return None;
		}
		let x = local.x as usize;
		let y = local.y as usize;
		if x >= Self::WIDTH || y >= Self::HEIGHT {
			return None;
		}
		Some(y * Self::WIDTH + x)
	}

	pub fn tile_indices_at_gpos(&self, anchor_gpos: GlobalTilePos, gpos: GlobalTilePos) -> Option<&Self::Cell> {
		let i = self.flat_index_from_gpos(anchor_gpos, gpos)?;
		self.cells.get(i)
	}

	pub fn tile_indices_at_gpos_mut(&mut self, anchor_gpos: GlobalTilePos, gpos: GlobalTilePos) -> Option<&mut Self::Cell> {
		let i = self.flat_index_from_gpos(anchor_gpos, gpos)?;
		self.cells.get_mut(i)
	}

	pub fn push_tile_index(&mut self, anchor_gpos: GlobalTilePos, gpos: GlobalTilePos, tile_index: TileIndex) -> bool {
		let Some(cell) = self.tile_indices_at_gpos_mut(anchor_gpos, gpos) else {
			return false;
		};
		if !cell.contains(&tile_index) {
			cell.push(tile_index);
		}
		true
	}
	pub fn remove_tile_index(&mut self, anchor_gpos: GlobalTilePos, gpos: GlobalTilePos, tile_index: TileIndex) -> bool {
		let Some(cell) = self.tile_indices_at_gpos_mut(anchor_gpos, gpos) else {
			return false;
		};
		let Some(pos) = cell.iter().position(|&t| t == tile_index) else {
			return false;
		};
		cell.remove(pos);
		true
	}
}
