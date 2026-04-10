use bevy::ecs::entity::MapEntities;
use bevy::platform::collections::HashMap;
use bevy::prelude::*;
#[allow(unused_imports)]

use std::fmt::{Debug, Display};
use std::{fmt::Formatter, hash::Hash};
use serde::{Deserialize, Serialize};

pub use crate::common_id_components::*;

#[derive(Component, Clone, Default)]
pub struct AssetScoped;

#[derive(Component, Clone, Default)]
pub struct SelectedForHotReload;

#[derive(Component, Clone, Default)]
pub struct EguiHolder;

#[derive(Component, Debug, Default, Clone, Copy, Serialize, Deserialize)]

pub struct ReplicateIfServerStarts;

#[derive(Component, Debug, Default, Clone, Copy, Serialize, Deserialize)]
#[require(AddHashIdFromStrId, bevy_replicon::prelude::Replicated)]
pub struct RemoveReplicatedAfterClone;


#[derive(Component, Clone, Default, Serialize, Deserialize, Reflect)]
pub struct DisplayName(pub String);

impl DisplayName {
    pub fn new<S: AsRef<str>>(name: S) -> Self {
        DisplayName(name.as_ref().to_string())
    }
    pub fn trunc<S: AsRef<str>>(name: S) -> Self {
        DisplayName(name.as_ref().trim().to_string())
    }

    pub fn insert_name_if_non_empty<S: AsRef<str>>(name: S, entity: &mut EntityCommands) {
        if !name.as_ref().trim().is_empty() {
            entity.insert(DisplayName(name.as_ref().to_string()));
        }
    }
}

impl Display for DisplayName {
    #[inline(always)]
    fn fmt(&self, f: &mut Formatter) -> std::result::Result<(), std::fmt::Error> {
        Display::fmt(&self.0, f)
    }
}
impl Debug for DisplayName {
    #[inline(always)]
    fn fmt(&self, f: &mut Formatter) -> std::result::Result<(), std::fmt::Error> {
        if self.0.is_empty() {
            write!(f, "")
        } else {
            write!(f, "DN({})", self.0)
        }
    }
}

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, )]
pub struct PathHolder(bevy::asset::AssetPath<'static>);
impl PathHolder {
    pub fn new<S>(path: S) -> Result<Self, BevyError>
    where
        S: AsRef<str> + Into<bevy::asset::AssetPath<'static>>,
    {
        Self::validate_path_exists(path.as_ref())?;
        let asset_path: bevy::asset::AssetPath<'static> = path.into();
        Ok(Self(asset_path))
    }
    pub fn validate_path_exists<S: AsRef<str>>(path: S) -> Result<(), BevyError> {
        let path = path.as_ref().trim();
        match path.is_empty() {
            true => return Err(BevyError::from("Image path is empty")),
            false => {}
        }
        let path = format!("assets/{}", path);
        match std::path::Path::new(&path).try_exists() {
            Ok(true) => Ok(()),
            Ok(false) => Err(BevyError::from(format!("no file at: {}", path))),
            Err(err) => Err(BevyError::from(format!("failed to check image path '{}': {}", path, err))),
        }
    }
    pub fn path(&self) -> &bevy::asset::AssetPath<'static> {
        &self.0
    }
}
impl Display for PathHolder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl From<PathHolder> for bevy::asset::AssetPath<'_> {
    fn from(holder: PathHolder) -> Self {
        bevy::asset::AssetPath::from(holder.0)
    }
}
#[derive(Component, Debug, Default, Clone)]
pub struct MultiplePathsHolder(HashMap<StrId, bevy::asset::AssetPath<'static>>);
impl MultiplePathsHolder {
    pub fn new<I, K, P>(paths: I) -> Result<Self, BevyError>
    where
        I: IntoIterator<Item = (K, P)>,
        K: Into<StrId>,
        P: AsRef<str> + Into<bevy::asset::AssetPath<'static>>,
    {
        let mut path_map = HashMap::default();
        for (str_id, path) in paths {
            PathHolder::validate_path_exists(path.as_ref())?;
            let asset_path: bevy::asset::AssetPath<'static> = path.into();
            path_map.insert(str_id.into(), asset_path);
        }
        Ok(MultiplePathsHolder(path_map))
    }
    pub fn paths(&self) -> &HashMap<StrId, bevy::asset::AssetPath<'static>> {
        &self.0
    }
}
#[derive(Component, Debug, Clone, )]
pub struct SampleSpritesamplers(pub Vec<Entity>);

#[derive(Deserialize, Message, Serialize, Clone, MapEntities)]
pub struct RemoveReplicated(#[entities] pub Entity);

#[derive(Component, Debug, Default, Deserialize, Serialize, Eq, Clone, Copy, Hash, PartialEq)]
pub enum Grounding {
    #[default]
    Grounded,
    Swimming,
    Floating,
}

#[derive(
    Component,
    Debug,
    Clone,
    Deserialize,
    Serialize,
    Reflect,
    Copy,
    PartialEq,
    Eq,
    Hash,
    MapEntities,
)]
pub struct TemplEntiRef(#[entities] pub Entity);

#[derive(Component, Debug, Clone, Deserialize, Serialize, Copy, PartialEq, Eq, Hash)]
pub struct TemplEntiHashIdRef(pub HashId);
impl From<u8> for Grounding {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::Grounded,
            1 => Self::Swimming,
            2 => Self::Floating,
            _ => Self::Grounded,
        }
    }
}
impl From<String> for Grounding {
    fn from(s: String) -> Self {
        Self::from(s.as_str())
    }
}
impl From<&str> for Grounding {
    fn from(s: &str) -> Self {
        match s.trim().to_lowercase().as_str() {
            "swimming" | "swim" | "s" | "1" => Self::Swimming,
            "floating" | "float" | "f" | "2" => Self::Floating,
            _ => Self::Grounded,
        }
    }
}

#[derive(Component, Debug, Clone, Default, Hash, PartialEq, Eq, )]
pub struct ImageHolder(pub Handle<Image>);
impl ImageHolder {
    pub fn new<S>(asset_server: &AssetServer, path: S) -> Result<Self, BevyError>
    where
        S: AsRef<str> + Into<bevy::asset::AssetPath<'static>>,
    {
        let img_path = format!("assets/{}", path.as_ref());
        match std::path::Path::new(&img_path).try_exists() {
            Ok(true) => {}
            Ok(false) => {
                let err = BevyError::from(format!("Image path does not exist: {}", img_path));
                error!(target: "image_loading", "{}", err);
                return Err(err);
            }
            Err(err) => {
                let err = BevyError::from(format!("Failed to check image path '{}': {}", img_path, err));
                error!(target: "image_loading", "{}", err);
                return Err(err);
            }
        }
        Ok(Self(asset_server.load(path)))
    }
    pub fn handle(&self) -> &Handle<Image> {
        &self.0
    }
}

#[derive(Component, Debug, Clone, Default)]
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

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, Hash, PartialEq, Reflect)]
pub enum VisibilityGameState {
    #[default]
    Inherited,
    Visible,
    Hidden,
}
impl From<Visibility> for VisibilityGameState {
    fn from(vis: Visibility) -> Self {
        match vis {
            Visibility::Inherited => VisibilityGameState::Inherited,
            Visibility::Visible => VisibilityGameState::Visible,
            Visibility::Hidden => VisibilityGameState::Hidden,
        }
    }
}
impl From<VisibilityGameState> for Visibility {
    fn from(rvis: VisibilityGameState) -> Self {
        match rvis {
            VisibilityGameState::Inherited => Visibility::Inherited,
            VisibilityGameState::Visible => Visibility::Visible,
            VisibilityGameState::Hidden => Visibility::Hidden,
        }
    }
}
