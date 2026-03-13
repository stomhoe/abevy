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
}
