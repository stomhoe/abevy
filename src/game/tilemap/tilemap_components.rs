#[allow(unused_imports)] use bevy::prelude::*;
use bevy_ecs_tilemap::{map::{TilemapGridSize, TilemapRenderSettings, TilemapSize}, tiles::TileTextureIndex};
#[allow(unused_imports)] use bevy_replicon::prelude::*;
#[allow(unused_imports)] use bevy_asset_loader::prelude::*;
use crate::{common::common_components::{EntityPrefix, HashIdMap}, game::tilemap::{chunking_resources::CHUNK_SIZE, tile::tile_utils::TILE_SIZE_PXS}, AppState};


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
