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

    fn try_insert(&mut self, hash_id: HashId, entity: Entity, prefix: &EntityPrefix, id_display: impl std::fmt::Debug) -> Result {
        if self.0.contains_key(&hash_id) {
            return Err(BevyError::from(format!(
                "Failed to insert {} {} {:?} into map, already present", prefix, entity, id_display
            )));
        }
        self.0.insert(hash_id, entity);
        Ok(())
    }

    pub fn insert<K>(&mut self, id: K, entity: Entity, prefix: &EntityPrefix) -> Result
    where
        K: AsRef<str>,
    {
        let hash_id = HashId::from(id.as_ref());
        self.try_insert(hash_id, entity, prefix, id.as_ref())
    }

    pub fn insert_with_hash(&mut self, hash_id: &HashId, entity: Entity, prefix: &EntityPrefix) -> Result {
        self.try_insert(*hash_id, entity, prefix, hash_id)
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
    pub fn get_multiple<K>(&self, ids: &[K]) -> Result<Vec<Entity>, BevyError>
    where
        K: AsRef<str>,
    {
        let mut entities = Vec::with_capacity(ids.len());
        for id in ids {
            entities.push(self.get(id)?);
        }
        Ok(entities)
    }

    pub fn get_with_hash(&self, hash_id: &HashId) -> Result<Entity, BevyError> {
        self.0.get(hash_id).copied().ok_or_else(|| {
            BevyError::from(format!("Entity with hash id {:?} not found", hash_id))
        })
    }
    pub fn get_multiple_with_hash(&self, hash_ids: &[HashId]) -> Result<Vec<Entity>, BevyError> {
        let mut entities = Vec::with_capacity(hash_ids.len());
        for hash_id in hash_ids {
            entities.push(self.get_with_hash(hash_id)?);
        }
        Ok(entities)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&HashId, &Entity)> {
        self.0.iter()
    }
}
