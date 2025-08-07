use bevy::platform::collections::HashMap;
#[allow(unused_imports)] use bevy::prelude::*;
use bevy_ecs_tilemap::{map::{TilemapGridSize, TilemapRenderSettings, TilemapSize}, tiles::TileTextureIndex};
#[allow(unused_imports)] use bevy_replicon::prelude::*;
#[allow(unused_imports)] use bevy_asset_loader::prelude::*;
use crate::{common::common_components::{EntityPrefix, HashId, HashIdMap}, game::{game_components::ImageHolder, tilemap::{chunking_resources::CHUNK_SIZE, tile::tile_utils::TILE_SIZE_PXS}}, AppState};


#[derive(Component, Debug, Default)]
#[require(
    StateScoped::<AppState>, EntityPrefix::new("tilemap"), 
    TilemapRenderSettings {render_chunk_size: CHUNK_SIZE*2, y_sort: false},
    TilemapGridSize { x: TILE_SIZE_PXS.x as f32, y: TILE_SIZE_PXS.y as f32 },
    TilemapSize { x: CHUNK_SIZE.x as u32, y: CHUNK_SIZE.y as u32 },
)]
pub struct Tilemap;


#[derive(Component, Debug, Clone, Default)]
pub struct TmapHashIdtoTextureIndex(pub HashIdMap<TileTextureIndex>);


#[derive(Component, Debug, Default, Clone)]
pub struct TileMapHandles(Vec<Handle<Image>>);

impl TileMapHandles {
    pub fn new(handles: Vec<Handle<Image>>) -> Self {
        if handles.is_empty() {
            Self(vec![Handle::<Image>::default()])
        } else {
            Self(handles)
        }
    }
    pub fn first_handle(&self) -> Handle<Image> {
        self.0.first().cloned().unwrap_or_default()
    }

    pub fn push_handle(&mut self, handle: Handle<Image>) {
        self.0.push(handle);
    }
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn handles_mut(&mut self) -> &mut Vec<Handle<Image>> {
        &mut self.0
    }

    pub fn take_handles(&mut self) -> Vec<Handle<Image>> {
        std::mem::take(&mut self.0)
    }
}
// Implement IntoIterator for TileMapHandles
impl IntoIterator for TileMapHandles {
    type Item = Handle<Image>;
    type IntoIter = std::vec::IntoIter<Handle<Image>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

// Implement IntoIterMut for TileMapHandles
impl<'a> IntoIterator for &'a mut TileMapHandles {
    type Item = &'a mut Handle<Image>;
    type IntoIter = std::slice::IterMut<'a, Handle<Image>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter_mut()
    }
}



#[derive(Component, Debug, Clone, Default)]
pub struct TileIdsHandles { pub ids: Vec<HashId>, pub handles: Vec<Handle<Image>>,}

impl TileIdsHandles {
    pub fn from_paths(asset_server: &AssetServer, img_paths: HashMap<String, String>,
    ) -> Result<Self, BevyError> {

        if img_paths.is_empty() {
            return Err(BevyError::from("TileImgsMap cannot be created with an empty image paths map"));
        }
        let mut ids = Vec::new();
        let mut handles = Vec::new();
        for (key, path) in img_paths {
            let image_holder = ImageHolder::new(asset_server, &path)?;
            ids.push(HashId::from(key));
            handles.push(image_holder.0);
        }

        Ok(Self { ids, handles, })

    }

    pub fn first_handle(&self) -> Handle<Image> {
        self.handles.first().cloned().unwrap_or_else(|| Handle::default())
    }

    pub fn clone_handles(&self) -> Vec<Handle<Image>> {
        self.handles.clone()
    }

    pub fn iter(&self) -> impl Iterator<Item = (HashId, &Handle<Image>)> {
        self.ids.iter().cloned().zip(self.handles.iter())
    }
}
