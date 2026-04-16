use bevy::{
    asset::RenderAssetUsages,
    ecs::{entity::EntityHashMap, system::SystemParam},
    math::U16Vec2,
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};
use bevy_ecs_tilemap::prelude::*;
use common::TILEMAP_SYSTEM;
use sprite_shared::AcZ;
use ::tilemap_shared::*;

use crate::{
    tile::{tile_resources::*, tile_shader::{tile_shader_components::*, tile_shader_resources::TileShaderEntityMap}},
    tilemap_structs::{MapKey, NeedsTerrblRefresh},
};

#[derive(SystemParam)]
pub struct RefreshTerrblTilemapsParams<'w, 's> {
    pub texture_overlay_mat: ResMut<'w, Assets<TerrBlendMat>>,
    pub images: ResMut<'w, Assets<Image>>,
    pub shader_query: Query<'w, 's, &'static TileShader>,
    pub shader_map: Res<'w, TileShaderEntityMap>,
    pub tile_ref_query: Query<'w, 's, &'static TileRef>,
    pub tile_map: Res<'w, TileEntityMap>,
    pub tile_texture_index_query: Query<'w, 's, &'static TileTextureIndex>,
    pub terrbl_params_query: Query<'w, 's, &'static TerrBlendParams>,
    pub tilemaps: Query<'w, 's, (
        Entity,
        &'static TileStorage,
        &'static TileShaderRef,
        &'static TilemapTileSize,
        &'static DimensionRef,
        &'static ChunkPos,
        &'static AcZ,
        &'static mut MaterialTilemapHandle<TerrBlendMat>,
    ), (With<NeedsTerrblRefresh>, )>,
    pub all_tilemaps: Query<'w, 's, (
        Entity,
        &'static TileStorage,
        &'static TileShaderRef,
        &'static TilemapTileSize,
        &'static DimensionRef,
        &'static ChunkPos,
        &'static AcZ,
    ), ()>,
}

pub fn refresh_terrbl_tilemaps(
    mut cmd: Commands,
    mut params: RefreshTerrblTilemapsParams,
    mut terrbl_debug_budget: Local<u32>,
) {
    if *terrbl_debug_budget == 0 {
        *terrbl_debug_budget = 40;
    }

    let mut terrbl_mapkeys: bevy::platform::collections::HashMap<MapKey, Entity> = bevy::platform::collections::HashMap::default();
    for (tmap_ent, _storage, shader_ref, tile_size, &dim_ref, &chunk_pos, &ac_z) in &params.all_tilemaps {
        let Ok(shader_ent) = params.shader_map.0.get_cloned(shader_ref.0) else {
            continue;
        };
        let Ok(shader) = params.shader_query.get(shader_ent) else {
            continue;
        };
        if matches!(shader, TileShader::TerrBlend(_)) {
            terrbl_mapkeys.insert(
                MapKey::new(
                    dim_ref,
                    chunk_pos,
                    ac_z,
                    U16Vec2::new(tile_size.x as u16, tile_size.y as u16),
                    Some(*shader_ref),
                ),
                tmap_ent,
            );
        }
    }

    for (tmap_ent, storage, shader_ref, tile_size, &dim_ref, &chunk_pos, &ac_z, mut material_handle) in &mut params.tilemaps {
        let Ok(shader_ent) = params.shader_map.0.get_cloned(shader_ref.0) else {
            cmd.entity(tmap_ent).try_remove::<NeedsTerrblRefresh>();
            continue;
        };
        let Ok(shader) = params.shader_query.get(shader_ent) else {
            cmd.entity(tmap_ent).try_remove::<NeedsTerrblRefresh>();
            continue;
        };
        let tile_size_px = U16Vec2::new(tile_size.x as u16, tile_size.y as u16);
        match shader {
            TileShader::TerrBlend(_) => {
                let chunk_w = storage.size.x as i32;
                let chunk_h = storage.size.y as i32;
                let Some(material) = build_terrbl_material_for_map(
                    &mut params.images,
                    &params.tile_ref_query,
                    &params.tile_map,
                    &params.tile_texture_index_query,
                    &EntityHashMap::default(),
                    &params.terrbl_params_query,
                    storage,
                    tmap_ent,
                    tile_size_px,
                    &mut terrbl_debug_budget,
                    |x, y| {
                        let dx = if x < 0 {
                            -1
                        } else if x >= chunk_w {
                            1
                        } else {
                            0
                        };
                        let dy = if y < 0 {
                            -1
                        } else if y >= chunk_h {
                            1
                        } else {
                            0
                        };
                        if dx == 0 && dy == 0 {
                            return storage.get(&TilePos { x: x as u32, y: y as u32 });
                        }
                        let neighbor_key = MapKey::new(
                            dim_ref,
                            chunk_pos + IVec2::new(dx, dy),
                            ac_z,
                            tile_size_px,
                            Some(*shader_ref),
                        );
                        let Some(&neighbor_tmap_ent) = terrbl_mapkeys.get(&neighbor_key) else {
                            return None;
                        };
                        let Ok((_, neighbor_storage, _, _, _, _, _)) = params.all_tilemaps.get(neighbor_tmap_ent) else {
                            return None;
                        };
                        let nx = if dx < 0 {
                            neighbor_storage.size.x as i32 - 1
                        } else if dx > 0 {
                            0
                        } else {
                            x
                        };
                        let ny = if dy < 0 {
                            neighbor_storage.size.y as i32 - 1
                        } else if dy > 0 {
                            0
                        } else {
                            y
                        };
                        if nx < 0 || ny < 0 || nx >= neighbor_storage.size.x as i32 || ny >= neighbor_storage.size.y as i32 {
                            return None;
                        }
                        neighbor_storage.get(&TilePos { x: nx as u32, y: ny as u32 })
                    },
                ) else {
                    error!(
                        target: TILEMAP_SYSTEM,
                        "Failed to refresh terrbl material for marked map: tmap {:?}, tile_size {:?}, storage {}x{}",
                        tmap_ent,
                        tile_size,
                        storage.size.x,
                        storage.size.y,
                    );
                    cmd.entity(tmap_ent).try_remove::<NeedsTerrblRefresh>();
                    continue;
                };
                trace!(target: TILEMAP_SYSTEM, "terrbl debug: refreshed marked material for tmap {:?}", tmap_ent);
                let curr_handle = (**material_handle).clone();
                if let Some(curr_mat) = params.texture_overlay_mat.get_mut(&curr_handle) {
                    *curr_mat = material;
                } else {
                    **material_handle = params.texture_overlay_mat.add(material);
                }
            }
        }
        cmd.entity(tmap_ent).try_remove::<NeedsTerrblRefresh>();
    }
}

pub fn build_terrbl_material_for_map(
    images: &mut Assets<Image>,
    tile_ref_query: &Query<&TileRef>,
    tile_map: &TileEntityMap,
    tile_texture_index_query: &Query<&TileTextureIndex>,
    tile_runtime_info: &EntityHashMap<(TileRef, TileTextureIndex)>,
    templ_terrbl_query: &Query<&TerrBlendParams>,
    storage: &TileStorage,
    tmap_ent: Entity,
    tile_size_px: U16Vec2,
    terrbl_debug_budget: &mut u32,
    mut tile_lookup: impl FnMut(i32, i32) -> Option<Entity>,
) -> Option<TerrBlendMat> {
    const MAX_TERRBL_OVERLAYS: usize = 8;
    let width = storage.size.x;
    let height = storage.size.y;
    if width == 0 || height == 0 {
        error!(
            target: TILEMAP_SYSTEM,
            "terrbl debug: build skipped due to zero storage size for tmap {:?} (storage: {}x{}, tile_size: {:?}, )",
            tmap_ent,
            width,
            height,
            tile_size_px,
        );
        return None;
    }
    let padded_width = width + 2;
    let padded_height = height + 2;
    let px_count = (padded_width as usize) * (padded_height as usize);
    let mut tile_indices_data = vec![0_u8; px_count * 4];
    let mut tile_flags_data = vec![0_u8; px_count * 4];
    let mut tile_params_data = vec![0_u8; px_count * 16];
    let mut tile_tint_data = vec![0_u8; px_count * 16];
    let mut overlay_textures: Vec<Handle<Image>> = Vec::new();

    for y in 0..padded_height {
        for x in 0..padded_width {
            let local_x = x as i32 - 1;
            let local_y = y as i32 - 1;
            let Some(tile_ent) = tile_lookup(local_x, local_y) else {
                continue;
            };
            let (templ_ref, base_texture_index) = if let Some((templ_ref, base_texture_index)) = tile_runtime_info.get(&tile_ent) {
                (*templ_ref, *base_texture_index)
            } else if let (Ok(templ_ref), Ok(base_texture_index)) = (
                tile_ref_query.get(tile_ent),
                tile_texture_index_query.get(tile_ent),
            ) {
                (*templ_ref, *base_texture_index)
            } else {
                if *terrbl_debug_budget > 0 {
                    *terrbl_debug_budget -= 1;
                    error!(target: TILEMAP_SYSTEM, "terrbl debug: missing tile runtime/query data at local ({}, {}) ent {:?}", local_x, local_y, tile_ent);
                }
                continue;
            };
            let px_i = ((y as usize) * (padded_width as usize) + (x as usize)) * 4;
            encode_u16(&mut tile_indices_data, px_i, base_texture_index.0 as u16);
            let Ok(templ_ent) = tile_map.0.get_cloned(templ_ref.0) else {
                if *terrbl_debug_budget > 0 {
                    *terrbl_debug_budget -= 1;
                    error!(target: TILEMAP_SYSTEM, "terrbl debug: no template entity for tile ref {:?} local ({}, {})", templ_ref, local_x, local_y);
                }
                continue;
            };
            let Ok(params) = templ_terrbl_query.get(templ_ent) else {
                if *terrbl_debug_budget > 0 {
                    *terrbl_debug_budget -= 1;
                    error!(target: TILEMAP_SYSTEM, "terrbl debug: no TerrBlendParams on templ {:?} local ({}, {})", templ_ent, local_x, local_y);
                }
                continue;
            };
            let mut flags = 0_u8;
            flags |= 1 << 0;
            if params.blend_enabled { flags |= 1 << 1; }
            if params.has_tint { flags |= 1 << 3; }
            if params.has_tint_mask_target {
                flags |= 1 << 4;
                tile_flags_data[px_i + 1] = (params.tint_mask_target.x.clamp(0.0, 1.0) * 255.0) as u8;
                tile_flags_data[px_i + 2] = (params.tint_mask_target.y.clamp(0.0, 1.0) * 255.0) as u8;
                tile_flags_data[px_i + 3] = (params.tint_mask_target.z.clamp(0.0, 1.0) * 255.0) as u8;
            } else {
                tile_flags_data[px_i + 3] = 255;
            }

            let mut overlay_idx = 0_u16;
            let path_label = params.texture_path.to_string();
            if !path_label.is_empty() {
                let overlay_handle = params.texture_handle.clone();
                if overlay_handle == Handle::default() {
                    if *terrbl_debug_budget > 0 {
                        *terrbl_debug_budget -= 1;
                        error!(target: TILEMAP_SYSTEM, "terrbl debug: missing texture handle for '{}' at local ({}, {})", path_label, local_x, local_y);
                    }
                    continue;
                }
                flags |= 1 << 2;
                overlay_idx = match overlay_textures.iter().position(|h| *h == overlay_handle) {
                    Some(i) => i as u16,
                    None => {
                        if overlay_textures.len() >= MAX_TERRBL_OVERLAYS {
                            if *terrbl_debug_budget > 0 {
                                *terrbl_debug_budget -= 1;
                                error!(target: TILEMAP_SYSTEM, "terrbl debug: too many overlay textures in one terrbl map (max {}), skipping '{}' at local ({}, {})", MAX_TERRBL_OVERLAYS, path_label, local_x, local_y);
                            }
                            flags &= !(1 << 2);
                            0
                        } else {
                            overlay_textures.push(overlay_handle);
                            (overlay_textures.len() - 1) as u16
                        }
                    }
                };
            }
            encode_u16(&mut tile_indices_data, px_i + 2, overlay_idx);
            tile_flags_data[px_i] = flags;
            if *terrbl_debug_budget > 0 {
                *terrbl_debug_budget -= 1;
                trace!(target: TILEMAP_SYSTEM, "terrbl debug: local ({}, {}) base_idx {} overlay_idx {} flags {:08b} has_params {} blend_enabled {} tex '{}'", local_x, local_y, base_texture_index.0, overlay_idx, flags, true, params.blend_enabled, path_label);
            }
            encode_f32x4(&mut tile_params_data, px_i * 4, [params.scale, params.speed, params.wavy_strength, params.priority]);
            encode_f32x4(&mut tile_tint_data, px_i * 4, [params.tint.x, params.tint.y, params.tint.z, params.time_offset]);
        }
    }
    let tile_indices_map = images.add(create_image_u8(padded_width, padded_height, tile_indices_data));
    let tile_flags_map = images.add(create_image_u8(padded_width, padded_height, tile_flags_data));
    let tile_params_map = images.add(create_image_f32(padded_width, padded_height, tile_params_data));
    let tile_tint_map = images.add(create_image_f32(padded_width, padded_height, tile_tint_data));

    let mut mat = TerrBlendMat {
        tile_indices_map,
        tile_flags_map,
        tile_params_map,
        tile_tint_map,
        map_size_tiles: Vec2::new(width as f32, height as f32),
        time: 0.0,
        ..Default::default()
    };
    if let Some(h) = overlay_textures.first() { mat.overlay_tex_0 = h.clone(); }
    if let Some(h) = overlay_textures.get(1) { mat.overlay_tex_1 = h.clone(); }
    if let Some(h) = overlay_textures.get(2) { mat.overlay_tex_2 = h.clone(); }
    if let Some(h) = overlay_textures.get(3) { mat.overlay_tex_3 = h.clone(); }
    if let Some(h) = overlay_textures.get(4) { mat.overlay_tex_4 = h.clone(); }
    if let Some(h) = overlay_textures.get(5) { mat.overlay_tex_5 = h.clone(); }
    if let Some(h) = overlay_textures.get(6) { mat.overlay_tex_6 = h.clone(); }
    if let Some(h) = overlay_textures.get(7) { mat.overlay_tex_7 = h.clone(); }
    Some(mat)
}

fn encode_u16(out: &mut [u8], index: usize, value: u16) {
    out[index] = (value & 0x00FF) as u8;
    out[index + 1] = ((value >> 8) & 0x00FF) as u8;
}

fn encode_f32x4(out: &mut [u8], index: usize, values: [f32; 4]) {
    let mut byte_i = index;
    for value in values {
        let bytes = value.to_ne_bytes();
        out[byte_i] = bytes[0];
        out[byte_i + 1] = bytes[1];
        out[byte_i + 2] = bytes[2];
        out[byte_i + 3] = bytes[3];
        byte_i += 4;
    }
}

fn create_image_u8(width: u32, height: u32, data: Vec<u8>) -> Image {
    Image::new(
        Extent3d { width, height, depth_or_array_layers: 1 },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8Unorm,
        RenderAssetUsages::default(),
    )
}

fn create_image_f32(width: u32, height: u32, data: Vec<u8>) -> Image {
    Image::new(
        Extent3d { width, height, depth_or_array_layers: 1 },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba32Float,
        RenderAssetUsages::default(),
    )
}
