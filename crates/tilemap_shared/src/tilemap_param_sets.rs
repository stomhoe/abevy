use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use common::{common_components::Tag, common_tag_components::TagSet};

use crate::{
    CardinalDirection,
    DimensionRef,
    GlobalTilePos,
    HashIdToTexIndex,
    InteractionZones,
    LoadedChunks,
    SpriteTilesAtGpos,
    Tilemaps,
};

#[derive(SystemParam)]
#[allow(unused_parens, )]
/// system which uses this must be put .in_set(PreChunkDespawnReaders)
pub struct TileGatheringParamSet<'w, 's> {
    spritetiles_at_gpos: ResMut<'w, SpriteTilesAtGpos>,
    loaded_chunks: Res<'w, LoadedChunks>,
    chunk_children: Query<'w, 's, &'static Tilemaps>,
    pub cardinal_direction_query: Query<'w, 's, &'static mut CardinalDirection, ()>,
    pub tilemap_query: Query<'w, 's, (&'static mut TileStorage, &'static mut HashIdToTexIndex, &'static mut TilemapTexture),>,
    tile_tags: Query<'w, 's, &'static TagSet>,
    pub to_drain: Local<'s, Vec<Entity>>,
}
impl<'w, 's> TileGatheringParamSet<'w, 's> {
    pub fn drain_tiles_to_drain(&mut self) -> impl Iterator<Item = Entity> + '_ {
        self.to_drain.drain(..)
    }

    pub fn gather_tiles_at(&mut self, dim: DimensionRef, gpos: GlobalTilePos) -> &[Entity] {
        self.to_drain.clear();
        self.gather_tiles_at_to_drain(dim, gpos);
        self.to_drain.as_slice()
    }

    pub fn gather_tiles_at_extend(&self, collected_entis: &mut impl Extend<Entity>, dim: DimensionRef, gpos: GlobalTilePos) {
        let chunk_pos = gpos.to_chunkpos();
        collected_entis.extend(self.spritetiles_at_gpos.tiles_at_pos(dim, gpos).iter().copied());
        let Some(&chunk_ent) = self.loaded_chunks.0.get(&(dim, chunk_pos)) else {
            return;
        };
        if let Ok(tilemaps) = self.chunk_children.get(chunk_ent) {
            for tmap_ent in tilemaps.iter() {
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

    pub fn has_tile_at(&self, dim: DimensionRef, gpos: GlobalTilePos, tag: Tag) -> bool {
        for &tile_ent in self.spritetiles_at_gpos.tiles_at_pos(dim, gpos) {
            let Ok(tile_tags) = self.tile_tags.get(tile_ent) else {
                continue;
            };
            if tile_tags.contains(tag.clone()) {
                return true;
            }
        }
        let chunk_pos = gpos.to_chunkpos();
        let Some(&chunk_ent) = self.loaded_chunks.0.get(&(dim, chunk_pos)) else {
            return false;
        };
        let Ok(tilemaps) = self.chunk_children.get(chunk_ent) else {
            return false;
        };
        let tpos = gpos.to_tilepos();
        for tmap_ent in tilemaps.iter() {
            let Ok((storage, ..)) = self.tilemap_query.get(tmap_ent) else {
                continue;
            };
            let Some(tile_ent) = storage.get(&tpos) else {
                continue;
            };
            let Ok(tile_tags) = self.tile_tags.get(tile_ent) else {
                continue;
            };
            if tile_tags.contains(tag.clone()) {
                return true;
            }
        }
        false
    }

    pub fn gather_tiles_at_to_drain(&mut self, dim: DimensionRef, gpos: GlobalTilePos) {
        let chunk_pos = gpos.to_chunkpos();
        self.to_drain.extend(self.spritetiles_at_gpos.tiles_at_pos(dim, gpos).iter().copied());
        let Some(&chunk_ent) = self.loaded_chunks.0.get(&(dim, chunk_pos)) else {
            return;
        };
        if let Ok(tilemaps) = self.chunk_children.get(chunk_ent) {
            for tmap_ent in tilemaps.iter() {
                let Ok((storage, ..)) = self.tilemap_query.get(tmap_ent) else {
                    continue;
                };
                let tpos = gpos.to_tilepos();
                if let Some(tile_ent) = storage.get(&tpos) {
                    self.to_drain.push(tile_ent);
                }
            }
        }
    }

    pub fn insert_spritetile(
        &mut self,
        tile_ent: Entity,
        dim: DimensionRef,
        gpos: GlobalTilePos,
        interaction_zones: Option<&InteractionZones>,
    ) {
        self.spritetiles_at_gpos.insert(tile_ent, dim, gpos, interaction_zones);
    }
}
