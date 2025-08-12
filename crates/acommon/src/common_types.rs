use bevy::platform::collections::HashMap;
#[allow(unused_imports)] use bevy::prelude::*;
use crate::{
    common_components::*,
//    common_resources::*,
//    common_constants::*,
//    common_layout::*,
//    common_events::*,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Deserialize, Serialize, Reflect)]
pub struct HashIdToEntityMap(pub HashMap<HashId, Entity>);

impl HashIdToEntityMap {
    pub fn new() -> Self { Self(HashMap::new()) }

    fn try_insert(&mut self, hash_id: HashId, entity: Entity, ) -> Result {
        if self.0.contains_key(&hash_id) {
            return Err(BevyError::from(format!(
                "Failed to insert {} into map, already present", entity,
            )));
        }
        self.0.insert(hash_id, entity);
        Ok(())
    }

    pub fn insert<K>(&mut self, id: K, entity: Entity, ) -> Result
    where
        K: AsRef<str>,
    {
        let hash_id = HashId::from(id.as_ref());
        self.try_insert(hash_id, entity, )
    }

    pub fn insert_with_hash(&mut self, hash_id: HashId, entity: Entity, ) -> Result {
        self.try_insert(hash_id, entity, )
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
    pub fn clear(&mut self) { self.0.clear(); }
}

#[derive(Clone, PartialEq, Eq, Hash, Reflect)]
pub struct FixedStr<const N: usize>([u8; N]);

impl<const N: usize> FixedStr<N> {
    pub fn new<S: AsRef<str>>(s: S) -> Self {
        let bytes = s.as_ref().as_bytes();
        let mut arr = [0u8; N];
        let len = bytes.len().min(N);
        arr[..len].copy_from_slice(&bytes[..len]);
        Self(arr)
    }
    pub fn new_with_result<S: AsRef<str>>(s: S) -> Result<Self, BevyError> {
        if s.as_ref().len() > N {
            return Err(BevyError::from(format!(
                "String too long for FixedStr<{}>: '{}'",
                N, s.as_ref()
            )));
        }
        Ok(Self::new(s))
    }
    pub fn is_empty(&self) -> bool { self.0.iter().all(|&b| b == 0) }
    pub fn as_str(&self) -> &str {
        let nul_pos = self.0.iter().position(|&b| b == 0).unwrap_or(N);
        std::str::from_utf8(&self.0[..nul_pos]).unwrap_or("")
    }
}
impl<const N: usize> Default for FixedStr<N> {fn default() -> Self { Self([0u8; N]) } }
impl<const N: usize> std::fmt::Display for FixedStr<N> { #[inline(always)] fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result { std::fmt::Display::fmt(self.as_str(), f) } }
impl<const N: usize> std::fmt::Debug for FixedStr<N> { #[inline(always)] fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result { write!(f, "{}", self.as_str()) } }
impl<const N: usize> serde::Serialize for FixedStr<N> { fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: serde::Serializer, { serializer.serialize_str(self.as_str()) } }
impl<'de, const N: usize> serde::Deserialize<'de> for FixedStr<N> { fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: serde::Deserializer<'de>, { let s = <&str>::deserialize(deserializer)?; Ok(FixedStr::new(s)) } }
impl<const N: usize> From<&str> for FixedStr<N> { fn from(s: &str) -> Self { FixedStr::new(s) } }
impl<const N: usize> From<String> for FixedStr<N> { fn from(s: String) -> Self { FixedStr::new(s) } }
impl<const N: usize> AsRef<str> for FixedStr<N> { fn as_ref(&self) -> &str { self.as_str() } }