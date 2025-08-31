use bevy::{math::U16Vec2, render::sync_world::SyncToRenderWorld};
#[allow(unused_imports)] use bevy::prelude::*;
use bevy_ecs_tilemap::{FrustumCulling, prelude::*};
#[allow(unused_imports)] use bevy_replicon::prelude::*;
use common::{common_components::*, common_states::*};
use game_common::game_common_components::YSortOrigin;
use tilemap_shared::{ChunkPos, GlobalTilePos};


use crate::{chunking_components::Chunk, terrain_gen::terrgen_oplist_components::OplistSize, tile::tile_components::Tile };

#[derive(Bundle, Debug, Default)]
pub struct TilemapConfig {
    entity_prefix: EntityPrefix,
    pub tile_size: TilemapTileSize,
    grid_size: TilemapGridSize,
    map_size: TilemapSize,
    render_settings: TilemapRenderSettings,
    y_sort: YSortOrigin,
    /*
    spacing: TilemapSpacing,
    transform: Transform,
    global_transform: GlobalTransform,
    visibility: Visibility,
    inherited_visibility: InheritedVisibility,
    view_visibility: ViewVisibility,
    frustum_culling: FrustumCulling,
    sync: SyncToRenderWorld,
    anchor: TilemapAnchor,
*/
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
            ..Default::default()
        }
    }
    pub fn new_storage(oplist_size: OplistSize) -> TileStorage {
        TileStorage::empty((ChunkPos::CHUNK_SIZE / oplist_size.inner()).into())
    }
}


#[derive(Component, Debug, Clone, Default, Reflect)]
pub struct TmapHashIdtoTextureIndex(pub HashIdMap<TileTextureIndex>);
