use bevy::{math::U16Vec2, render::sync_world::SyncToRenderWorld, };
#[allow(unused_imports)] use bevy::prelude::*;
use bevy_ecs_tilemap::{FrustumCulling, prelude::*};
#[allow(unused_imports)] use bevy_replicon::prelude::*;
use common::{common_components::*, };

use sprite_shared::AcZ;
use ::tilemap_shared::*;


#[derive(Bundle, Debug, Default)]
pub struct TilemapConfig {
    entity_prefix: Prefix,
    grid_size: TilemapGridSize,
    map_type: TilemapType,
    map_size: TilemapSize,
    spacing: TilemapSpacing,
    img_px_size: TilemapTileSize,
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
    ac_z: AcZ,
    /*
*/
}

impl TilemapConfig {
    pub fn new(size_in_tiles: SizeInTiles, img_px_size: U16Vec2, chunk_pos: ChunkPos, ac_z: AcZ, y_sort: bool) -> Self {
        Self {
            entity_prefix: Prefix::trunc("Tilemap"),
            img_px_size: TilemapTileSize::from(img_px_size.as_vec2()),
            grid_size: TilemapGridSize::from( GlobalTilePos::TILE_SIZE_PXS.as_vec2()),
            map_size: size_in_tiles.tilemap_size(),
            render_settings: TilemapRenderSettings {
                render_chunk_size: SizeInTiles::default().render_chunk_size(),
                y_sort,
            },
            transform: Transform::from_translation(chunk_pos.to_pixelpos().extend(0.0)),
            chunk_pos,
            ac_z,
            ..Default::default()
        }
    }
    pub fn new_storage(oplist_size: SizeInTiles) -> TileStorage {
        TileStorage::empty((ChunkPos::CHUNK_SIZE / oplist_size.inner()).into())
    }
}
