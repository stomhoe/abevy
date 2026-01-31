#[allow(unused_imports)] use bevy::prelude::*;

use common::common_components::HashId;
use game_common::game_common_components::{ArgsMap, EntityZeroRef};
use rand::{Rng, SeedableRng};
use ::tilemap_shared::*;

use crate::regioning::{
    dungeoning_utils::{
        carve_corridor_horizontal, carve_corridor_vertical, carve_room_circle, carve_room_rectangle,
        carve_room_triangle,
    },
    regioning_components::*,
    regioning_messages::{StructureBuildCompliance, StructurePrepareTilesOrder},
    regioning_sgc_components::StructuredGenConfig,
};
use crate::tile::{tile_components::DeleteOtherTiles, tile_resources::TileEzerosMap};

const CHAMBERS_CORRIDORS: HashId = HashId::hash("chamberscorridors");

/// Cache for Corridor/Chambers dungeon configuration
#[derive(Debug, Clone)]
pub struct CorridorConfig {
    rect_weight: f32,
    circle_weight: f32,
    triangle_weight: f32,
    same_shape_chance: f32,
    corridor_wiggle_chance: f32,
    corridor_wiggle_step_max: i32,
    corridor_detour_chance: f32,
    corridor_detour_max_offset: i32,
}

impl CorridorConfig {
    fn from_args(args: &ArgsMap) -> Self {
        let rect_weight: f32 = args.parse_arg("room_shape_weight_rectangle", 1.0);
        let circle_weight: f32 = args.parse_arg("room_shape_weight_circle", 1.0);
        let triangle_weight: f32 = args.parse_arg("room_shape_weight_triangle", 1.0);
        let same_shape_chance: f32 = args.parse_arg("room_same_shape_chance", 0.25);
        let corridor_wiggle_chance: f32 = args.parse_arg("corridor_wiggle_chance", 0.0);
        let corridor_wiggle_step_max: i32 = args.parse_arg("corridor_wiggle_step_max", 1);
        let corridor_detour_chance: f32 = args.parse_arg("corridor_detour_chance", 0.0);
        let corridor_detour_max_offset: i32 = args.parse_arg("corridor_detour_max_offset", 0);

        Self {
            rect_weight: rect_weight.max(0.0),
            circle_weight: circle_weight.max(0.0),
            triangle_weight: triangle_weight.max(0.0),
            same_shape_chance: same_shape_chance.clamp(0.0, 1.0),
            corridor_wiggle_chance: corridor_wiggle_chance.clamp(0.0, 1.0),
            corridor_wiggle_step_max: corridor_wiggle_step_max.clamp(0, 4),
            corridor_detour_chance: corridor_detour_chance.clamp(0.0, 1.0),
            corridor_detour_max_offset: corridor_detour_max_offset.clamp(0, 32),
        }
    }
}

/// Cache for tile IDs used in Corridor dungeons
#[derive(Debug, Clone)]
pub struct CorridorTileIds {
    floor_tile_id: HashId,
    wall_tile_id: HashId,
    lava_tile_id: Option<HashId>,
}

impl CorridorTileIds {
    fn from_args(args: &ArgsMap) -> Self {
        let floor_tile_id = args
            .get("floor_tile_id")
            .and_then(|v| v.first())
            .map(|s| HashId::hash(s.as_str()))
            .unwrap_or_else(|| HashId::hash("dunewbie"));
        
        let wall_tile_id = args
            .get("wall_tile_id")
            .and_then(|v| v.first())
            .map(|s| HashId::hash(s.as_str()))
            .unwrap_or_else(|| HashId::hash("gray"));
        
        let lava_tile_id = args
            .get("lava_tile_id")
            .and_then(|v| v.first())
            .map(|s| HashId::hash(s.as_str()));

        Self {
            floor_tile_id,
            wall_tile_id,
            lava_tile_id,
        }
    }
}

#[allow(unused_parens, )]
pub fn corridor_dungeon_building_system(
    mut reader: MessageReader<StructurePrepareTilesOrder>,
    structured_gens: Query<(&StructuredGenConfig,),()>,
    mut writer: MessageWriter<StructureBuildCompliance>,
    ezeros_map: Res<TileEzerosMap>,
    settings: Single<&GlobalGenSettings>,
    dimension_hash: Query<&HashId>,
    mut config_cache: Local<Option<CorridorConfig>>,
    mut tile_ids_cache: Local<Option<CorridorTileIds>>,
) {
    let mut compliances_to_emit = Vec::new();
    for build_order in reader.read() {

        let Ok((structured_gen_cfg,)) = structured_gens.get(build_order.structured_gen_cfg_ent) else { continue; };

        if structured_gen_cfg.structure_hash_id() != CHAMBERS_CORRIDORS {
            continue;
        }
        let tile_ids = tile_ids_cache.get_or_insert_with(|| CorridorTileIds::from_args(&structured_gen_cfg.args));
        let floor_tile_id = tile_ids.floor_tile_id;
        let wall_tile_id = tile_ids.wall_tile_id;
        let lava_tile_id = tile_ids.lava_tile_id;

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
        }

        // Cache config on first call
        let cfg = config_cache.get_or_insert_with(|| CorridorConfig::from_args(&structured_gen_cfg.args));

        let rect_weight = cfg.rect_weight;
        let circle_weight = cfg.circle_weight;
        let triangle_weight = cfg.triangle_weight;
        let same_shape_chance = cfg.same_shape_chance;
        let corridor_wiggle_chance = cfg.corridor_wiggle_chance;
        let corridor_wiggle_step_max = cfg.corridor_wiggle_step_max;
        let corridor_detour_chance = cfg.corridor_detour_chance;
        let corridor_detour_max_offset = cfg.corridor_detour_max_offset;

        let pick_shape = |rng: &mut rand_pcg::Pcg64Mcg| -> RoomShape {
            let total = rect_weight + circle_weight + triangle_weight;
            if total <= 0.0 {
                return RoomShape::Rectangle;
            }
            let roll = rng.random_range(0.0..total);
            if roll < rect_weight {
                RoomShape::Rectangle
            } else if roll < rect_weight + circle_weight {
                RoomShape::Circle
            } else {
                RoomShape::Triangle
            }
        };

        let use_same_shape = rng.random_range(0.0..1.0) < same_shape_chance;
        let same_shape = if use_same_shape { Some(pick_shape(&mut rng)) } else { None };

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

        #[derive(Clone, Copy)]
        struct Room { x: i32, y: i32, w: i32, h: i32 }
        let mut rooms: Vec<Room> = Vec::new();
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
                    carve_room_triangle(&mut floor_map, tile_width, tile_height, rm.x, rm.y, rm.w, rm.h);
                }
            }
        }

        let mut corridor_map = vec![false; tile_map_size];
        let corridor_radius = 1;
        
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
        if lava_entity.is_some() {
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

        let delete_template = DeleteOtherTiles::default();
        for &chunk_pos in chunk_positions {
            let mut tiles4chunk: TilesFromBuilder = Vec::new();
            for tile_pos in chunk_pos.get_tilepositions_within_chunk(OplistSize::default()) {
                let local_tile = tile_pos.0 - origin_tile.0;
                if local_tile.x < 0 || local_tile.y < 0 { continue; }
                let idx_x = local_tile.x as usize;
                let idx_y = local_tile.y as usize;
                if idx_x >= tile_width || idx_y >= tile_height { continue; }
                let map_idx = idx_y * tile_width + idx_x;
                
                if hazard_map[map_idx] {
                    let ezero_ref = if let Some(lava) = lava_entity { lava } else { wall_entity };
                    tiles4chunk.push((tile_pos, ezero_ref, Some(delete_template.clone())));
                } else if floor_map[map_idx] {
                    tiles4chunk.push((tile_pos, floor_entity, Some(delete_template.clone())));
                } else if wall_map[map_idx] {
                    tiles4chunk.push((tile_pos, wall_entity, Some(delete_template.clone())));
                }
            }
            if !tiles4chunk.is_empty() {
                compliances_to_emit.push(StructureBuildCompliance {
                    structure_gen_cfg_ent: build_order.structured_gen_cfg_ent,
                    dimension_ref: build_order.dimension_ref,
                    chunk_pos,
                    tiles: tiles4chunk,
                });
            }
        }
        let region_pos = chunk_positions[0].to_region_pos();
        debug!(target: "dungeoning", "Spawned advanced BSP dungeon: {} rooms, {} chunks at {:?}", rooms.len(), chunk_positions.len(), region_pos);
    }
    writer.write_batch(compliances_to_emit);
}
