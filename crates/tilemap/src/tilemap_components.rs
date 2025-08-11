#[allow(unused_imports)] use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
use common::{common_components::{EntityPrefix, HashIdMap}, common_states::AppState};


use crate::{chunking_components::ChunkInitState, tile::tile_components::Tile, };

#[derive(Component, Debug, Default)]
#[require(
    StateScoped::<AppState>(AppState::StatefulGameSession), EntityPrefix::new("Tilemap"), 
    TilemapRenderSettings {render_chunk_size: ChunkInitState::SIZE*2, y_sort: false},
    TilemapGridSize::from(Tile::PIXELS.as_vec2()),
    TilemapSize::from(ChunkInitState::SIZE),
)]
pub struct Tilemap;
impl Tilemap {
    pub fn default_storage() -> TileStorage { TileStorage::empty(ChunkInitState::SIZE.into()) }
}

#[derive(Component, Debug, Clone, Default,)]
pub struct TmapHashIdtoTextureIndex(pub HashIdMap<TileTextureIndex>);



#[derive(Component, Debug, Default, Clone)]
pub struct TileMapHandles(Vec<Handle<Image>>);

impl TileMapHandles {
    pub fn new(handles: Vec<Handle<Image>>) -> Self {
        if handles.is_empty() {
            Self(vec![Handle::<Image>::default()])
        } else { Self(handles) }
    }
    pub fn len(&self) -> usize { self.0.len() }
    pub fn first_handle(&self) -> Handle<Image> { self.0.first().cloned().unwrap_or_default() }
    pub fn push_handle(&mut self, handle: Handle<Image>) { self.0.push(handle); }
    pub fn handles_mut(&mut self) -> &mut Vec<Handle<Image>> { &mut self.0 }
    pub fn take_handles(&mut self) -> Vec<Handle<Image>> { std::mem::take(&mut self.0) }
}
impl IntoIterator for TileMapHandles {
    type Item = Handle<Image>;
    type IntoIter = std::vec::IntoIter<Handle<Image>>;
    fn into_iter(self) -> Self::IntoIter { self.0.into_iter() }
}
impl<'a> IntoIterator for &'a mut TileMapHandles {
    type Item = &'a mut Handle<Image>;
    type IntoIter = std::slice::IterMut<'a, Handle<Image>>;
    fn into_iter(self) -> Self::IntoIter { self.0.iter_mut() }
}



