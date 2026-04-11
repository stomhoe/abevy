use bevy::prelude::*;
use common::common_components::HashId;
use common::common_types::HashIdToEntityMap;

#[derive(Resource, Debug, Clone, Default)]
pub struct BeingEntityMap(pub HashIdToEntityMap);

impl BeingEntityMap {
    pub fn insert(&mut self, hash_id: HashId, entity: Entity) -> Option<Entity> {
        self.0.overwrite(hash_id, entity)
    }

    pub fn remove(&mut self, hash_id: HashId) -> Option<Entity> {
        self.0.remove(hash_id)
    }
}
