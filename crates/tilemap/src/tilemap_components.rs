use bevy::math::U16Vec2;
#[allow(unused_imports)] use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
use common::{common_components::*, common_states::*};
use tilemap_shared::{ChunkPos, GlobalTilePos};


use crate::{chunking_components::ChunkInitState, terrain_gen::terrgen_oplist_components::OplistSize, tile::tile_components::Tile };

#[derive(Bundle, Debug, Default)]
pub struct TilemapConfig {
    pub entity_prefix: EntityPrefix,
    pub tile_size: TilemapTileSize,
    pub grid_size: TilemapGridSize,
    pub map_size: TilemapSize,
    pub render_settings: TilemapRenderSettings,
}

impl TilemapConfig {
    pub fn new(oplist_size: OplistSize, tile_size: U16Vec2) -> Self {
        let oplist_size_val = oplist_size.inner();
        Self {
            entity_prefix: EntityPrefix::new("Tilemap"),
            tile_size: TilemapTileSize::from(tile_size.as_vec2()),
            grid_size: TilemapGridSize::from(GlobalTilePos::TILE_SIZE_PXS.as_vec2() * oplist_size_val.as_vec2()),
            map_size: TilemapSize::from(ChunkPos::CHUNK_SIZE / oplist_size_val),
            render_settings: TilemapRenderSettings {
                render_chunk_size: ChunkPos::CHUNK_SIZE * 2 / oplist_size_val,
                y_sort: false,
            },
        }
    }
    pub fn new_storage(oplist_size: OplistSize) -> TileStorage {
        TileStorage::empty((ChunkPos::CHUNK_SIZE / oplist_size.inner()).into())
    }
}


#[derive(Component, Debug, Clone, Default, Reflect)]
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
    pub fn handles(&self) -> &Vec<Handle<Image>> { &self.0 }
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

