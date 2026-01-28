use bevy::{math::U16Vec2, render::sync_world::SyncToRenderWorld, transform};
#[allow(unused_imports)] use bevy::prelude::*;
use bevy_ecs_tilemap::{FrustumCulling, prelude::*};
#[allow(unused_imports)] use bevy_replicon::prelude::*;
use common::{common_components::*, common_states::*};
use dimension_shared::DimensionRef;
use game_common::game_common_components::ReparentingRetries;
use sprite_shared::AcZ;
use ::tilemap_shared::*;


#[derive(Bundle, Debug, Default)]
pub struct TilemapConfig {
    entity_prefix: Prefix,
    grid_size: TilemapGridSize,
    map_type: TilemapType,
    map_size: TilemapSize,
    spacing: TilemapSpacing,
    pub tile_size: TilemapTileSize,
    transform: Transform,
    chunk_pos: ChunkPos,
    global_transform: GlobalTransform,
    render_settings: TilemapRenderSettings,
    visibility: Visibility,
    inherited_visibility: InheritedVisibility,
    view_visibility: ViewVisibility,
    frustum_culling: FrustumCulling,
    sync: SyncToRenderWorld,
    anchor: TilemapAnchor,
    retries: ReparentingRetries,
    ac_z: AcZ,
    /*
*/
}

impl TilemapConfig {
    pub fn new(oplist_size: OplistSize, tile_size: U16Vec2, chunk_pos: ChunkPos, ac_z: AcZ) -> Self {
        let oplist_size_val = oplist_size.inner();
        Self {
            entity_prefix: Prefix::trunc("Tilemap"),
            tile_size: TilemapTileSize::from(tile_size.as_vec2()),
            grid_size: TilemapGridSize::from(GlobalTilePos::TILE_SIZE_PXS.as_vec2() * oplist_size_val.as_vec2()),
            map_size: TilemapSize::from(ChunkPos::CHUNK_SIZE / oplist_size_val),
            render_settings: TilemapRenderSettings {
                render_chunk_size: ChunkPos::CHUNK_SIZE * 2 / oplist_size_val,
                y_sort: false,
            },
            transform: Transform::from_translation(chunk_pos.to_pixelpos().extend(0.0)),
            chunk_pos,
            ac_z,
            ..Default::default()
        }
    }
    pub fn new_storage(oplist_size: OplistSize) -> TileStorage {
        TileStorage::empty((ChunkPos::CHUNK_SIZE / oplist_size.inner()).into())
    }
}


#[derive(Component, Debug, Clone, Default, Reflect)]
pub struct TmapHashIdtoTextureIndex(pub HashIdMap<TileTextureIndex>);
impl TmapHashIdtoTextureIndex {
    pub fn with_capacity(capacity: usize) -> Self {
        Self(HashIdMap::with_capacity(capacity))
    }
}