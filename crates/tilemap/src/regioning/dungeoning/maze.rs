#[allow(unused_imports)] use bevy::prelude::*;

use common::common_components::HashId;
use common::common_tag_components::TagSet;
use game_common::game_common_components::EntityZeroRef;
use rand::{Rng, SeedableRng};
use ::tilemap_shared::*;

use crate::regioning::{    regioning_components::*,
    regioning_messages::{StructureBuildCompliance, SgcPrepareTilesOrder},
    regioning_sgc_components::StructuredGenConfig,
};
use crate::tile::tile_resources::*;
use super::dungeoning_ids::MAZE;

#[allow(unused_parens, )]
pub fn maze_dungeon_building_system(
    mut reader: MessageReader<SgcPrepareTilesOrder>,
    structured_gens: Query<(&StructuredGenConfig,),()>,
    mut writer: MessageWriter<StructureBuildCompliance>,
    ezeros_map: Res<TileEntityMap>,
    ezero_tag_query: Query<Option<&'static TagSet>, (With<game_common::game_common_components::EntityZero>, common::AnyDisabling)>,
    settings: Query<&GlobalGenSettings>,
    dimension_hash: Query<&HashId>,
) {
    let Ok(settings) = settings.single() else {
        return;
    };
    let mut compliances_to_emit = Vec::new();
    for build_order in reader.read() {
        let Ok((structured_gen_cfg,)) = structured_gens.get(build_order.structured_gen_cfg_ent) else { continue; };

        if structured_gen_cfg.structure_hash_id() != MAZE {
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
            .map(|s| HashId::hash(s.as_str()));
        let delete_other_tiles_by_tile_id = crate::regioning::dungeoning_utils::DeleteOtherTilesConfigMap::from_args(&structured_gen_cfg.args);

        let floor_entity = match ezeros_map.0.get_cloned(floor_tile_id) {
            Ok(entity) => EntityZeroRef(entity),
            Err(_) => {
                error!(target: "dungeoning", "TileEzero '{:?}' not found for maze dungeon", floor_tile_id);
                continue;
            }
        };

        let wall_entity = match ezeros_map.0.get_cloned(wall_tile_id) {
            Ok(entity) => EntityZeroRef(entity),
            Err(_) => {
                error!(target: "dungeoning", "TileEzero '{:?}' not found for maze dungeon", wall_tile_id);
                continue;
            }
        };

        let lava_entity = lava_tile_id.and_then(|id| ezeros_map.0.get_cloned(id).ok()).map(EntityZeroRef);

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
            error!(target: "dungeoning", "Dimension entity {:?} has no HashId component for maze dungeon", build_order.dimension_ref);
            continue;
        };

        let seed = chunk_positions[0].hash_value(&settings, dimension_hash, 1);
        let mut rng = rand_pcg::Pcg64Mcg::seed_from_u64(seed);

        let corridor_wiggle_chance: f32 = structured_gen_cfg
            .args
            .parse_arg("corridor_wiggle_chance", 0.0);
        let corridor_wiggle_chance = corridor_wiggle_chance.clamp(0.0, 1.0);

        #[derive(Clone, Copy)]
        enum ShapeType {
            Circle,
            Triangle,
            Hexagon,
            Square,
        }

        let point_in_shape = |px: i32, py: i32, cx: i32, cy: i32, radius: i32, shape: ShapeType| -> bool {
            let dx = px - cx;
            let dy = py - cy;
            match shape {
                ShapeType::Circle => dx * dx + dy * dy <= radius * radius,
                ShapeType::Square => dx.abs() <= radius && dy.abs() <= radius,
                ShapeType::Hexagon => {
                    let abs_dx = dx.abs();
                    let abs_dy = dy.abs();
                    abs_dy <= radius && abs_dx <= (radius * 2) && abs_dy * 2 + abs_dx <= radius * 3
                }
                ShapeType::Triangle => {
                    let abs_dx = dx.abs();
                    if abs_dx > radius { return false; }
                    let height = (radius * 3) / 2;
                    dy >= -height && dy <= radius && (radius - abs_dx.max(0)) * 2 + dy >= 0
                }
            }
        };

        let num_islands = if tile_width > 300 && tile_height > 300 {
            rng.random_range(4..=6)
        } else if tile_width > 200 && tile_height > 200 {
            rng.random_range(3..=5)
        } else if tile_width > 100 && tile_height > 100 {
            rng.random_range(2..=4)
        } else {
            1
        };

        let mut island_seeds: Vec<(usize, usize, ShapeType)> = Vec::new();
        let margin = 4;
        if tile_width >= margin * 2 && tile_height >= margin * 2 {
            for _ in 0..num_islands {
                let mut valid_pos = false;
                let mut attempts = 0;
                let min_island_distance = if num_islands > 1 {
                    (tile_width.min(tile_height) / 4).max(90) as i32
                } else {
                    0
                };

                while !valid_pos && attempts < 30 {
                    let island_x = rng.random_range(margin..tile_width.saturating_sub(margin));
                    let island_y = rng.random_range(margin..tile_height.saturating_sub(margin));

                    let mut too_close = false;
                    for &(ex_x, ex_y, _) in &island_seeds {
                        let dx = (island_x as i32 - ex_x as i32).abs();
                        let dy = (island_y as i32 - ex_y as i32).abs();
                        if dx * dx + dy * dy < min_island_distance * min_island_distance {
                            too_close = true;
                            break;
                        }
                    }

                    if !too_close {
                        let shape_types = [
                            ShapeType::Circle,
                            ShapeType::Triangle,
                            ShapeType::Hexagon,
                            ShapeType::Square,
                        ];
                        let shape_idx = rng.random_range(0..shape_types.len());
                        let shape = shape_types[shape_idx];
                        island_seeds.push((island_x, island_y, shape));
                        valid_pos = true;
                    }
                    attempts += 1;
                }
            }
        }

        for &(island_cx, island_cy, shape) in &island_seeds {
            let max_available_width = (tile_width as i32 - 8).max(10) as usize;
            let max_available_height = (tile_height as i32 - 8).max(10) as usize;
            let island_size_multiplier = if island_seeds.len() == 1 { 0.95 } else { 0.75 };
            let max_island_size = ((max_available_width.min(max_available_height) as f32 * island_size_multiplier) as usize).max(50);
            let min_island_size = (max_island_size / 3).max(30);

            let island_size = rng.random_range(min_island_size..=max_island_size);
            let maze_width = island_size | 1;
            let maze_height = island_size | 1;
            let island_radius = island_size / 2;

            let island_x_start = (island_cx as i32 - maze_width as i32 / 2).max(0) as usize;
            let island_y_start = (island_cy as i32 - maze_height as i32 / 2).max(0) as usize;
            let island_x_end = (island_x_start + maze_width).min(tile_width - 1);
            let island_y_end = (island_y_start + maze_height).min(tile_height - 1);
            let actual_maze_width = island_x_end - island_x_start;
            let actual_maze_height = island_y_end - island_y_start;

            if actual_maze_width < 3 || actual_maze_height < 3 { continue; }

            let mut maze = vec![1; actual_maze_width * actual_maze_height];
            let mut stack: Vec<(usize, usize)> = Vec::new();
            let start_range_x = ((actual_maze_width.saturating_sub(2)) / 2).max(1);
            let start_range_y = ((actual_maze_height.saturating_sub(2)) / 2).max(1);
            let start_x = (rng.random_range(0..start_range_x) * 2 + 1).min(actual_maze_width - 2);
            let start_y = (rng.random_range(0..start_range_y) * 2 + 1).min(actual_maze_height - 2);

            if start_x < actual_maze_width && start_y < actual_maze_height {
                maze[start_y * actual_maze_width + start_x] = 0;
                stack.push((start_x, start_y));

                while let Some((cx, cy)) = stack.last().copied() {
                    let mut directions: [usize; 4] = [0, 1, 2, 3];
                    for i in 0..4 {
                        let j = rng.random_range(0..4);
                        directions.swap(i, j);
                    }

                    let mut found = false;
                    for dir in directions {
                        let (dx, dy) = match dir {
                            0 => (2, 0),
                            1 => (-2, 0),
                            2 => (0, 2),
                            _ => (0, -2),
                        };

                        let nx = (cx as i32 + dx) as usize;
                        let ny = (cy as i32 + dy) as usize;

                        if nx > 0 && nx < actual_maze_width && ny > 0 && ny < actual_maze_height {
                            let idx = ny * actual_maze_width + nx;
                            if maze[idx] == 1 {
                                let wall_x = (cx as i32 + dx / 2) as usize;
                                let wall_y = (cy as i32 + dy / 2) as usize;
                                let wall_idx = wall_y * actual_maze_width + wall_x;
                                maze[wall_idx] = 0;
                                maze[idx] = 0;
                                stack.push((nx, ny));
                                found = true;
                                break;
                            }
                        }
                    }

                    if !found {
                        stack.pop();
                    }
                }
            }

            for my in 0..actual_maze_height {
                for mx in 0..actual_maze_width {
                    let tx = island_x_start + mx;
                    let ty = island_y_start + my;
                    if tx < tile_width && ty < tile_height {
                        let px = tx as i32;
                        let py = ty as i32;
                        let cx = island_cx as i32;
                        let cy = island_cy as i32;
                        if point_in_shape(px, py, cx, cy, island_radius as i32, shape) {
                            let map_idx = ty * tile_width + tx;
                            if maze[my * actual_maze_width + mx] == 0 {
                                floor_map[map_idx] = true;
                            }
                        }
                    }
                }
            }

            let mut room_positions: Vec<(usize, usize)> = Vec::new();
            let num_regular_rooms = rng.random_range(2..=4);
            for _ in 0..num_regular_rooms {
                if actual_maze_width < 12 || actual_maze_height < 12 { break; }

                let room_w = rng.random_range(6..=12).min(actual_maze_width.saturating_sub(4));
                let room_h = rng.random_range(6..=12).min(actual_maze_height.saturating_sub(4));
                let max_room_x = actual_maze_width.saturating_sub(room_w + 2);
                let max_room_y = actual_maze_height.saturating_sub(room_h + 2);
                if max_room_x <= 2 || max_room_y <= 2 { continue; }

                let room_x = island_x_start + rng.random_range(2..max_room_x);
                let room_y = island_y_start + rng.random_range(2..max_room_y);

                for ry in 0..room_h {
                    for rx in 0..room_w {
                        let tx = room_x + rx;
                        let ty = room_y + ry;
                        if tx < tile_width && ty < tile_height {
                            floor_map[ty * tile_width + tx] = true;
                        }
                    }
                }

                room_positions.push((room_x + room_w / 2, room_y + room_h / 2));
            }

            let num_rooms = rng.random_range(2..=6);
            for _ in 0..num_rooms {
                if rng.random_range(0..100) > 40 { continue; }

                let room_x = island_x_start + rng.random_range(2..actual_maze_width.saturating_sub(2));
                let room_y = island_y_start + rng.random_range(2..actual_maze_height.saturating_sub(2));
                let room_size = rng.random_range(4..=8);
                let room_shape = rng.random_range(0..2);

                match room_shape {
                    1 => {
                        let radius = room_size / 2;
                        for ry in 0..room_size {
                            for rx in 0..room_size {
                                let dx = rx as i32 - radius as i32;
                                let dy = ry as i32 - radius as i32;
                                if dx * dx + dy * dy <= (radius as i32) * (radius as i32) {
                                    let tx = room_x + rx;
                                    let ty = room_y + ry;
                                    if tx < tile_width && ty < tile_height {
                                        let map_idx = ty * tile_width + tx;
                                        floor_map[map_idx] = true;
                                    }
                                }
                            }
                        }
                    }
                    _ => {
                        for ry in 0..room_size {
                            for rx in 0..room_size {
                                let tx = room_x + rx;
                                let ty = room_y + ry;
                                if tx < tile_width && ty < tile_height {
                                    let map_idx = ty * tile_width + tx;
                                    floor_map[map_idx] = true;
                                }
                            }
                        }
                    }
                }
                room_positions.push((room_x + room_size / 2, room_y + room_size / 2));
            }

            let num_breaks = rng.random_range(2..=5);
            for _ in 0..num_breaks {
                let break_x = island_x_start + rng.random_range(0..actual_maze_width);
                let break_y = island_y_start + rng.random_range(0..actual_maze_height);
                let break_size = rng.random_range(1..=2);

                for dy in 0..break_size {
                    for dx in 0..break_size {
                        let tx = break_x + dx;
                        let ty = break_y + dy;
                        if tx < tile_width && ty < tile_height && rng.random_range(0..100) > 50 {
                            let map_idx = ty * tile_width + tx;
                            floor_map[map_idx] = true;
                        }
                    }
                }
            }

            for &(room_x, room_y) in &room_positions {
                let search_radius = 8;
                let mut found_connection = false;

                for dy in -search_radius..=search_radius {
                    if found_connection { break; }
                    for dx in -search_radius..=search_radius {
                        let cx = (room_x as i32 + dx).max(0) as usize;
                        let cy = (room_y as i32 + dy).max(0) as usize;
                        if cx >= tile_width || cy >= tile_height { continue; }

                        let idx = cy * tile_width + cx;
                        if floor_map[idx] && (dx.abs() > 2 || dy.abs() > 2) {
                            let corridor_x_start = (room_x as i32).min(cx as i32);
                            let corridor_x_end = (room_x as i32).max(cx as i32);
                            let corridor_y_start = (room_y as i32).min(cy as i32);
                            let corridor_y_end = (room_y as i32).max(cy as i32);

                            for x in corridor_x_start..=corridor_x_end {
                                let mut yy = room_y as i32;
                                if rng.random_range(0.0..1.0) < corridor_wiggle_chance {
                                    yy += rng.random_range(-1..=1);
                                }
                                if x >= 0 && (x as usize) < tile_width && yy >= 0 && (yy as usize) < tile_height {
                                    floor_map[(yy as usize * tile_width) + (x as usize)] = true;
                                }
                            }

                            for y in corridor_y_start..=corridor_y_end {
                                let mut xx = cx as i32;
                                if rng.random_range(0.0..1.0) < corridor_wiggle_chance {
                                    xx += rng.random_range(-1..=1);
                                }
                                if xx >= 0 && (xx as usize) < tile_width && y >= 0 && (y as usize) < tile_height {
                                    floor_map[(y as usize * tile_width) + (xx as usize)] = true;
                                }
                            }

                            found_connection = true;
                            break;
                        }
                    }
                }
            }
        }

        if island_seeds.len() > 1 {
            let scan_size = 24;
            for start_y in (0..tile_height).step_by(scan_size) {
                for start_x in (0..tile_width).step_by(scan_size) {
                    let end_x = (start_x + scan_size).min(tile_width);
                    let end_y = (start_y + scan_size).min(tile_height);

                    let mut wall_count = 0;
                    let region_size = (end_x - start_x) * (end_y - start_y);

                    for y in start_y..end_y {
                        for x in start_x..end_x {
                            if !floor_map[y * tile_width + x] {
                                wall_count += 1;
                            }
                        }
                    }

                    let wall_density = (wall_count as f32) / (region_size as f32);
                    if wall_density > 0.02 && wall_density < 0.20 {
                        for y in start_y..end_y {
                            for x in start_x..end_x {
                                floor_map[y * tile_width + x] = true;
                            }
                        }
                    }
                }
            }
        }

        if island_seeds.len() > 1 {
            let mut sorted_islands = island_seeds.clone();
            sorted_islands.sort_by_key(|c| (c.0, c.1));

            for i in 1..sorted_islands.len() {
                let from = sorted_islands[i - 1];
                let to = sorted_islands[i];

                let (sx, ex) = if from.0 <= to.0 {
                    (from.0, to.0)
                } else {
                    (to.0, from.0)
                };

                for x in sx..=ex {
                    let mut y = from.1 as i32;
                    if rng.random_range(0.0..1.0) < corridor_wiggle_chance {
                        y += rng.random_range(-1..=1);
                    }
                    if y >= 0 && (y as usize) < tile_height && x < tile_width {
                        floor_map[(y as usize) * tile_width + x] = true;
                    }
                }

                let (sy, ey) = if from.1 <= to.1 {
                    (from.1, to.1)
                } else {
                    (to.1, from.1)
                };

                for y in sy..=ey {
                    let mut x = to.0 as i32;
                    if rng.random_range(0.0..1.0) < corridor_wiggle_chance {
                        x += rng.random_range(-1..=1);
                    }
                    if x >= 0 && (x as usize) < tile_width && y < tile_height {
                        floor_map[y * tile_width + (x as usize)] = true;
                    }
                }
            }
        }

        if lava_entity.is_some() {
            let hazard_count = rng.random_range(0..=4);
            for _ in 0..hazard_count {
                if rng.random_range(0..100) > 40 { continue; }

                let haz_x = rng.random_range(4..tile_width.saturating_sub(4));
                let haz_y = rng.random_range(4..tile_height.saturating_sub(4));
                let hazard_radius = rng.random_range(1..=3) as i32;

                for dy in -hazard_radius..=hazard_radius {
                    for dx in -hazard_radius..=hazard_radius {
                        let dist_sq = dx * dx + dy * dy;
                        if dist_sq > hazard_radius * hazard_radius { continue; }

                        let hx = (haz_x as i32 + dx).max(0) as usize;
                        let hy = (haz_y as i32 + dy).max(0) as usize;
                        if hx < tile_width && hy < tile_height {
                            let idx = hy * tile_width + hx;
                            if floor_map[idx] {
                                hazard_map[idx] = true;
                                floor_map[idx] = false;
                            }
                        }
                    }
                }
            }
        }

        let mut wall_map = vec![false; tile_map_size];
        for y in 0..tile_height {
            for x in 0..tile_width {
                if !floor_map[y * tile_width + x] && !hazard_map[y * tile_width + x] { continue; }
                let neighbors = [
                    (y.saturating_sub(1), x),
                    (y + 1, x),
                    (y, x.saturating_sub(1)),
                    (y, x + 1),
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

        let floor_delete_other_tiles = delete_other_tiles_by_tile_id.get(&floor_tile_id, ezero_tag_query.get(floor_entity.0).ok().flatten());
        let wall_delete_other_tiles = delete_other_tiles_by_tile_id.get(&wall_tile_id, ezero_tag_query.get(wall_entity.0).ok().flatten());
        let lava_delete_other_tiles = lava_tile_id.and_then(|tile_id| {
            lava_entity.and_then(|lava_entity| delete_other_tiles_by_tile_id.get(&tile_id, ezero_tag_query.get(lava_entity.0).ok().flatten()))
        });
        let mut chunk_tiles: Vec<(ChunkPos, TilesFromBuilder)> = Vec::with_capacity(chunk_positions.len());
        for &chunk_pos in chunk_positions {
            let mut tiles4chunk: TilesFromBuilder = Vec::new();
            for tile_pos in chunk_pos.get_tilepositions_within_chunk() {
                let local_tile = tile_pos.0 - origin_tile.0;
                if local_tile.x < 0 || local_tile.y < 0 { continue; }
                let idx_x = local_tile.x as usize;
                let idx_y = local_tile.y as usize;
                if idx_x >= tile_width || idx_y >= tile_height { continue; }
                let map_idx = idx_y * tile_width + idx_x;

                if hazard_map[map_idx] {
                    let ezero_ref = if let Some(lava) = lava_entity { lava } else { wall_entity };
                    let delete_other_tiles = if lava_entity.is_some() {
                        lava_delete_other_tiles.clone()
                    } else {
                        wall_delete_other_tiles.clone()
                    };
                    tiles4chunk.push((tile_pos, ezero_ref, delete_other_tiles));
                } else if floor_map[map_idx] {
                    tiles4chunk.push((tile_pos, floor_entity, floor_delete_other_tiles.clone()));
                } else if wall_map[map_idx] {
                    tiles4chunk.push((tile_pos, wall_entity, wall_delete_other_tiles.clone()));
                }
            }

            chunk_tiles.push((chunk_pos, tiles4chunk));
        }
        compliances_to_emit.push(StructureBuildCompliance {
            i: build_order.i,
            structure_gen_cfg_ent: build_order.structured_gen_cfg_ent,
            dimension_ref: build_order.dimension_ref,
            chunks: chunk_tiles,
            terrgen_disabled_for_chunks: Vec::new(),

        });
        let region_pos = chunk_positions[0].to_region_pos();
        debug!(target: "dungeoning", "Spawned minotaur maze with {} islands across {} chunks at {:?}", island_seeds.len(), chunk_positions.len(), region_pos);
    }
    writer.write_batch(compliances_to_emit);
}
