
use bevy::platform::collections::{HashMap, HashSet};
#[allow(unused_imports)] use bevy::prelude::*;

use common::common_components::HashId;
use dimension_shared::DimensionRef;
use game_common::game_common_components::EntityZeroRef;
use rand::{Rng, SeedableRng};
use rand_distr::{Normal, Distribution};
use ::tilemap_shared::*;

use crate::{regioning::{regioning_components::*, regioning_messages::{ClaimedChunks, OfferChunk, StructureBuildCompliance, StructurePrepareTilesOrder}, regioning_sgc_components::StructuredGenConfig}, tile::{tile_components::DeleteOtherTiles, tile_resources::TileEzerosMap}, };


const DRUNKWALK: HashId = HashId::hash("drunkwalk");
const IDK: HashId = HashId::hash("idk");

const ADMITTED_STRUCTURE_IDS: &[HashId] = &[
    DRUNKWALK,
    IDK,
];

#[allow(unused_parens)]
pub fn claim_chunks_for_various_dungeon_types(
    mut offered_chunks: MessageReader<OfferChunk>,
    mut writer: MessageWriter<ClaimedChunks>,
    region_dimension: Query<&DimensionRef>,
    structured_gens: Query<(&StructuredGenConfig,)>,
    dimension_hash: Query<&HashId>,
    settings: Single<&GlobalGenSettings>,
) {
    let mut claims_to_emit = Vec::new();
    let mut already_claimed: HashSet<ChunkPos> = HashSet::new();
    
    for offer in offered_chunks.read() {
        info!(target: "dungeoning", "Processing OfferChunk {:?} for region entity {:?}", offer.i, offer.region_ent);
        let Ok((structured_gen_cfg,)) = structured_gens.get(offer.structured_gen_cfg_ent)
        else { 
            error!(target: "dungeoning", "StructuredGenConfig entity {:?} not found when making DrunkwalkDungeon, skipping structure spawn", offer.structured_gen_cfg_ent);
            continue; };

        let id = structured_gen_cfg.hash;
        
        if !ADMITTED_STRUCTURE_IDS.contains(&id) {
            trace!(target: "dungeoning", "StructuredGenConfig entity {:?} is not in admitted structures, skipping", offer.structured_gen_cfg_ent);
            continue;
        }
        let center_chunk = offer.start_pos;

        let Ok(dimension_ref) = region_dimension.get(offer.region_ent) else {
            warn!(target: "dungeoning", "Region entity {:?} has no DimensionRef component when claiming chunks for structure spawn, skipping", offer.region_ent);
            continue;
        };

        let Ok(dimension_hash) = dimension_hash.get(dimension_ref.0) else {
            warn!(target: "dungeoning", "Dimension entity {:?} has no HashId component when claiming chunks for structure spawn, skipping", dimension_ref.0);
            continue;
        };

        let seed = center_chunk.hash_value(&settings, 0).wrapping_add(dimension_hash.as_u64());
        let mut rng = rand_pcg::Pcg64Mcg::seed_from_u64(seed);

        // Use normal distribution with mean 4, clamped to [2, 6]
        let side_length = {
            let normal = Normal::new(4.0, 1.5).unwrap();
            (normal.sample(&mut rng) as i32).clamp(2, 7)
        };
        let half_spread = side_length / 2;
        let region_pos = center_chunk.to_region_pos();

        // Compute smart offsets: start with ideal centered range, then shift away from region bounds
        let mut start_offset = -half_spread;
        let mut end_offset = start_offset + side_length - 1;

        // Try candidate positions, shifting right/up if we'd go out of bounds
        let mut valid_positions = Vec::new();
        for dy in start_offset..=end_offset {
            for dx in start_offset..=end_offset {
                let candidate = center_chunk + IVec2::new(dx, dy);
                if region_pos.contains_chunkpos(candidate) {
                    valid_positions.push(candidate);
                }
            }
        }

        // If we got a partial square due to boundary, intelligently shift the entire offset range
        if !valid_positions.is_empty() && valid_positions.len() < (side_length * side_length) as usize {
            // Find bounds of what we got vs what we wanted
            let ideal_min_x = center_chunk.x() + start_offset;
            let ideal_max_x = center_chunk.x() + end_offset;
            let ideal_min_y = center_chunk.y() + start_offset;
            let ideal_max_y = center_chunk.y() + end_offset;

            let actual_min_x = valid_positions.iter().map(|p| p.x()).min().unwrap_or(ideal_min_x);
            let actual_max_x = valid_positions.iter().map(|p| p.x()).max().unwrap_or(ideal_max_x);
            let actual_min_y = valid_positions.iter().map(|p| p.y()).min().unwrap_or(ideal_min_y);
            let actual_max_y = valid_positions.iter().map(|p| p.y()).max().unwrap_or(ideal_max_y);

            // Shift offsets to move away from boundary that caused truncation
            if actual_min_x > ideal_min_x {
                let shift = actual_min_x - ideal_min_x;
                start_offset += shift;
                end_offset += shift;
            } else if actual_max_x < ideal_max_x {
                let shift = ideal_max_x - actual_max_x;
                start_offset -= shift;
                end_offset -= shift;
            }

            if actual_min_y > ideal_min_y {
                let shift = actual_min_y - ideal_min_y;
                start_offset += shift;
                end_offset += shift;
            } else if actual_max_y < ideal_max_y {
                let shift = ideal_max_y - actual_max_y;
                start_offset -= shift;
                end_offset -= shift;
            }

            // Recollect with adjusted offsets
            valid_positions.clear();
            for dy in start_offset..=end_offset {
                for dx in start_offset..=end_offset {
                    let candidate = center_chunk + IVec2::new(dx, dy);
                    if region_pos.contains_chunkpos(candidate) {
                        valid_positions.push(candidate);
                    }
                }
            }
        }

        // Filter out already claimed chunks
        let mut chunk_positions: Vec<ChunkPos> = valid_positions
            .into_iter()
            .filter(|pos| !already_claimed.contains(pos))
            .collect();

        if chunk_positions.is_empty() {
            warn!(target: "dungeoning", "No eligible unclaimed chunks around {:?} for ExampleStructure, skipping", center_chunk);
            continue;
        }

        // Track these chunks as claimed
        for &chunk_pos in &chunk_positions {
            already_claimed.insert(chunk_pos);
        }

        chunk_positions.sort_unstable_by_key(|chunk| (chunk.y(), chunk.x()));
        let chunk_count = chunk_positions.len();
        claims_to_emit.push(ClaimedChunks {
            i: offer.i,
            region_ent: offer.region_ent,
            sgc_ent: offer.structured_gen_cfg_ent,
            chunks_gpos: chunk_positions,
            partition_tolerant: false,
        });
        trace!(target: "dungeoning", "Emitting ClaimedChunks for ExampleStructure covering {} chunks around {:?}", chunk_count, center_chunk);
    }
    writer.write_batch(claims_to_emit);
}


#[allow(unused_parens)]
pub fn drunkwalk_dungeon_building_system(
    mut reader: MessageReader<StructurePrepareTilesOrder>,
    structured_gens: Query<(&StructuredGenConfig,),()>,
    mut writer: MessageWriter<StructureBuildCompliance>,
    ezeros_map: Res<TileEzerosMap>,
    dimension_hash: Query<&HashId>,
    settings: Single<&GlobalGenSettings>,
) {
    let mut compliances_to_emit = Vec::new();    
    for build_order in reader.read() {
        let Ok((structured_gen_cfg,)) = structured_gens.get(build_order.structured_gen_cfg_ent) 
        else { 
            error!(target: "dungeoning", "StructuredGenConfig entity {:?} not found when making DrunkwalkDungeon, skipping structure spawn", build_order.structured_gen_cfg_ent);
            continue; };

        if structured_gen_cfg.hash != DRUNKWALK {
            continue;
        }

        const FLOOR_TILE_ID: HashId = HashId::hash("cyan");
        let floor_entity = match ezeros_map.0.get(FLOOR_TILE_ID) {
            Ok(entity) => EntityZeroRef(entity),
            Err(_) => {
                error!(target: "dungeoning", "TileEzero with id '{}' not found in TileEzerosMap when making DrunkwalkDungeon, skipping structure spawn", FLOOR_TILE_ID);
                continue;
            }
        };

        const WALL_TILE_ID: HashId = HashId::hash("purple");
        let wall_entity = match ezeros_map.0.get(WALL_TILE_ID) {
            Ok(entity) => EntityZeroRef(entity),
            Err(_) => {
                error!(target: "dungeoning", "TileEzero with id '{}' not found in TileEzerosMap when making DrunkwalkDungeon, skipping structure spawn", WALL_TILE_ID);
                continue;
            }
        };

        const LAVA_TILE_ID: HashId = HashId::hash("orange");
        let lava_entity = ezeros_map.0.get(LAVA_TILE_ID).ok().map(EntityZeroRef);

        let chunk_positions = &build_order.chunks_gpos;
        if chunk_positions.is_empty() {
            continue;
        }

        let min_chunk_x = chunk_positions.iter().map(|chunk| chunk.x()).min().unwrap();
        let max_chunk_x = chunk_positions.iter().map(|chunk| chunk.x()).max().unwrap();
        let min_chunk_y = chunk_positions.iter().map(|chunk| chunk.y()).min().unwrap();
        let max_chunk_y = chunk_positions.iter().map(|chunk| chunk.y()).max().unwrap();

        let chunk_width = (max_chunk_x - min_chunk_x + 1) as usize;
        let chunk_height = (max_chunk_y - min_chunk_y + 1) as usize;
        let tile_width = chunk_width * ChunkPos::CHUNK_SIZE.x as usize;
        let tile_height = chunk_height * ChunkPos::CHUNK_SIZE.y as usize;
        if tile_width == 0 || tile_height == 0 {
            continue;
        }

        let origin_chunk = ChunkPos::new(min_chunk_x, min_chunk_y);
        let origin_tile = origin_chunk.to_tilepos();

        let tile_map_size = tile_width * tile_height;
        let mut floor_map = vec![false; tile_map_size];
        let mut hazard_map = vec![false; tile_map_size];
        
        let Ok(dimension_hash) = dimension_hash.get(build_order.dimension_ref.0) else {
            error!(target: "dungeoning", "Dimension entity {:?} has no HashId component when making DrunkwalkDungeon, skipping structure spawn", build_order.dimension_ref);
            continue;
        };
        
        let seed = chunk_positions[0].hash_value(&settings, 0).wrapping_add(dimension_hash.as_u64());
        let mut rng = rand_pcg::Pcg64Mcg::seed_from_u64(seed);

        // Multiple drunkwalks with wider paths and more aggressive carving
        let num_walkers = rng.random_range(5..=10);
        let target_floor_tiles = std::cmp::max(1, ((tile_map_size as f32) * 0.45).ceil() as usize);
        let mut carved = 0;

        for _ in 0..num_walkers {
            let mut walker_x = rng.random_range(0..tile_width);
            let mut walker_y = rng.random_range(0..tile_height);
            let walker_steps = rng.random_range(100..350);
            let bias_chance = rng.random_range(15..45);
            let path_width = rng.random_range(2..=4);

            for _ in 0..walker_steps {
                if carved >= target_floor_tiles { break; }
                
                // Carve a wider path
                for dy in -(path_width as i32)..=path_width as i32 {
                    for dx in -(path_width as i32)..=path_width as i32 {
                        let x = (walker_x as i32 + dx).max(0) as usize;
                        let y = (walker_y as i32 + dy).max(0) as usize;
                        if x < tile_width && y < tile_height {
                            let idx = y * tile_width + x;
                            if !floor_map[idx] {
                                floor_map[idx] = true;
                                carved += 1;
                            }
                        }
                    }
                }

                let direction = if rng.random_range(0..100) < bias_chance {
                    rng.random_range(0..4)
                } else {
                    rng.random_range(0..4)
                };

                match direction {
                    0 => if walker_x < tile_width - 1 { walker_x += 1; },
                    1 => if walker_x > 0 { walker_x -= 1; },
                    2 => if walker_y < tile_height - 1 { walker_y += 1; },
                    _ => if walker_y > 0 { walker_y -= 1; },
                }
            }
        }

        // Larger, more ambitious chambers
        #[derive(Clone, Copy)]
        struct Chamber { center_x: usize, center_y: usize }
        let mut chambers: Vec<Chamber> = Vec::new();
        let chamber_count = rng.random_range(6..=12);
        for _ in 0..chamber_count {
            let center_x = rng.random_range(8..tile_width.saturating_sub(8));
            let center_y = rng.random_range(8..tile_height.saturating_sub(8));
            let radius = rng.random_range(4..=10) as i32;

            for dy in -radius..=radius {
                for dx in -radius..=radius {
                    let dist_sq = dx * dx + dy * dy;
                    if dist_sq > radius * radius { continue; }

                    let rx = center_x as i32 + dx;
                    let ry = center_y as i32 + dy;
                    if rx < 0 || ry < 0 { continue; }

                    let rx = rx as usize;
                    let ry = ry as usize;
                    if rx >= tile_width || ry >= tile_height { continue; }

                    floor_map[ry * tile_width + rx] = true;
                }
            }
            chambers.push(Chamber { center_x, center_y });
        }

        // Connect chambers with corridors with random widths
        if chambers.len() > 1 {
            let mut sorted_chambers = chambers.clone();
            sorted_chambers.sort_by_key(|c| (c.center_x, c.center_y));
            
            for i in 1..sorted_chambers.len() {
                let from = sorted_chambers[i - 1];
                let to = sorted_chambers[i];
                let corridor_width = {
                    let normal = Normal::new(1.5, 1.0).unwrap();
                    (normal.sample(&mut rng) as i32).clamp(1, 5) as usize
                };
                
                // Horizontal corridor
                let (sx, ex) = if from.center_x <= to.center_x {
                    (from.center_x, to.center_x)
                } else {
                    (to.center_x, from.center_x)
                };
                
                for x in sx..=ex {
                    for dy in -(corridor_width as i32)..=corridor_width as i32 {
                        let y = (from.center_y as i32 + dy).max(0) as usize;
                        if x < tile_width && y < tile_height {
                            floor_map[y * tile_width + x] = true;
                        }
                    }
                }
                
                // Vertical corridor
                let (sy, ey) = if from.center_y <= to.center_y {
                    (from.center_y, to.center_y)
                } else {
                    (to.center_y, from.center_y)
                };
                
                for y in sy..=ey {
                    for dx in -(corridor_width as i32)..=corridor_width as i32 {
                        let x = (to.center_x as i32 + dx).max(0) as usize;
                        if x < tile_width && y < tile_height {
                            floor_map[y * tile_width + x] = true;
                        }
                    }
                }
            }
        }

        // Add lava hazards in some chambers
        if lava_entity.is_some() {
            for _ in 0..chamber_count.max(2) {
                if rng.random_range(0..100) > 35 { continue; }
                
                let center_x = rng.random_range(8..tile_width.saturating_sub(8));
                let center_y = rng.random_range(8..tile_height.saturating_sub(8));
                let hazard_radius = rng.random_range(3..=6) as i32;

                for dy in -hazard_radius..=hazard_radius {
                    for dx in -hazard_radius..=hazard_radius {
                        let dist_sq = dx * dx + dy * dy;
                        if dist_sq > hazard_radius * hazard_radius { continue; }

                        let hx = center_x as i32 + dx;
                        let hy = center_y as i32 + dy;
                        if hx < 0 || hy < 0 { continue; }

                        let hx = hx as usize;
                        let hy = hy as usize;
                        if hx >= tile_width || hy >= tile_height { continue; }

                        if floor_map[hy * tile_width + hx] {
                            hazard_map[hy * tile_width + hx] = true;
                            floor_map[hy * tile_width + hx] = false;
                        }
                    }
                }
            }
        }

        // Smooth isolated tiles
        let mut smoothed = floor_map.clone();
        for y in 1..tile_height - 1 {
            for x in 1..tile_width - 1 {
                if !floor_map[y * tile_width + x] { continue; }
                let neighbors = [
                    floor_map[(y - 1) * tile_width + x],
                    floor_map[(y + 1) * tile_width + x],
                    floor_map[y * tile_width + (x - 1)],
                    floor_map[y * tile_width + (x + 1)],
                ]
                .iter().filter(|&&b| b).count();

                if neighbors == 0 {
                    smoothed[y * tile_width + x] = false;
                }
            }
        }
        floor_map = smoothed;

        let delete_template = DeleteOtherTiles::default();
        for &chunk_pos in chunk_positions {
            let mut tiles4chunk: TilesFromBuilder = Vec::new();
            for tile_pos in chunk_pos.get_tilepositions_within_chunk(OplistSize::default()) {
                let local_tile = tile_pos.0 - origin_tile.0;
                if local_tile.x < 0 || local_tile.y < 0 {
                    continue;
                }
                let idx_x = local_tile.x as usize;
                let idx_y = local_tile.y as usize;
                if idx_x >= tile_width || idx_y >= tile_height {
                    continue;
                }
                let map_idx = idx_y * tile_width + idx_x;
                let ezero_ref = if hazard_map[map_idx] {
                    if let Some(lava) = lava_entity { lava } else { wall_entity }
                } else if floor_map[map_idx] {
                    floor_entity
                } else {
                    wall_entity
                };
                tiles4chunk.push((tile_pos, ezero_ref, Some(delete_template.clone())));
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
        debug!(target: "dungeoning", "Spawned organic drunkwalk dungeon across {} chunks at {:?}", chunk_positions.len(), region_pos);
    }
    writer.write_batch(compliances_to_emit);
}

#[allow(unused_parens)]
pub fn advanced_dungeon_building_system(
    mut reader: MessageReader<StructurePrepareTilesOrder>,
    structured_gens: Query<(&StructuredGenConfig,),()>,
    mut writer: MessageWriter<StructureBuildCompliance>,
    ezeros_map: Res<TileEzerosMap>,
    settings: Single<&GlobalGenSettings>,
    dimension_hash: Query<&HashId>,
) {
    let mut compliances_to_emit = Vec::new();
    for build_order in reader.read() {

        let Ok((structured_gen_cfg,)) = structured_gens.get(build_order.structured_gen_cfg_ent) else { continue; };

        if structured_gen_cfg.hash != IDK {
            continue;
        }

        const FLOOR_TILE_ID: HashId = HashId::hash("cyan");
        let floor_entity = match ezeros_map.0.get(FLOOR_TILE_ID) {
            Ok(entity) => EntityZeroRef(entity),
            Err(_) => {
                error!(target: "dungeoning", "TileEzero '{}' not found", FLOOR_TILE_ID);
                continue;
            }
        };

        const WALL_TILE_ID: HashId = HashId::hash("purple");
        let wall_entity = match ezeros_map.0.get(WALL_TILE_ID) {
            Ok(entity) => EntityZeroRef(entity),
            Err(_) => {
                error!(target: "dungeoning", "TileEzero '{}' not found", WALL_TILE_ID);
                continue;
            }
        };

        const LAVA_TILE_ID: HashId = HashId::hash("orange");
        let lava_entity = ezeros_map.0.get(LAVA_TILE_ID).ok().map(EntityZeroRef);

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

        let Ok(dimension_hash) = dimension_hash.get(build_order.dimension_ref.0) else {
            error!(target: "dungeoning", "Dimension entity {:?} has no HashId component when making AdvancedDungeon, skipping structure spawn", build_order.dimension_ref);
            continue;
        };

        let seed = chunk_positions[0].hash_value(&settings, 1).wrapping_add(dimension_hash.as_u64());
        let mut rng = rand_pcg::Pcg64Mcg::seed_from_u64(seed);

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

        // Carve rooms with varied sizes
        for rm in &rooms {
            for yy in 0..rm.h {
                for xx in 0..rm.w {
                    let tx = (rm.x + xx) as usize;
                    let ty = (rm.y + yy) as usize;
                    if tx < tile_width && ty < tile_height {
                        floor_map[ty * tile_width + tx] = true;
                    }
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
                let (sx,ex) = if x0 <= x1 {(x0,x1)} else {(x1,x0)};
                
                for x in sx..=ex {
                    for dy in -corridor_radius..=corridor_radius {
                        let yy = y0 + dy;
                        if x >= 0 && (x as usize) < tile_width && yy >= 0 && (yy as usize) < tile_height {
                            let idx = (yy as usize) * tile_width + (x as usize);
                            floor_map[idx] = true;
                            corridor_map[idx] = true;
                        }
                    }
                }
                
                let (sy,ey) = if y0 <= y1 {(y0,y1)} else {(y1,y0)};
                for y in sy..=ey {
                    for dx in -corridor_radius..=corridor_radius {
                        let xx = x1 + dx;
                        if xx >= 0 && (xx as usize) < tile_width && y >= 0 && (y as usize) < tile_height {
                            let idx = (y as usize) * tile_width + (xx as usize);
                            floor_map[idx] = true;
                            corridor_map[idx] = true;
                        }
                    }
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
                
                let ezero_ref = if hazard_map[map_idx] {
                    if let Some(lava) = lava_entity { lava } else { wall_entity }
                } else if floor_map[map_idx] {
                    floor_entity
                } else {
                    wall_entity
                };
                tiles4chunk.push((tile_pos, ezero_ref, Some(delete_template.clone())));
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