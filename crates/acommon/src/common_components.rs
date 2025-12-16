
use bevy::prelude::*;
use indexmap::IndexMap;
#[allow(unused_imports)] 
use serde::{Deserialize, Serialize};
use bevy::platform::collections::HashMap;
use std::hash::{Hash, Hasher};
use crate::{common_states::*, common_types::*};
use std::fmt::{Debug, Display};

pub use crate::common_id_components::*;

pub type SessionScoped = DespawnOnExit::<AppState>;

///used for hot reloading assets
pub type AssetScoped = DespawnOnExit::<AssetsLoadingState>;

pub type TgenHotLoadingScoped = DespawnOnExit::<TerrainHotReloading>;


#[derive(Component, Clone, Default, Serialize, Deserialize, Reflect)]
pub struct DisplayName(pub String);

impl DisplayName {
    pub fn new<S: AsRef<str>>(name: S) -> Self {
        DisplayName(name.as_ref().to_string())
    }
    pub fn new_trimmed<S: AsRef<str>>(name: S) -> Self {
        DisplayName(name.as_ref().trim().to_string())
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




#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, Reflect)]
pub struct ImagePathHolder(bevy::asset::AssetPath<'static>);

impl ImagePathHolder {
    pub fn new<S>(path: S) -> Result<Self, BevyError>
    where
        S: AsRef<str> + Into<bevy::asset::AssetPath<'static>>,
    {
        ImagePathHolder::validate_path_exists(path.as_ref())?;
        let asset_path: bevy::asset::AssetPath<'static> = path.into();
        Ok(ImagePathHolder(asset_path))
    }
    pub fn validate_path_exists<S: AsRef<str>>(path: S) -> Result<(), BevyError> {
        let img_path = format!("assets/{}", path.as_ref());
        if !std::path::Path::new(&img_path).exists() {
            let err = BevyError::from(format!("Image path does not exist: {}", img_path));
            error!(target: "image_loading", "{}", err);
            return Err(err);
        }
        Ok(())
    }
    pub fn path(&self) -> &bevy::asset::AssetPath<'static> {
        &self.0
    }
}
impl std::fmt::Display for ImagePathHolder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { write!(f, "{}", self.0) }
}
impl From<ImagePathHolder> for bevy::asset::AssetPath<'_> {
    fn from(holder: ImagePathHolder) -> Self { bevy::asset::AssetPath::from(holder.0) }
}
#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, Reflect)]
pub struct MultipleImagePathHolder(Vec<bevy::asset::AssetPath<'static>>);
impl MultipleImagePathHolder {
    pub fn new<S, I>(paths: I) -> Result<Self, BevyError>
    where
        S: AsRef<str> + Into<bevy::asset::AssetPath<'static>>,
        I: IntoIterator<Item = S>,
    {
        let mut path_vec = Vec::new();
        for path in paths {
            ImagePathHolder::validate_path_exists(path.as_ref())?;
            let asset_path: bevy::asset::AssetPath<'static> = path.into();
            path_vec.push(asset_path);
        }
        Ok(MultipleImagePathHolder(path_vec))
    }
    pub fn paths(&self) -> &Vec<bevy::asset::AssetPath<'static>> {
        &self.0
    }
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
    pub fn handle(&self) -> &Handle<Image> {
        &self.0
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