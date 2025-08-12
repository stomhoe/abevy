
use bevy::prelude::*;
use indexmap


::IndexMap;
#[allow(unused_imports)] 
use serde::{Deserialize, Serialize};
use bevy::platform::collections::HashMap;
use std::hash::{Hash, Hasher};
use rand_distr::weighted::WeightedAliasIndex;
use rand::prelude::*;
use crate::{common_states::*, common_types::*};

#[derive(Component, Debug, Default, Clone, Hash, PartialEq, Reflect)]
#[require(StateScoped::<AppState>(AppState::StatefulGameSession))]
pub struct SessionScoped;

#[derive(Component, Debug, Default, Clone, Hash, PartialEq, Reflect)]
#[require(StateScoped::<LoadedAssetsSession>(LoadedAssetsSession::KeepAlive), )]
pub struct AssetScoped;

#[derive(Component, Debug, Default, Clone, Hash, PartialEq, Reflect)]
#[require(StateScoped::<TerrainGenHotLoading>(TerrainGenHotLoading::KeepAlive), )]
pub struct TgenScoped;

#[derive(Component, Default, Deserialize, Serialize, Clone, Reflect)]
pub struct EntityPrefix(pub FixedStr<20>);
impl EntityPrefix {
    pub fn new<S: AsRef<str>>(id: S) -> Self { Self(FixedStr::new(id)) }
    pub fn as_str(&self) -> &str { self.0.as_str() }
}
impl core::fmt::Debug for EntityPrefix {
    #[inline(always)]
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "EntityPrefix({})", self.as_str())
    }
}
impl core::fmt::Display for EntityPrefix {
    #[inline(always)]
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        core::fmt::Display::fmt(self.as_str(), f)
    }
}




#[derive(Component, Clone, Default, Serialize, Deserialize, Reflect)]
pub struct DisplayName(pub String);

impl DisplayName {
    pub fn new(name: impl Into<String>) -> Self {DisplayName(name.into())}
}

impl core::fmt::Display for DisplayName {
    #[inline(always)]
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        core::fmt::Display::fmt(&self.0, f)
    }
}
impl core::fmt::Debug for DisplayName {
    #[inline(always)]
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        if self.0.is_empty() {write!(f, "")} else {write!(f, "DN({})", self.0)}
    }
}

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, Hash, PartialEq, Eq, Reflect )]
pub struct StrId(FixedStr<32>);
impl StrId {
    pub fn new<S: AsRef<str>>(id: S) -> Result<Self, BevyError> {
        let s = id.as_ref();
        if s.len() >= 1 {
            FixedStr::new_with_result(s).map(Self)
        } else {
            Err(BevyError::from(format!("StrId '{}' must be at least 1 character long", s)))
        }
    }
    pub fn as_str(&self) -> &str { &self.0.as_str() }
    pub fn is_empty(&self) -> bool { self.0.is_empty() }
}
impl std::fmt::Display for StrId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.0.is_empty() {
            write!(f, "")
        } else {
            write!(f, "Id({})", self.0)
        }
    }
}

impl AsRef<str> for StrId {fn as_ref(&self) -> &str {&self.0.as_str() }}

#[derive(Component, Default, Deserialize, Serialize, Clone, Hash, PartialEq, Eq, Copy, Reflect, )]
pub struct HashId(u64);
impl HashId {}
impl<S: AsRef<str>> From<S> for HashId {
    fn from(id: S) -> Self {
        let s = id.as_ref();
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        use std::hash::{Hash, Hasher};
        s.hash(&mut hasher);
        Self((&hasher).finish())
    }
}

impl std::fmt::Display for HashId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "HId({:05})", self.0 & 0xFFFFF)
    }
}
impl std::fmt::Debug for HashId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "HId({:05})", self.0 & 0xFFFFF)
    }
}


#[derive(Component, Default, Deserialize, Serialize, Clone, Debug)]
pub struct HashIdMap<T>(pub HashMap<HashId, T>);
impl<T> HashIdMap<T> {
    pub fn new() -> Self { Self(HashMap::new()) }
    pub fn insert<S: AsRef<str>>(&mut self, key: S, value: T) -> Option<T> { self.0.insert(HashId::from(key), value) }
    pub fn insert_with_id(&mut self, id: HashId, value: T) -> Option<T> { self.0.insert(id, value) }

    pub fn get<S: AsRef<str>>(&self, key: S) -> Option<&T> { self.0.get(&HashId::from(key)) }
    pub fn get_mut<S: AsRef<str>>(&mut self, key: S) -> Option<&mut T> { self.0.get_mut(&HashId::from(key)) }
    pub fn remove<S: AsRef<str>>(&mut self, key: S) -> Option<T> { self.0.remove(&HashId::from(key)) }
    pub fn contains_key<S: AsRef<str>>(&self, key: S) -> bool { self.0.contains_key(&HashId::from(key)) }
    pub fn len(&self) -> usize { self.0.len() }
    pub fn is_empty(&self) -> bool { self.0.is_empty() }
    pub fn iter(&self) -> impl Iterator<Item = (&HashId, &T)> { self.0.iter() }
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&HashId, &mut T)> { self.0.iter_mut() }
}
use delegate::delegate;

#[derive(Component, Default, Deserialize, Serialize, Clone, Debug)]
pub struct HashIdIndexMap<T>(pub IndexMap<HashId, T>);
impl<T> HashIdIndexMap<T> {
    pub fn new() -> Self { Self(IndexMap::new()) }
    pub fn insert<S: AsRef<str>>(&mut self, key: S, value: T) -> Option<T> { self.0.insert(HashId::from(key), value) }
    pub fn get<S: AsRef<str>>(&self, key: S) -> Option<&T> { self.0.get(&HashId::from(key)) }
    pub fn get_mut<S: AsRef<str>>(&mut self, key: S) -> Option<&mut T> { self.0.get_mut(&HashId::from(key)) }
    pub fn first(&self) -> Option<&T> {
        self.0.values().next()
    }
    pub fn contains_key<S: AsRef<str>>(&self, key: S) -> bool {self.0.contains_key(&HashId::from(key))}
    delegate! {
        to self.0 {
            pub fn iter(&self) -> impl Iterator<Item = (&HashId, &T)>;
            pub fn iter_mut(&mut self) -> impl Iterator<Item = (&HashId, &mut T)>;
            pub fn get_index(&self, i: usize) -> Option<(&HashId, &T)>;
            pub fn get_index_mut(&mut self, i: usize) -> Option<(&HashId, &mut T)>;
            pub fn values(&self) -> impl Iterator<Item = &T>;
            pub fn values_mut(&mut self) -> impl Iterator<Item = &mut T>;
            pub fn len(&self) -> usize;
            pub fn is_empty(&self) -> bool;
            pub fn keys(&self) -> impl Iterator<Item = &HashId>;
            pub fn clear(&mut self);
        }
    }
}




#[derive(Component, Debug, Default, Deserialize, Serialize, Clone)]
pub struct ImagePathHolder(String);
impl ImagePathHolder {
    pub fn new<S: AsRef<str>>(path: S) -> Result<Self, BevyError> {
        let img_path = format!("assets/{}", path.as_ref());
        if !std::path::Path::new(&img_path).exists() {
            let err = BevyError::from(format!("Image path does not exist: {}", img_path));
            error!(target: "image_loading", "{}", err);
            return Err(err);
        }
        Ok(Self(path.as_ref().to_string()))
    }
    pub fn path(&self) -> &str { &self.0 }
}
impl std::fmt::Display for ImagePathHolder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { write!(f, "{}", self.0) }
}
impl From<ImagePathHolder> for bevy::asset::AssetPath<'_> {
    fn from(holder: ImagePathHolder) -> Self { bevy::asset::AssetPath::from(holder.0) }
}


#[derive(Component, Debug, Clone, Default, Hash, PartialEq, Eq, Reflect)]
pub struct ImageHolder(pub Handle<Image>);
impl ImageHolder {

    pub fn new<S: AsRef<str>>(asset_server: &AssetServer, path: S) -> Result<Self, BevyError> {
        let img_path = format!("assets/{}", path.as_ref());
        if !std::path::Path::new(&img_path).exists() {
            let err = BevyError::from(format!("Image path does not exist: {}", img_path));
            error!(target: "image_loading", "{}", err);
            return Err(err);
        }
        Ok(Self(asset_server.load(path.as_ref())))
    }
}


#[derive(Component, Debug, Clone, Default, )]
pub struct ImageHolderMap(pub HashIdIndexMap<Handle<Image>>);
impl ImageHolderMap {
    pub fn from_paths(
        asset_server: &AssetServer, 
        img_paths: HashMap<String, String>, 
    ) -> Result<Self, BevyError> {
        let mut map = HashIdIndexMap::default();
        for (key, path) in img_paths {
            let image_holder = ImageHolder::new(asset_server, &path)?;
            map.insert(key, image_holder.0);
        }
        Ok(Self(map))
    }
    pub fn first_handle(&self) -> Handle<Image> {
        self.0.first().cloned().unwrap_or_else(|| Handle::default())
    }
   
}



pub type EntityWeightedMap = WeightedMap<Entity>;

#[derive(Debug, Component)]
pub struct WeightedMap<K> {
    weights: Vec<u32>, choices: Vec<K>,
    dist: WeightedAliasIndex<u32>,
}
#[allow(unused_parens, dead_code)]
impl<K: Eq + std::hash::Hash + Clone + Serialize + for<'de> Deserialize<'de>> WeightedMap<K> {
    pub fn new(weights_map: HashMap<K, u32>) -> Self {
        let weights: Vec<u32> = weights_map.values().cloned().collect();
        let choices: Vec<K> = weights_map.keys().cloned().collect();
        let dist = WeightedAliasIndex::new(weights.clone()).unwrap();
        Self {weights, choices, dist,}
    }
    pub fn rand_weighted<R: Rng>(&self, rng: &mut R) -> Option<&K> {
        let index = self.dist.sample(rng) as usize;
        self.choices.get(index)
    }

    pub fn choices(&self) -> &Vec<K> {&self.choices}
}
impl<'de, K: Eq + std::hash::Hash + Clone + Serialize + Deserialize<'de>> Deserialize<'de> for WeightedMap<K> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Helper<K> { weights: Vec<u32>, choices: Vec<K> }
        let Helper { weights, choices } = Helper::deserialize(deserializer)?;
        let dist = WeightedAliasIndex::new(weights.clone()).map_err(serde::de::Error::custom)?;
        Ok(WeightedMap { weights, choices, dist })
    }
}
impl<K: Eq + std::hash::Hash + Clone + Serialize> Serialize for WeightedMap<K> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where S: serde::Serializer,
    {
        #[derive(Serialize)]
        struct Helper<'a, K> { weights: &'a Vec<u32>, choices: &'a Vec<K> }
        let helper = Helper { weights: &self.weights, choices: &self.choices };
        helper.serialize(serializer)
    }
}



