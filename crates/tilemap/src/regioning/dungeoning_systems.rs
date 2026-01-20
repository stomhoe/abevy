
#[allow(unused_imports)] use bevy::prelude::*;

use common::common_components::HashId;
use game_common::game_common_components::EntityZeroRef;
use rand::{Rng, SeedableRng};
use ::tilemap_shared::*;

use crate::{regioning::{regioning_components::*, regioning_messages::{ClaimedChunks, OfferChunk, StructureBuildCompliance, StructureBuildOrder}, regioning_structured_gen_cfg_components::StructuredGenConfig}, tile::{tile_components::DeleteOtherTiles, tile_resources::TileEzerosMap}, };


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
    structured_gens: Query<(&StructuredGenConfig,)>,
) {
    let mut claims_to_emit = Vec::new();
    for claim_request in offered_chunks.read() {
        let Ok((structured_gen_cfg,)) = structured_gens.get(claim_request.structured_gen_cfg_ent)
        else { continue; };

        let id = structured_gen_cfg.hash;
        
        if !ADMITTED_STRUCTURE_IDS.contains(&id) {
            trace!(target: "structure_spawn", "StructuredGenConfig entity {:?} is not in admitted structures, skipping", claim_request.structured_gen_cfg_ent);
            continue;
        }

        let center_chunk = claim_request.start_gpos;
        let parity_seed = (center_chunk.x() as i64).abs() + (center_chunk.y() as i64).abs();
        let side_length = 3 + (parity_seed % 2) as i32;
        let half_spread = side_length / 2;
        let start_offset = -half_spread;
        let end_offset = start_offset + side_length - 1;
        let region_pos = center_chunk.to_region_pos();

        let mut chunk_positions = Vec::new();
        for dy in start_offset..=end_offset {
            for dx in start_offset..=end_offset {
                let candidate = center_chunk + IVec2::new(dx, dy);
                if region_pos.contains_chunkpos(candidate) {
                    chunk_positions.push(candidate);
                }
            }
        }

        if chunk_positions.is_empty() {
            warn!(target: "structure_spawn", "No eligible chunks around {:?} for ExampleStructure, skipping", center_chunk);
            continue;
        }
        chunk_positions.sort_unstable_by_key(|chunk| (chunk.y(), chunk.x()));
        let chunk_count = chunk_positions.len();
        claims_to_emit.push(ClaimedChunks {
            i: claim_request.i,
            region_ent: claim_request.region_ent,
            sgc_ent: claim_request.structured_gen_cfg_ent,
            chunks_gpos: chunk_positions,
            partition_tolerant: false,
        });
        trace!(target: "structure_spawn", "Emitting ClaimedChunks for ExampleStructure covering {} chunks around {:?}", chunk_count, center_chunk);
    }
    writer.write_batch(claims_to_emit);
}


#[allow(unused_parens)]
pub fn drunkwalk_dungeon_building_system(
    mut reader: MessageReader<StructureBuildOrder>,
    structured_gens: Query<(&StructuredGenConfig,),()>,
    mut writer: MessageWriter<StructureBuildCompliance>,
    ezeros_map: Res<TileEzerosMap>,
    settings: Single<&GlobalGenSettings>,
) {
    
    let mut compliances_to_emit = Vec::new();    
    for build_order in reader.read() {
        let Ok((structured_gen_cfg,)) = structured_gens.get(build_order.structured_gen_cfg_ent) 
        else { continue; };

        let id = &structured_gen_cfg.hash;
        
        if *id != DRUNKWALK{
            continue;
        }

        const FLOOR_TILE_ID: HashId = HashId::hash("cyan");
        let floor_entity = match ezeros_map.0.get_with_hash(&FLOOR_TILE_ID) {
            Ok(entity) => EntityZeroRef(entity),
            Err(_) => {
                error!(target: "structure_spawn", "TileEzero with id '{}' not found in TileEzerosMap when making DrunkwalkDungeon, skipping structure spawn", FLOOR_TILE_ID);
                continue;
            }
        };

        const WALL_TILE_ID: HashId = HashId::hash("purple");
        let wall_entity = match ezeros_map.0.get_with_hash(&WALL_TILE_ID) {
            Ok(entity) => EntityZeroRef(entity),
            Err(_) => {
                error!(target: "structure_spawn", "TileEzero with id '{}' not found in TileEzerosMap when making DrunkwalkDungeon, skipping structure spawn", WALL_TILE_ID);
                continue;
            }
        };
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
        let seed = chunk_positions[0].hash_value(&settings, 0);

        let mut rng = rand_pcg::Pcg64Mcg::seed_from_u64(seed);

        let mut walker_x = tile_width / 2;
        let mut walker_y = tile_height / 2;
        let target_floor_tiles = std::cmp::max(1, ((tile_map_size as f32) * 0.35).ceil() as usize);
        let tile_width_minus_one = tile_width - 1;
        let tile_height_minus_one = tile_height - 1;
        let mut carved = 0;
        while carved < target_floor_tiles {
            let idx = walker_y * tile_width + walker_x;
            if !floor_map[idx] {
                floor_map[idx] = true;
                carved += 1;
            }
            match rng.random_range(0..4) {
                0 => if walker_x < tile_width_minus_one { walker_x += 1; }
                1 => if walker_x > 0 { walker_x -= 1; }
                2 => if walker_y < tile_height_minus_one { walker_y += 1; }
                _ => if walker_y > 0 { walker_y -= 1; }
            }
        }

        let room_attempts = std::cmp::max(1, tile_map_size / 250);
        for _ in 0..room_attempts {
            let center_x = rng.random_range(0..tile_width);
            let center_y = rng.random_range(0..tile_height);
            let radius = rng.random_range(1..=3) as i32;
            for dy in -radius..=radius {
                for dx in -radius..=radius {
                    let rx = center_x as i32 + dx;
                    let ry = center_y as i32 + dy;
                    if rx < 0 || ry < 0 {
                        continue;
                    }
                    let rx = rx as usize;
                    let ry = ry as usize;
                    if rx >= tile_width || ry >= tile_height {
                        continue;
                    }
                    floor_map[ry * tile_width + rx] = true;
                }
            }
        }

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
                let ezero_ref = if floor_map[map_idx] {
                    floor_entity
                } else {
                    wall_entity
                };
                tiles4chunk.push((tile_pos, ezero_ref, Some(delete_template.clone())));
            }
            compliances_to_emit.push(StructureBuildCompliance {
                structure_gen_cfg_ent: build_order.structured_gen_cfg_ent,
                dimension_ref: build_order.dimension_ref,
                chunk_pos,
                tiles: tiles4chunk,
            });
        }
        debug!(target: "structure_spawn", "Spawned dungeon for ExampleStructure across {} chunks at {:?}", chunk_positions.len(), build_order.region_pos);
    }
    writer.write_batch(compliances_to_emit);
}

#[allow(unused_parens)]
pub fn advanced_dungeon_building_system(
    mut reader: MessageReader<StructureBuildOrder>,
    structured_gens: Query<(&StructuredGenConfig,),()>,
    mut writer: MessageWriter<StructureBuildCompliance>,
    ezeros_map: Res<TileEzerosMap>,
    settings: Single<&GlobalGenSettings>,
) {
    // BSP rooms-and-corridors generator (distinct from drunkwalk)
    let mut compliances_to_emit = Vec::new();
    for build_order in reader.read() {
        let Ok((structured_gen_cfg,)) = structured_gens.get(build_order.structured_gen_cfg_ent) else { continue; };

        // Only run for the admitted "advanced" structure id
        if structured_gen_cfg.hash != IDK {
            continue;
        }

        const FLOOR_TILE_ID: HashId = HashId::hash("cyan");
        let floor_entity = match ezeros_map.0.get_with_hash(&FLOOR_TILE_ID) {
            Ok(entity) => EntityZeroRef(entity),
            Err(_) => {
                error!(target: "structure_spawn", "TileEzero with id '{}' not found in TileEzerosMap when making AdvancedDungeon, skipping structure spawn", FLOOR_TILE_ID);
                continue;
            }
        };

        const WALL_TILE_ID: HashId = HashId::hash("purple");
        let wall_entity = match ezeros_map.0.get_with_hash(&WALL_TILE_ID) {
            Ok(entity) => EntityZeroRef(entity),
            Err(_) => {
                error!(target: "structure_spawn", "TileEzero with id '{}' not found in TileEzerosMap when making AdvancedDungeon, skipping structure spawn", WALL_TILE_ID);
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

        // Seed RNG differently from drunkwalk (salt=1) so results differ
        let seed = chunk_positions[0].hash_value(&settings, 1);
        let mut rng = rand_pcg::Pcg64Mcg::seed_from_u64(seed);

        // BSP split representation
        #[derive(Clone, Copy)]
        struct Rect { x: i32, y: i32, w: i32, h: i32 }
        impl Rect {
            fn area(&self) -> i32 { self.w * self.h }
            fn center(&self) -> (i32,i32) { (self.x + self.w/2, self.y + self.h/2) }
        }

        // Start with whole map rectangle
        let mut leafs: Vec<Rect> = vec![ Rect { x: 0, y: 0, w: tile_width as i32, h: tile_height as i32 } ];

        let target_rooms = std::cmp::max(1, (tile_map_size / 300).min(12)); // heuristic
        let min_room_size = 4;
        let max_splits = 200;

        // Splitting loop
        for _ in 0..max_splits {
            if leafs.len() >= target_rooms { break; }
            // pick largest leaf to split more often
            let (idx, rect) = leafs.iter().enumerate().max_by_key(|(_,r)| r.area()).map(|(i,r)| (i,*r)).unwrap();
            let can_split_h = rect.h >= (min_room_size*2 + 2);
            let can_split_w = rect.w >= (min_room_size*2 + 2);
            if !can_split_h && !can_split_w { break; }

            // Split along longer axis
            if can_split_w && (!can_split_h || rect.w >= rect.h) {
                // choose split so both sides >= min_room_size
                let split_min = rect.x + min_room_size;
                let split_max = rect.x + rect.w - min_room_size;
                if split_max - split_min <= 0 { break; }
                let split = rng.random_range(split_min..split_max);
                let left = Rect { x: rect.x, y: rect.y, w: split - rect.x, h: rect.h };
                let right = Rect { x: split, y: rect.y, w: rect.x + rect.w - split, h: rect.h };
                leafs.remove(idx);
                leafs.push(left); leafs.push(right);
            } else if can_split_h {
                let split_min = rect.y + min_room_size;
                let split_max = rect.y + rect.h - min_room_size;
                if split_max - split_min <= 0 { break; }
                let split = rng.random_range(split_min..split_max);
                let top = Rect { x: rect.x, y: rect.y, w: rect.w, h: split - rect.y };
                let bot = Rect { x: rect.x, y: split, w: rect.w, h: rect.y + rect.h - split };
                leafs.remove(idx);
                leafs.push(top); leafs.push(bot);
            } else { break; }
        }

        // From leaf rects, choose room rectangles inset with random padding
        #[derive(Clone, Copy)] struct Room { x: i32, y: i32, w: i32, h: i32 }
        let mut rooms: Vec<Room> = Vec::new();
        for r in &leafs {
            // available room space inside the leaf with a 1-tile margin on each side
            let available_w = r.w - 2;
            let available_h = r.h - 2;
            if available_w < min_room_size as i32 || available_h < min_room_size as i32 { continue; }
            let room_w = rng.random_range(min_room_size..=available_w) as i32;
            let room_h = rng.random_range(min_room_size..=available_h) as i32;
            // compute placement bounds and guard against pathological cases
            let px_min = r.x + 1;
            let px_max = r.x + r.w - room_w - 1;
            let py_min = r.y + 1;
            let py_max = r.y + r.h - room_h - 1;
            if px_max < px_min || py_max < py_min { continue; }
            let px = rng.random_range(px_min ..= px_max);
            let py = rng.random_range(py_min ..= py_max);
            rooms.push(Room { x: px, y: py, w: room_w, h: room_h });
        }

        // Carve rooms into floor_map
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

        // Corridor map: mark corridor tiles separately so we can avoid randomly placing pillars there
        let mut corridor_map = vec![false; tile_map_size];

        // Connect rooms using simple nearest-neighbor chain (ensures connectivity)
        // Make corridors wider (radius = 1 -> 3 tiles wide) and deterministic (no random carving within corridor)
        let corridor_radius: i32 = 1; // change this to make corridors wider (>=1)
        if !rooms.is_empty() {
            // compute centers
            let mut centers: Vec<(i32,i32)> = rooms.iter().map(|r| (r.x + r.w/2, r.y + r.h/2)).collect();
            // sort by x to make corridor layout pleasant
            centers.sort_by_key(|c| c.0);
            for i in 1..centers.len() {
                let (x0,y0) = centers[i-1];
                let (x1,y1) = centers[i];
                // L-shaped corridor: horizontal then vertical
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

        // Optional: scatter a few pillars inside rooms for variety
        // Avoid placing pillars in corridor tiles to keep corridors intact
        let pillar_attempts = (rooms.len() * 3).max(0);
        for _ in 0..pillar_attempts {
            if rooms.is_empty() { break; }
            let r = rooms[rng.random_range(0..rooms.len())];
            let px = rng.random_range(r.x..(r.x + r.w)) as usize;
            let py = rng.random_range(r.y..(r.y + r.h)) as usize;
            if px < tile_width && py < tile_height {
                let idx = py * tile_width + px;
                // don't place pillars in corridors
                if corridor_map[idx] { continue; }
                // place a small 1x1 pillar (make it wall)
                floor_map[idx] = false;
            }
        }

        // Build per-chunk tile lists similar to drunkwalk
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
                let ezero_ref = if floor_map[map_idx] { floor_entity } else { wall_entity };
                tiles4chunk.push((tile_pos, ezero_ref, Some(delete_template.clone())));
            }
            compliances_to_emit.push(StructureBuildCompliance {
                structure_gen_cfg_ent: build_order.structured_gen_cfg_ent,
                dimension_ref: build_order.dimension_ref,
                chunk_pos,
                tiles: tiles4chunk,
            });
        }

        debug!(target: "structure_spawn", "Spawned advanced BSP dungeon across {} chunks at {:?}", chunk_positions.len(), build_order.region_pos);
    }
    writer.write_batch(compliances_to_emit);
}