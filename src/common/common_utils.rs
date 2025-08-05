use bevy::platform::collections::HashMap;
#[allow(unused_imports)] use bevy::prelude::*;
use crate::{common::{
    common_components::*,
//    common_resources::*,
//    common_constants::*,
//    common_layout::*,
//    common_events::*,
}};

#[derive(Debug, Default, Clone, serde::Deserialize, serde::Serialize, )]
pub struct HashIdToEntityMap(pub HashMap<HashId, Entity>);
impl HashIdToEntityMap {
    pub fn new() -> Self { Self(HashMap::new()) }

    pub fn insert<K>(&mut self, id: K, entity: Entity, prefix: &EntityPrefix) -> Result
    where
        K: AsRef<str>,
    {
        let hash_id = HashId::from(id.as_ref());
        if self.0.contains_key(&hash_id) {
            return Err(BevyError::from(format!(
                "Failed to insert {} {} {} into map, already present", prefix, entity, id.as_ref()
            )));
        }
        self.0.insert(hash_id, entity);
        Ok(())
    }

    pub fn get<K>(&self, id: &K) -> Result<Entity, BevyError>
    where
        K: AsRef<str>,
    {
        let hash_id: HashId = HashId::from(id.as_ref());
        self.0.get(&hash_id).copied().ok_or_else(|| {
            BevyError::from(format!("Entity with id {} not found", id.as_ref()))
        })
    }

    pub fn get_with_hash(&self, hash_id: &HashId) -> Result<Entity, BevyError> {
        self.0.get(hash_id).copied().ok_or_else(|| {
            BevyError::from(format!("Entity with hash id {:?} not found", hash_id))
        })
    }

}
