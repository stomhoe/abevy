use bevy::{
    ecs::entity::{EntityHashMap, EntityHashSet},
    platform::collections::{HashMap, HashSet},
    prelude::*,
};
use tilemap_shared::{ChunkPos, DimensionRef, MacroChunkPos};

use crate::wildlife_spawning_systems::watched_chunk_keys;

#[derive(Resource, Default)]
pub struct NaturalSpawnReservationIndex {
    pub by_chunk: HashMap<(DimensionRef, ChunkPos), EntityHashSet>,
    pub reservation_by_being: EntityHashMap<(DimensionRef, ChunkPos)>,
}

#[derive(Resource, Default)]
pub struct SeededNaturalWildlifeMacroChunks(pub HashSet<(DimensionRef, MacroChunkPos)>);

impl NaturalSpawnReservationIndex {
    pub fn insert(&mut self, being_ent: Entity, dim_ref: DimensionRef, home_chunk: ChunkPos) {
        self.remove_being(being_ent);
        for key in watched_chunk_keys(dim_ref, home_chunk) {
            self.by_chunk.entry(key).or_default().insert(being_ent);
        }
        self.reservation_by_being.insert(being_ent, (dim_ref, home_chunk));
    }

    pub fn remove_being(&mut self, being_ent: Entity) {
        let Some((dim_ref, home_chunk)) = self.reservation_by_being.remove(&being_ent) else {
            return;
        };
        for key in watched_chunk_keys(dim_ref, home_chunk) {
            let should_remove = {
                let Some(being_ents) = self.by_chunk.get_mut(&key) else {
                    continue;
                };
                being_ents.remove(&being_ent);
                being_ents.is_empty()
            };
            if should_remove {
                self.by_chunk.remove(&key);
            }
        }
    }

    pub fn remove_being_except_chunk(
        &mut self,
        being_ent: Entity,
        skip_key: (DimensionRef, ChunkPos),
    ) {
        let Some((dim_ref, home_chunk)) = self.reservation_by_being.remove(&being_ent) else {
            return;
        };
        for key in watched_chunk_keys(dim_ref, home_chunk) {
            if key == skip_key {
                continue;
            }
            let should_remove = {
                let Some(being_ents) = self.by_chunk.get_mut(&key) else {
                    continue;
                };
                being_ents.remove(&being_ent);
                being_ents.is_empty()
            };
            if should_remove {
                self.by_chunk.remove(&key);
            }
        }
    }
}
