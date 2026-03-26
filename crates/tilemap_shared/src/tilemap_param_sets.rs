use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

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
    pub to_drain: Local<'s, Vec<Entity>>,
}
impl<'w, 's> TileGatheringParamSet<'w, 's> {
    pub fn cardinal_direction_query(
        &mut self,
    ) -> &mut Query<'w, 's, &'static mut CardinalDirection, ()> {
        &mut self.cardinal_direction_query
    }

    pub fn drain_gathered(&mut self) -> impl Iterator<Item = Entity> + '_ {
        self.to_drain.drain(..)
    }

    pub fn gather_tiles(&mut self, dim: DimensionRef, gpos: GlobalTilePos) -> &[Entity] {
        self.to_drain.clear();
        self.gather_tiles_to_drain(dim, gpos);
        self.to_drain.as_slice()
    }

    pub fn gather_tiles_extend(&self, collected_entis: &mut impl Extend<Entity>, dim: DimensionRef, gpos: GlobalTilePos) {
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
    pub fn gather_tiles_to_drain(&mut self, dim: DimensionRef, gpos: GlobalTilePos) {
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

    pub fn get_being_direction(&mut self, being: Entity) -> Option<CardinalDirection> {
        self.cardinal_direction_query.get_mut(being).ok().map(|direction| *direction)
    }

    pub fn set_being_direction(&mut self, being: Entity, direction: CardinalDirection) -> bool {
        let Ok(mut current_direction) = self.cardinal_direction_query.get_mut(being) else {
            return false;
        };
        *current_direction = direction;
        true
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
