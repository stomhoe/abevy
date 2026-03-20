#[allow(unused_imports)] use bevy::prelude::*;
use bevy::platform::collections::HashSet;

use common::common_components::HashId;
#[allow(unused_imports)] use common::log_targets::DUNGEONING_SYSTEM;
use game_common::game_common_components::EntityZeroRef;
use rand::{Rng, SeedableRng};
use rand_distr::num_traits::Float;
use ::tilemap_shared::*;

use crate::regioning::{    regioning_components::*,
    regioning_messages::{StructureBuildCompliance, SgcPrepareTilesOrder},
    regioning_sgc_components::StructuredGenConfig,
};
use crate::tile::tile_resources::*;
use super::super::dungeoning_carve_helpers::{
    carve_corridor_horizontal, carve_corridor_vertical, carve_room_circle, carve_room_rectangle,
    carve_room_regular_polygon, carve_room_triangle_vertices,
};
use super::super::dungeoning_ids::CHAMBERS_CORRIDORS;
use super::super::dungeoning_utils::extend_occupied_gpos;

#[derive(Clone, Copy)]
pub struct Room {
    x: i32,
    y: i32,
    w: i32,
    h: i32,
}

#[allow(unused_parens, )]
pub fn corridor_dungeon_building_system(
    mut reader: MessageReader<SgcPrepareTilesOrder>,
    structured_gens: Query<(&StructuredGenConfig,),()>,
    mut writer: MessageWriter<StructureBuildCompliance>,
    ezeros_map: Res<TileEntityMap>,
    ezero_size_query: Query<&SizeInTiles, (With<game_common::game_common_components::EntityZero>, common::AnyDisabling)>,
    settings: Query<&GlobalGenSettings>,
    dimension_hash: Query<&HashId>,
    mut compliances_to_emit: Local<Vec<StructureBuildCompliance>>,
    mut rooms: Local<Vec<Room>>,
    mut tiles4chunk: Local<TilesFromBuilder>,
) {
    let Ok(settings) = settings.single() else {
        error_once!("Failed to get global gen settings");
        return;
    };
    compliances_to_emit.clear();
    rooms.clear();
    tiles4chunk.clear();
    for build_order in reader.read() {

        let Ok((structured_gen_cfg,)) = structured_gens.get(build_order.structured_gen_cfg_ent) else { continue; };

        if structured_gen_cfg.structure_hash_id() != CHAMBERS_CORRIDORS {
            continue;
        }
        let floor_tile_id = structured_gen_cfg.args
            .get("floor_tile_id")
            .and_then(|v| v.first())
            .map(|s| HashId::hash(s.as_str()))
            .unwrap_or_else(|| HashId::hash("dunewbie"));

        let wall_tile_id = structured_gen_cfg.args
            .get("wall_tile_id")
            .and_then(|v| v.first())
            .map(|s| HashId::hash(s.as_str()))
            .unwrap_or_else(|| HashId::hash("gray"));

        let lava_tile_id = structured_gen_cfg.args
            .get("lava_tile_id")
            .and_then(|v| v.first())
            .map(|s| HashId::hash(s.as_str()))
            .unwrap_or_else(|| HashId::hash("lava"));
        let delete_other_tiles_by_tile_id = super::super::dungeoning_utils::DeleteOtherTilesConfigMap::from_args(&structured_gen_cfg.args);
        let terrgen_disable_by_tile_id = super::super::dungeoning_utils::TerrGenDisableConfigMap::from_args(&structured_gen_cfg.args);

        let floor_entity = match ezeros_map.0.get_cloned(floor_tile_id) {
            Ok(entity) => EntityZeroRef(entity),
            Err(_) => {
                error!(target: "dungeoning", "TileEzero '{:?}' not found", floor_tile_id);
                continue;
            }
        };

        let wall_entity = match ezeros_map.0.get_cloned(wall_tile_id) {
            Ok(entity) => EntityZeroRef(entity),
            Err(_) => {
                error!(target: "dungeoning", "TileEzero '{:?}' not found", wall_tile_id);
                continue;
            }
        };

        let lava_entity = match ezeros_map.0.get_cloned(lava_tile_id) {
            Ok(entity) => EntityZeroRef(entity),
            Err(_) => {
                error!(target: "dungeoning", "TileEzero '{:?}' not found", lava_tile_id);
                continue;
            }
        };

        let chunk_positions = &build_order.chunks_gpos;
        if chunk_positions.is_empty() { continue; }

        let min_chunk_x = chunk_positions.iter().map(|chunk| chunk.x()).min().unwrap();
        let max_chunk_x = chunk_positions.iter().map(|chunk| chunk.x()).max().unwrap();
        let min_chunk_y = chunk_positions.iter().map(|chunk| chunk.y()).min().unwrap();
        let max_chunk_y = chunk_positions.iter().map(|chunk| chunk.y()).max().unwrap();

        let chunk_width = (max_chunk_x - min_chunk_x + 1) as usize;
        let chunk_height = (max_chunk_y - min_chunk_y + 1) as usize;
        let tile_width = chunk_width * ChunkPos::CHUNK_SIZE.x as usize;
        let tile_height = chunk_height * ChunkPos::CHUNK_SIZE.y as usize;
        if tile_width == 0 || tile_height == 0 { continue; }

        let origin_chunk = ChunkPos::new(min_chunk_x, min_chunk_y);
        let origin_tile = origin_chunk.to_tilepos();

        let tile_map_size = tile_width * tile_height;
        let mut floor_map = vec![false; tile_map_size];
        let mut hazard_map = vec![false; tile_map_size];

        let Ok(&dimension_hash) = dimension_hash.get(build_order.dimension_ref.0) else {
            error!(target: "dungeoning", "Dimension entity {:?} has no HashId component when making AdvancedDungeon, skipping structure spawn", build_order.dimension_ref);
            continue;
        };

        let seed = chunk_positions[0].hash_value(&settings, dimension_hash, 1);
        let mut rng = rand_pcg::Pcg64Mcg::seed_from_u64(seed);

        #[derive(Clone, Copy)]
        enum RoomShape {
            Rectangle,
            Circle,
            Triangle,
            RegularPolygon,
        }

        let rect_weight: f32 = structured_gen_cfg
            .args
            .parse_arg("room_shape_weight_rectangle", 1.0)
            .max(0.0);
        let circle_weight: f32 = structured_gen_cfg
            .args
            .parse_arg("room_shape_weight_circle", 1.0)
            .max(0.0);
        let triangle_weight: f32 = structured_gen_cfg
            .args
            .parse_arg("room_shape_weight_triangle", 1.0)
            .max(0.0);
        let polygon_weight: f32 = structured_gen_cfg
            .args
            .parse_arg("room_shape_weight_polygon", 1.0)
            .max(0.0);
        let same_shape_chance: f32 = structured_gen_cfg
            .args
            .parse_arg("room_same_shape_chance", 0.25)
            .clamp(0.0, 1.0);
        let polygon_min_sides: i32 = structured_gen_cfg
            .args
            .parse_arg("room_polygon_min_sides", 4)
            .clamp(3, 12);
        let polygon_max_sides: i32 = structured_gen_cfg
            .args
            .parse_arg("room_polygon_max_sides", 8)
            .clamp(polygon_min_sides, 16);
        let polygon_rotation_min_deg: f32 = structured_gen_cfg
            .args
            .parse_arg("room_polygon_rotation_min_deg", 0.0);
        let polygon_rotation_max_deg: f32 = structured_gen_cfg
            .args
            .parse_arg("room_polygon_rotation_max_deg", 360.0);
        let same_shape_rect_weight: f32 = structured_gen_cfg
            .args
            .parse_arg("room_same_shape_weight_rectangle", rect_weight)
            .max(0.0);
        let same_shape_circle_weight: f32 = structured_gen_cfg
            .args
            .parse_arg("room_same_shape_weight_circle", circle_weight)
            .max(0.0);
        let same_shape_triangle_weight: f32 = structured_gen_cfg
            .args
            .parse_arg("room_same_shape_weight_triangle", triangle_weight)
            .max(0.0);
        let same_shape_polygon_weight: f32 = structured_gen_cfg
            .args
            .parse_arg("room_same_shape_weight_polygon", polygon_weight)
            .max(0.0);
        let _corridor_wiggle_chance: Option<f32> = structured_gen_cfg
            .args
            .parse_opt_arg("corridor_wiggle_chance");
        let _corridor_wiggle_step_max: Option<i32> = structured_gen_cfg
            .args
            .parse_opt_arg("corridor_wiggle_step_max");
        let corridor_detour_chance: f32 = structured_gen_cfg
            .args
            .parse_arg("corridor_detour_chance", 0.0)
            .clamp(0.0, 1.0);
        let corridor_detour_max_offset = structured_gen_cfg
            .args
            .parse_arg("corridor_detour_max_offset", 0)
            .clamp(0, 32);

        let pick_shape = |rng: &mut rand_pcg::Pcg64Mcg| -> RoomShape {
            let total = rect_weight + circle_weight + triangle_weight + polygon_weight;
            if total <= 0.0 {
                return RoomShape::Rectangle;
            }
            let roll = rng.random_range(0.0..total);
            if roll < rect_weight {
                RoomShape::Rectangle
            } else if roll < rect_weight + circle_weight {
                RoomShape::Circle
            } else if roll < rect_weight + circle_weight + triangle_weight {
                RoomShape::Triangle
            } else {
                RoomShape::RegularPolygon
            }
        };

        let pick_same_shape = |rng: &mut rand_pcg::Pcg64Mcg| -> RoomShape {
            let total = same_shape_rect_weight + same_shape_circle_weight + same_shape_triangle_weight + same_shape_polygon_weight;
            if total <= 0.0 {
                return RoomShape::Rectangle;
            }
            let roll = rng.random_range(0.0..total);
            if roll < same_shape_rect_weight {
                RoomShape::Rectangle
            } else if roll < same_shape_rect_weight + same_shape_circle_weight {
                RoomShape::Circle
            } else if roll < same_shape_rect_weight + same_shape_circle_weight + same_shape_triangle_weight {
                RoomShape::Triangle
            } else {
                RoomShape::RegularPolygon
            }
        };

        let use_same_shape = rng.random_range(0.0..1.0) < same_shape_chance;
        let same_shape = if use_same_shape { Some(pick_same_shape(&mut rng)) } else { None };

        #[derive(Clone, Copy)]
        struct Rect { x: i32, y: i32, w: i32, h: i32 }
        impl Rect {
            fn area(&self) -> i32 { self.w * self.h }
        }

        let mut leafs: Vec<Rect> = vec![ Rect { x: 0, y: 0, w: tile_width as i32, h: tile_height as i32 } ];

        let target_rooms = std::cmp::max(2, (tile_map_size / 250).min(20));
        let min_room_size = 5;
        let max_splits = 300;

        for _ in 0..max_splits {
            if leafs.len() >= target_rooms { break; }
            let (idx, rect) = match leafs.iter().enumerate().max_by_key(|(_,r)| r.area()) {
                Some((i, r)) => (i, *r),
                None => continue,
            };
            let can_split_h = rect.h >= (min_room_size * 2 + 3);
            let can_split_w = rect.w >= (min_room_size * 2 + 3);
            if !can_split_h && !can_split_w { break; }

            if can_split_w && (!can_split_h || rect.w >= rect.h) {
                let split_min = rect.x + min_room_size;
                let split_max = rect.x + rect.w - min_room_size;
                if split_max - split_min <= 0 { break; }
                let split = rng.random_range(split_min..split_max);
                let left = Rect { x: rect.x, y: rect.y, w: split - rect.x, h: rect.h };
                let right = Rect { x: split, y: rect.y, w: rect.x + rect.w - split, h: rect.h };
                leafs.remove(idx);
                leafs.push(left);
                leafs.push(right);
            } else if can_split_h {
                let split_min = rect.y + min_room_size;
                let split_max = rect.y + rect.h - min_room_size;
                if split_max - split_min <= 0 { break; }
                let split = rng.random_range(split_min..split_max);
                let top = Rect { x: rect.x, y: rect.y, w: rect.w, h: split - rect.y };
                let bot = Rect { x: rect.x, y: split, w: rect.w, h: rect.y + rect.h - split };
                leafs.remove(idx);
                leafs.push(top);
                leafs.push(bot);
            }
        }

        for r in &leafs {
            let available_w = r.w - 2;
            let available_h = r.h - 2;
            if available_w < min_room_size as i32 || available_h < min_room_size as i32 { continue; }
            let room_w = rng.random_range(min_room_size as i32..=available_w) as i32;
            let room_h = rng.random_range(min_room_size as i32..=available_h) as i32;
            let px_min = r.x + 1;
            let px_max = (r.x + r.w - room_w - 1).max(px_min);
            let py_min = r.y + 1;
            let py_max = (r.y + r.h - room_h - 1).max(py_min);
            let px = rng.random_range(px_min..=px_max);
            let py = rng.random_range(py_min..=py_max);
            rooms.push(Room { x: px, y: py, w: room_w, h: room_h });
        }

        // Carve rooms with configurable shapes
        for rm in &rooms {
            let shape = same_shape.unwrap_or_else(|| pick_shape(&mut rng));
            match shape {
                RoomShape::Rectangle => {
                    carve_room_rectangle(&mut floor_map, tile_width, tile_height, rm.x, rm.y, rm.w, rm.h);
                }
                RoomShape::Circle => {
                    carve_room_circle(&mut floor_map, tile_width, tile_height, rm.x, rm.y, rm.w, rm.h);
                }
                RoomShape::Triangle => {
                    // Randomly choose triangle type
                    let triangle_type = rng.random_range(0..3);

                    let (v0, v1, v2) = match triangle_type {
                        0 => {
                            // Isosceles triangles - randomly choose orientation
                            let orientation = rng.random_range(0..4);
                            match orientation {
                                0 => {
                                    // Pointing up
                                    ((rm.x + rm.w / 2, rm.y), (rm.x, rm.y + rm.h), (rm.x + rm.w, rm.y + rm.h))
                                }
                                1 => {
                                    // Pointing down
                                    ((rm.x + rm.w / 2, rm.y + rm.h), (rm.x, rm.y), (rm.x + rm.w, rm.y))
                                }
                                2 => {
                                    // Pointing right
                                    ((rm.x + rm.w, rm.y + rm.h / 2), (rm.x, rm.y), (rm.x, rm.y + rm.h))
                                }
                                _ => {
                                    // Pointing left
                                    ((rm.x, rm.y + rm.h / 2), (rm.x + rm.w, rm.y), (rm.x + rm.w, rm.y + rm.h))
                                }
                            }
                        }
                        1 => {
                            // Right-angled triangles - randomly choose corner
                            let corner = rng.random_range(0..4);
                            match corner {
                                0 => {
                                    // Right angle at top-left
                                    ((rm.x, rm.y), (rm.x + rm.w, rm.y), (rm.x, rm.y + rm.h))
                                }
                                1 => {
                                    // Right angle at top-right
                                    ((rm.x, rm.y), (rm.x + rm.w, rm.y), (rm.x + rm.w, rm.y + rm.h))
                                }
                                2 => {
                                    // Right angle at bottom-left
                                    ((rm.x, rm.y + rm.h), (rm.x + rm.w, rm.y + rm.h), (rm.x, rm.y))
                                }
                                _ => {
                                    // Right angle at bottom-right
                                    ((rm.x, rm.y + rm.h), (rm.x + rm.w, rm.y + rm.h), (rm.x + rm.w, rm.y))
                                }
                            }
                        }
                        _ => {
                            // Scalene triangles - random vertex positions
                            let v0_x = rm.x + rng.random_range(0..=rm.w / 3);
                            let v0_y = rm.y + rng.random_range(0..=rm.h / 4);
                            let v1_x = rm.x + rm.w - rng.random_range(0..=rm.w / 4);
                            let v1_y = rm.y + rm.h;
                            let v2_x = rm.x + rng.random_range(rm.w / 2..=rm.w);
                            let v2_y = rm.y + rng.random_range(rm.h / 3..=rm.h);
                            ((v0_x, v0_y), (v1_x, v1_y), (v2_x, v2_y))
                        }
                    };
                    carve_room_triangle_vertices(&mut floor_map, tile_width, tile_height, v0, v1, v2);
                }
                RoomShape::RegularPolygon => {
                    let sides = if polygon_max_sides <= polygon_min_sides {
                        polygon_min_sides
                    } else {
                        rng.random_range(polygon_min_sides..=polygon_max_sides)
                    };
                    let rotation_deg = if polygon_rotation_max_deg <= polygon_rotation_min_deg {
                        polygon_rotation_min_deg
                    } else {
                        rng.random_range(polygon_rotation_min_deg..=polygon_rotation_max_deg)
                    };
                    carve_room_regular_polygon(
                        &mut floor_map,
                        tile_width,
                        tile_height,
                        rm.x,
                        rm.y,
                        rm.w,
                        rm.h,
                        sides,
                        rotation_deg,
                    );
                }
            }
        }

        let mut corridor_map = vec![false; tile_map_size];
        let corridor_radius: Option<i32> = structured_gen_cfg
            .args
            .parse_opt_arg("corridor_radius");
        let corridor_wiggle_chance: Option<f32> = structured_gen_cfg
            .args
            .parse_opt_arg("corridor_wiggle_chance");
        let corridor_wiggle_step_max = structured_gen_cfg
            .args
            .parse_opt_arg("corridor_wiggle_step_max");

        if !rooms.is_empty() {
            let mut centers: Vec<(i32,i32)> = rooms.iter().map(|r| (r.x + r.w/2, r.y + r.h/2)).collect();
            centers.sort_by_key(|c| (c.0, c.1));

            for i in 1..centers.len() {
                let (x0,y0) = centers[i-1];
                let (x1,y1) = centers[i];

                if rng.random_range(0.0..1.0) < corridor_detour_chance {
                    let min_x = (x0.min(x1) - corridor_detour_max_offset).clamp(0, (tile_width - 1) as i32);
                    let max_x = (x0.max(x1) + corridor_detour_max_offset).clamp(0, (tile_width - 1) as i32);
                    let min_y = (y0.min(y1) - corridor_detour_max_offset).clamp(0, (tile_height - 1) as i32);
                    let max_y = (y0.max(y1) + corridor_detour_max_offset).clamp(0, (tile_height - 1) as i32);

                    let detour_x = rng.random_range(min_x..=max_x);
                    let detour_y = rng.random_range(min_y..=max_y);

                    carve_corridor_horizontal(
                        &mut rng,
                        &mut floor_map,
                        &mut corridor_map,
                        tile_width,
                        tile_height,
                        corridor_radius,
                        corridor_wiggle_chance,
                        corridor_wiggle_step_max,
                        y0,
                        x0,
                        detour_x,
                    );
                    carve_corridor_vertical(
                        &mut rng,
                        &mut floor_map,
                        &mut corridor_map,
                        tile_width,
                        tile_height,
                        corridor_radius,
                        corridor_wiggle_chance,
                        corridor_wiggle_step_max,
                        detour_x,
                        y0,
                        detour_y,
                    );
                    carve_corridor_horizontal(
                        &mut rng,
                        &mut floor_map,
                        &mut corridor_map,
                        tile_width,
                        tile_height,
                        corridor_radius,
                        corridor_wiggle_chance,
                        corridor_wiggle_step_max,
                        detour_y,
                        detour_x,
                        x1,
                    );
                    carve_corridor_vertical(
                        &mut rng,
                        &mut floor_map,
                        &mut corridor_map,
                        tile_width,
                        tile_height,
                        corridor_radius,
                        corridor_wiggle_chance,
                        corridor_wiggle_step_max,
                        x1,
                        detour_y,
                        y1,
                    );
                } else {
                    carve_corridor_horizontal(
                        &mut rng,
                        &mut floor_map,
                        &mut corridor_map,
                        tile_width,
                        tile_height,
                        corridor_radius,
                        corridor_wiggle_chance,
                        corridor_wiggle_step_max,
                        y0,
                        x0,
                        x1,
                    );
                    carve_corridor_vertical(
                        &mut rng,
                        &mut floor_map,
                        &mut corridor_map,
                        tile_width,
                        tile_height,
                        corridor_radius,
                        corridor_wiggle_chance,
                        corridor_wiggle_step_max,
                        x1,
                        y0,
                        y1,
                    );
                }
            }
        }

        // Add hazards (lava pits) in some rooms
        for _ in 0..rooms.len().max(1) {
            if rng.random_range(0..100) > 40 { continue; }
            let r = rooms[rng.random_range(0..rooms.len())];
            let pit_w = rng.random_range(2..=4);
            let pit_h = rng.random_range(2..=4);
            // Guard against pits too large for the room
            if pit_w >= r.w - 2 || pit_h >= r.h - 2 { continue; }
            let px = rng.random_range(r.x + 1..=(r.x + r.w - pit_w - 1));
            let py = rng.random_range(r.y + 1..=(r.y + r.h - pit_h - 1));

            for yy in py..py + pit_h {
                for xx in px..px + pit_w {
                    if (xx as usize) < tile_width && (yy as usize) < tile_height {
                        let idx = (yy as usize) * tile_width + (xx as usize);
                        hazard_map[idx] = true;
                        floor_map[idx] = false;
                    }
                }
            }
        }

        // Scatter pillars
        let pillar_attempts = (rooms.len() * 4).max(0);
        for _ in 0..pillar_attempts {
            if rooms.is_empty() { break; }
            let r = rooms[rng.random_range(0..rooms.len())];
            if r.w <= 0 || r.h <= 0 { continue; }
            let px = rng.random_range(r.x..=(r.x + r.w - 1)) as usize;
            let py = rng.random_range(r.y..=(r.y + r.h - 1)) as usize;
            if px < tile_width && py < tile_height {
                let idx = py * tile_width + px;
                if corridor_map[idx] || hazard_map[idx] { continue; }
                if rng.random_range(0..100) > 35 {
                    floor_map[idx] = false;
                }
            }
        }

        // Create wall outlines only (around floor tiles and corridors)
        let mut wall_map = vec![false; tile_map_size];
        for y in 0..tile_height {
            for x in 0..tile_width {
                if !floor_map[y * tile_width + x] && !hazard_map[y * tile_width + x] { continue; }
                // Mark adjacent non-floor tiles as walls (cardinal + diagonal corners)
                let neighbors = [
                    (y.saturating_sub(1), x),
                    (y + 1, x),
                    (y, x.saturating_sub(1)),
                    (y, x + 1),
                    // Diagonal corners for connected walls
                    (y.saturating_sub(1), x.saturating_sub(1)),
                    (y.saturating_sub(1), x + 1),
                    (y + 1, x.saturating_sub(1)),
                    (y + 1, x + 1),
                ];
                for (ny, nx) in neighbors {
                    if ny < tile_height && nx < tile_width {
                        let idx = ny * tile_width + nx;
                        if !floor_map[idx] && !hazard_map[idx] {
                            wall_map[idx] = true;
                        }
                    }
                }
            }
        }

        let floor_delete_other_tiles = delete_other_tiles_by_tile_id.get("floor_tile_id");
        let lava_delete_other_tiles = delete_other_tiles_by_tile_id.get("lava_tile_id");
        let disable_floor_terrgen = terrgen_disable_by_tile_id.should_disable_for("floor_tile_id");
        let disable_lava_terrgen = terrgen_disable_by_tile_id.should_disable_for("lava_tile_id");
        let mut chunk_tiles: Vec<(ChunkPos, TilesFromBuilder)> = Vec::with_capacity(chunk_positions.len());
        let mut terrgen_disabled_gpos_for_chunks = Vec::with_capacity(chunk_positions.len());
        for &chunk_pos in chunk_positions {
            tiles4chunk.clear();
            let mut blocked_gpos = HashSet::default();
            for tile_pos in chunk_pos.get_tilepositions_within_chunk() {
                let local_tile = tile_pos.0 - origin_tile.0;
                if local_tile.x < 0 || local_tile.y < 0 { continue; }
                let idx_x = local_tile.x as usize;
                let idx_y = local_tile.y as usize;
                if idx_x >= tile_width || idx_y >= tile_height { continue; }
                let map_idx = idx_y * tile_width + idx_x;

                if hazard_map[map_idx] {
                    tiles4chunk.push((tile_pos, lava_entity, lava_delete_other_tiles.clone()));
                    if disable_lava_terrgen {
                        let size = ezero_size_query.get(lava_entity.0).copied().unwrap_or_default().inner();
                        extend_occupied_gpos(&mut blocked_gpos, tile_pos, size);
                    }
                } else if floor_map[map_idx] {
                    tiles4chunk.push((tile_pos, floor_entity, floor_delete_other_tiles.clone()));
                    if disable_floor_terrgen {
                        let size = ezero_size_query.get(floor_entity.0).copied().unwrap_or_default().inner();
                        extend_occupied_gpos(&mut blocked_gpos, tile_pos, size);
                    }
                } else if wall_map[map_idx] {
                    tiles4chunk.push((tile_pos, wall_entity, None));
                }
            }
            chunk_tiles.push((chunk_pos, std::mem::take(&mut *tiles4chunk)));
            terrgen_disabled_gpos_for_chunks.push((chunk_pos, blocked_gpos));
        }
        compliances_to_emit.push(StructureBuildCompliance {
            i: build_order.i,
            structure_gen_cfg_ent: build_order.structured_gen_cfg_ent,
            dimension_ref: build_order.dimension_ref,
            chunks: chunk_tiles,
            terrgen_disabled_gpos_for_chunks,
            terrgen_disabled_for_chunks: Vec::new(),
        });
    }
    writer.write_batch(compliances_to_emit.drain(..));
}
