use bevy::prelude::*;
use smallvec::SmallVec;
use crate::tile::U16TileIndex;
use ::tilemap_shared::*;
use serde::{Serialize, Deserialize};

pub use ::tilemap_shared::{
	BiomeDistribution,
	BiomePackCountAvgedNormDists,
	BiomeTagWeightAtMacrochunk,
	MacrochunkPendingBiomeSamples,
};

#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct MacroChunkU16IndexMatrix {
	cells: Vec<Self::Cell>,
	occupied_bounds: Option<(UVec2, UVec2)>,
}

impl Default for MacroChunkU16IndexMatrix {
	fn default() -> Self {
		let cells: Vec<Self::Cell> = std::iter::repeat_with(|| SmallVec::new())
			.take(Self::CELL_COUNT)
			.collect();
		Self {
			cells,
			occupied_bounds: None,
		}
	}
}

impl MacroChunkU16IndexMatrix {
	pub type Cell = SmallVec<[U16TileIndex; Self::INLINE_CAP]>;
	const INLINE_CAP: usize = 4;//min 4(pointer size)
	const WIDTH: usize = (MacrochunkPos::SIZE_IN_CHUNKS.0.x * ChunkPos::CHUNK_SIZE.x as i32) as usize;
	const HEIGHT: usize = (MacrochunkPos::SIZE_IN_CHUNKS.0.y * ChunkPos::CHUNK_SIZE.y as i32) as usize;
	const CELL_COUNT: usize = Self::WIDTH * Self::HEIGHT;

	fn local_from_gpos(anchor_gpos: GlobalTilePos, gpos: GlobalTilePos) -> Option<UVec2> {
		let local = gpos.0 - anchor_gpos.0;
		if local.x < 0 || local.y < 0 {
			return None;
		}
		let local = local.as_uvec2();
		if local.x as usize >= Self::WIDTH || local.y as usize >= Self::HEIGHT {
			return None;
		}
		Some(local)
	}

	fn flat_index_from_local(local: UVec2) -> usize {
		(local.y as usize * Self::WIDTH) + local.x as usize
	}

	fn flat_index_from_gpos(&self, anchor_gpos: GlobalTilePos, gpos: GlobalTilePos) -> Option<usize> {
		let local = Self::local_from_gpos(anchor_gpos, gpos)?;
		Some(Self::flat_index_from_local(local))
	}

	fn expand_bounds_for_local(&mut self, local: UVec2) {
		self.occupied_bounds = Some(match self.occupied_bounds {
			Some((min, max)) => (min.min(local), max.max(local)),
			None => (local, local),
		});
	}

	fn recompute_occupied_bounds(&mut self) {
		let mut found_any = false;
		let mut min = UVec2::splat(u32::MAX);
		let mut max = UVec2::ZERO;
		for y in 0..Self::HEIGHT {
			for x in 0..Self::WIDTH {
				let local = UVec2::new(x as u32, y as u32);
				if self.cells[Self::flat_index_from_local(local)].is_empty() {
					continue;
				}
				found_any = true;
				min = min.min(local);
				max = max.max(local);
			}
		}
		self.occupied_bounds = found_any.then_some((min, max));
	}

	pub fn tile_indices_at_gpos(&self, anchor_gpos: GlobalTilePos, gpos: GlobalTilePos) -> Option<&Self::Cell> {
		let i = self.flat_index_from_gpos(anchor_gpos, gpos)?;
		self.cells.get(i)
	}

	pub fn tile_indices_at_gpos_mut(&mut self, anchor_gpos: GlobalTilePos, gpos: GlobalTilePos) -> Option<&mut Self::Cell> {
		let i = self.flat_index_from_gpos(anchor_gpos, gpos)?;
		self.cells.get_mut(i)
	}

	pub fn push_tile_index(&mut self, anchor_gpos: GlobalTilePos, gpos: GlobalTilePos, tile_index: U16TileIndex) -> bool {
		let Some(local) = Self::local_from_gpos(anchor_gpos, gpos) else {
			return false;
		};
		let i = Self::flat_index_from_local(local);
		let Some(cell) = self.cells.get_mut(i) else {
			return false;
		};
		let was_empty = cell.is_empty();
		if !cell.contains(&tile_index) {
			cell.push(tile_index);
			if was_empty {
				self.expand_bounds_for_local(local);
			}
		}
		true
	}
	pub fn remove_tile_index(&mut self, anchor_gpos: GlobalTilePos, gpos: GlobalTilePos, tile_index: U16TileIndex) -> bool {
		let Some(local) = Self::local_from_gpos(anchor_gpos, gpos) else {
			return false;
		};
		let i = Self::flat_index_from_local(local);
		let Some(cell) = self.cells.get_mut(i) else {
			return false;
		};
		let prev_bounds = self.occupied_bounds;
		let Some(pos) = cell.iter().position(|&t| t == tile_index) else {
			return false;
		};
		cell.remove(pos);
		if cell.is_empty()
			&& prev_bounds.is_some_and(|(min, max)| {
				local.x == min.x || local.x == max.x || local.y == min.y || local.y == max.y
			})
		{
			self.recompute_occupied_bounds();
		}
		true
	}

	pub fn occupied_bounds(&self) -> Option<(UVec2, UVec2)> {
		self.occupied_bounds
	}

	pub fn occupied_bounds_at_gpos(&self, anchor_gpos: GlobalTilePos) -> Option<(GlobalTilePos, GlobalTilePos)> {
		let (min, max) = self.occupied_bounds?;
		Some((
			GlobalTilePos(anchor_gpos.0 + min.as_ivec2()),
			GlobalTilePos(anchor_gpos.0 + max.as_ivec2()),
		))
	}
}
