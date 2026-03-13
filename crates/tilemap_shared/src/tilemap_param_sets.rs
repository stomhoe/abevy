use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

use crate::{DimensionRef, GlobalTilePos, HashIdToTexIndex, LoadedChunks, SpriteTilesAtGpos, Tilemaps};

#[derive(SystemParam)]
#[allow(unused_parens, )]
/// system which uses this must be put .in_set(PreChunkDespawnReaders)
pub struct TileGatheringParamSet<'w, 's> {
    spritetiles_at_gpos: Res<'w, SpriteTilesAtGpos>,
    loaded_chunks: Res<'w, LoadedChunks>,
    chunk_children: Query<'w, 's, &'static Tilemaps>,
    pub tilemap_query: Query<'w, 's, (&'static mut TileStorage, &'static HashIdToTexIndex),>,
    to_drain: Local<'s, Vec<Entity>>,
}
impl<'w, 's> TileGatheringParamSet<'w, 's> {
    pub fn gather_tiles_at(&self, collected_entis: &mut impl Extend<Entity>, dim: DimensionRef, gpos: GlobalTilePos) {
        let chunk_pos = gpos.to_chunkpos();
        collected_entis.extend(self.spritetiles_at_gpos.tiles_at_pos(dim, gpos).iter().copied());
        let Some(&chunk_ent) = self.loaded_chunks.0.get(&(dim, chunk_pos)) else {
            return;
        };
        if let Ok(tilemaps) = self.chunk_children.get(chunk_ent) {
            for &tmap_ent in tilemaps.entities() {
                let Ok((storage, ..)) = self.tilemap_query.get(tmap_ent) else {
                    continue;
                };
                let tpos = gpos.to_tilepos();
                if let Some(tile_ent) = storage.get(&tpos) {
                    collected_entis.extend(std::iter::once(tile_ent));
                }
            }
        }
    }

    pub fn gather_tiles_at_to_drain(&mut self, dim: DimensionRef, gpos: GlobalTilePos) -> &[Entity] {
        let to_drain: &mut Vec<Entity> = self.to_drain.as_mut();
        to_drain.clear();
        let chunk_pos = gpos.to_chunkpos();
        to_drain.extend(self.spritetiles_at_gpos.tiles_at_pos(dim, gpos).iter().copied());
        let Some(&chunk_ent) = self.loaded_chunks.0.get(&(dim, chunk_pos)) else {
            return &self.to_drain;
        };
        if let Ok(tilemaps) = self.chunk_children.get(chunk_ent) {
            for &tmap_ent in tilemaps.entities() {
                let Ok((storage, ..)) = self.tilemap_query.get(tmap_ent) else {
                    continue;
                };
                let tpos = gpos.to_tilepos();
                if let Some(tile_ent) = storage.get(&tpos) {
                    to_drain.push(tile_ent);
                }
            }
        }
        &self.to_drain
    }
}
