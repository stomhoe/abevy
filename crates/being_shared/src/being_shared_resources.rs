use bevy::{
    ecs::entity::EntityHashSet,
    platform::collections::HashMap,
    prelude::*,
};
use tilemap_shared::{ChunkPos, DimensionRef};

#[derive(Resource, Debug, Default)]
pub struct FrozenBgSimulatedBeingsMap(pub HashMap<(DimensionRef, ChunkPos), Vec<Entity>>);

#[derive(Resource, Debug, Default, Copy, Clone)]
pub struct WallPhaserOnSpawn(pub bool);

#[derive(Resource, Debug, Default, Copy, Clone)]
pub struct InvulnerableOnSpawn(pub bool);

#[derive(Resource, Default)]
pub struct BeingsToEnableOnChunkLoad {
    pub by_chunk: HashMap<(DimensionRef, ChunkPos), EntityHashSet>,
}

impl BeingsToEnableOnChunkLoad {
    pub fn insert(&mut self, being_ent: Entity, dim_ref: DimensionRef, home_chunk: ChunkPos) {
        self.by_chunk.entry((dim_ref, home_chunk)).or_default().insert(being_ent);
    }

    pub fn remove_being(&mut self, being_ent: Entity, dim_ref: DimensionRef, home_chunk: ChunkPos) {
        let key = (dim_ref, home_chunk);
        let should_remove = {
            let Some(being_ents) = self.by_chunk.get_mut(&key) else {
                return;
            };
            being_ents.remove(&being_ent);
            being_ents.is_empty()
        };
        if should_remove {
            self.by_chunk.remove(&key);
        }
    }
}
