use bevy::{math::U16Vec2, prelude::*};
use bevy_ecs_tilemap::prelude::TilemapTileSize;
use sprite_shared::prelude::AcZ;
use tilemap_shared::*;

use crate::tile::tile_shader::tile_shader_components::TileShaderRef;
use crate::tilemap_structs::{MapKey, TmapMap};

#[allow(unused_parens)]
pub fn on_tilemap_despawn(
    trig: On<Despawn, (TilemapTileSize,)>,
    query: Query<(&DimensionRef, &ChunkPos, &AcZ, &TilemapTileSize, &TileShaderRef)>,
    mut tmap_map: ResMut<TmapMap>,
) {
    let Ok((dimension_ref, chunk_pos, ac_z, tile_size, shader_ref)) = query.get(trig.entity) else {
        error_once!("Failed to get tilemap despawn query for entity {:?}", trig.entity);
        return;
    };
    let map_key = MapKey::new(
        *dimension_ref,
        *chunk_pos,
        *ac_z,
        U16Vec2::new(tile_size.x as u16, tile_size.y as u16),
        if shader_ref.is_placeholder() { None } else { Some(*shader_ref) },
    );
    tmap_map.0.remove(&map_key);
}
