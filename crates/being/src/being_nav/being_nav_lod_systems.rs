use camera::camera_components::CameraTarget;
use bevy::prelude::*;
use bevy::ecs::entity::EntityHashMap;
use ::being_shared::*;
use ::tilemap_shared::*;

const AI_LOD_NEAR_TILES: i32 = 72;
const AI_LOD_MID_TILES: i32 = 144;
const AI_LOD_FAR_TILES: i32 = 288;
const AI_LOD_HYSTERESIS_TILES: i32 = 12;

fn lod_level_for_tile_distance(tile_distance: i32) -> u8 {
    if tile_distance <= AI_LOD_NEAR_TILES {
        0
    } else if tile_distance <= AI_LOD_MID_TILES {
        1
    } else if tile_distance <= AI_LOD_FAR_TILES {
        2
    } else {
        3
    }
}

fn lod_level_with_hysteresis(previous_level: u8, tile_distance: i32) -> u8 {
    let d = tile_distance.max(0);
    let near_in = (AI_LOD_NEAR_TILES - AI_LOD_HYSTERESIS_TILES).max(1);
    let near_out = AI_LOD_NEAR_TILES + AI_LOD_HYSTERESIS_TILES;
    let mid_in = (AI_LOD_MID_TILES - AI_LOD_HYSTERESIS_TILES).max(1);
    let mid_out = AI_LOD_MID_TILES + AI_LOD_HYSTERESIS_TILES;
    let far_in = (AI_LOD_FAR_TILES - AI_LOD_HYSTERESIS_TILES).max(1);
    let far_out = AI_LOD_FAR_TILES + AI_LOD_HYSTERESIS_TILES;
    match previous_level {
        0 => {
            if d > near_out {
                lod_level_for_tile_distance(d)
            } else {
                0
            }
        }
        1 => {
            if d <= near_in {
                0
            } else if d > mid_out {
                lod_level_for_tile_distance(d)
            } else {
                1
            }
        }
        2 => {
            if d <= mid_in {
                lod_level_for_tile_distance(d)
            } else if d > far_out {
                3
            } else {
                2
            }
        }
        _ => {
            if d <= far_in {
                lod_level_for_tile_distance(d)
            } else {
                3
            }
        }
    }
}

#[allow(unused_parens, )]
pub fn update_being_lod_levels_from_camera(
    mut cmd: Commands,
    camera_query: Query<(&DimensionRef, &GlobalTransform), (With<CameraTarget>, )>,
    mut beings_query: Query<
        (
            Entity,
            &DimensionRef,
            &GlobalTilePos,
            Option<&mut LodLevel>,
        ),
        (LocalAiControlled, ),
    >,
    dim_map: Res<DimensionEntityMap>,
    mut cameras_by_dim: Local<EntityHashMap<Vec<GlobalTilePos>>>,
) {
    cameras_by_dim.clear();
    let camera_iter = camera_query.iter();
    let (lower, upper) = camera_iter.size_hint();
    cameras_by_dim.reserve(upper.unwrap_or(lower));
    for (&dim_ref, transform) in camera_iter {
        let Some(dim_ent) = dim_map.0.get_opt(dim_ref.0).copied() else {
            continue;
        };
        cameras_by_dim
            .entry(dim_ent)
            .or_default()
            .push(GlobalTilePos::from(transform.translation().xy()));
    }

    for (being_ent, &dim_ref, &being_gpos, lod_level) in beings_query.iter_mut() {
        let Some(dim_ent) = dim_map.0.get_opt(dim_ref.0).copied() else {
            continue;
        };
        let nearest_camera_tile_dist = cameras_by_dim
            .get(&dim_ent)
            .and_then(|camera_gpos| {
                camera_gpos
                    .iter()
                    .map(|&camera_gpos| {
                        let delta = camera_gpos.0 - being_gpos.0;
                        delta.abs().max_element()
                    })
                    .min()
            })
            .unwrap_or(i32::MAX / 4);
        let base_level = lod_level_for_tile_distance(nearest_camera_tile_dist);
        if let Some(mut lod_level) = lod_level {
            let next_level = lod_level_with_hysteresis(lod_level.0, nearest_camera_tile_dist);
            if lod_level.0 != next_level {
                lod_level.0 = next_level;
            }
        } else {
            cmd.entity(being_ent).try_insert(LodLevel(base_level));
        }
    }
}
