
use bevy::prelude::*;
use indexmap::IndexMap;
#[allow(unused_imports)] 
use serde::{Deserialize, Serialize};
use bevy::platform::collections::HashMap;
use std::hash::{Hash, Hasher};
use crate::{common_states::*, common_types::*};
use std::fmt::{Debug, Display};

pub use crate::common_id_components::*;

pub type SessionScoped = StateScoped::<AppState>;

pub type AssetScoped = StateScoped::<LocallyLoadedAssetsSession>;

//pub type RepliAssetScoped = StateScoped::<ReplicatedAssetsSession>;

pub type TgenHotLoadingScoped = StateScoped::<TerrainGenHotLoading>;


#[derive(Component, Clone, Default, Serialize, Deserialize, Reflect)]
pub struct DisplayName(pub String);

impl DisplayName {
    pub fn new<S: AsRef<str>>(name: S) -> Self {
        DisplayName(name.as_ref().to_string())
    }

    pub fn insert_name_if_non_empty<S: AsRef<str>>(name: S, entity: &mut EntityCommands) {
        let name_str = name.as_ref();
        if !name_str.is_empty() {
            entity.insert(DisplayName(name_str.to_string()));
        }
    }
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

    pub fn new<S>(asset_server: &AssetServer, path: S) -> Result<Self, BevyError>
    where
        S: AsRef<str> + Into<bevy::asset::AssetPath<'static>>,
    {
        let img_path = format!("assets/{}", path.as_ref());
        if !std::path::Path::new(&img_path).exists() {
            let err = BevyError::from(format!("Image path does not exist: {}", img_path));
            error!(target: "image_loading", "{}", err);
            return Err(err);
        }
        Ok(Self(asset_server.load(path)))
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
            let image_holder = ImageHolder::new(asset_server, path)?;
            map.insert(key, image_holder.0);
        }
        Ok(Self(map))
    }
    pub fn first_handle(&self) -> Handle<Image> {
        self.0.first().cloned().unwrap_or_else(|| Handle::default())
    }
   
}



// pub type EntityWeightedMap = WeightedMap<Entity>;

// #[derive(Debug, Component)]
// pub struct WeightedMap<K> {
//     weights: Vec<u32>, choices: Vec<K>,
//     dist: WeightedAliasIndex<u32>,
// }
// #[allow(unused_parens, dead_code)]
// impl<K: Eq + std::hash::Hash + Clone + Serialize + for<'de> Deserialize<'de>> WeightedMap<K> {
//     pub fn new(weights_map: HashMap<K, u32>) -> Self {
//         let weights: Vec<u32> = weights_map.values().cloned().collect();
//         let choices: Vec<K> = weights_map.keys().cloned().collect();
//         let dist = WeightedAliasIndex::new(weights.clone()).unwrap();
//         Self {weights, choices, dist,}
//     }
//     pub fn rand_weighted<R: Rng>(&self, rng: &mut R) -> Option<&K> {
//         let index = self.dist.sample(rng) as usize;
//         self.choices.get(index)
//     }

//     pub fn choices(&self) -> &Vec<K> {&self.choices}
// }
// impl<'de, K: Eq + std::hash::Hash + Clone + Serialize + Deserialize<'de>> Deserialize<'de> for WeightedMap<K> {
//     fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
//     where D: serde::Deserializer<'de>,
//     {
//         #[derive(Deserialize)]
//         struct Helper<K> { weights: Vec<u32>, choices: Vec<K> }
//         let Helper { weights, choices } = Helper::deserialize(deserializer)?;
//         let dist = WeightedAliasIndex::new(weights.clone()).map_err(serde::de::Error::custom)?;
//         Ok(WeightedMap { weights, choices, dist })
//     }
// }
// impl<K: Eq + std::hash::Hash + Clone + Serialize> Serialize for WeightedMap<K> {
//     fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
//     where S: serde::Serializer,
//     {
//         #[derive(Serialize)]
//         struct Helper<'a, K> { weights: &'a Vec<u32>, choices: &'a Vec<K> }
//         let helper = Helper { weights: &self.weights, choices: &self.choices };
//         helper.serialize(serializer)
//     }
// }



