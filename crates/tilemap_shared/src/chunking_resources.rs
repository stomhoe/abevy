use bevy::platform::collections::{HashMap, HashSet};
use bevy::prelude::*;
use smallvec::SmallVec;

use crate::*;

#[derive(Resource, Default)]
pub struct LoadedChunks (pub HashMap<(DimensionRef, ChunkPos), Entity>,);

#[derive(Resource, Debug, Default)]
pub struct DiscoveredMacroChunks(pub HashSet<(DimensionRef, MacroChunkPos)>);

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
