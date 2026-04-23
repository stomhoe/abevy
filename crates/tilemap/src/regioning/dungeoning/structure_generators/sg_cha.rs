#[allow(unused_imports)] use bevy::{platform::collections::*, prelude::*};
use rand_pcg::Pcg64Mcg;

use common::common_components::HashId;
#[allow(unused_imports)] use common::log_targets::DUNGEONING_SYSTEM;
use rand::{Rng, SeedableRng};
use ::tilemap_shared::*;

use crate::regioning::{    regioning_components::*,
    regioning_messages::{StructureBuildCompliance, SgcPrepareTilesOrder, TerrGenDisabledGposForChunks},
    regioning_sgc_components::StructuredGenConfig,
};
use crate::terrain::terrgen_async_resources::TerrGenBlockedGposMask;
use crate::tile::tile_resources::*;
use super::super::dungeoning_carve_helpers::{
    carve_corridor_horizontal_typed as carve_corridor_horizontal,
    carve_corridor_vertical_typed as carve_corridor_vertical,
    carve_room_ellipse_typed as carve_room_ellipse,
    carve_room_pentacle,
    carve_room_rectangle_typed as carve_room_rectangle,
    carve_room_trapezoid_typed as carve_room_trapezoid,
    carve_room_regular_polygon_typed as carve_room_regular_polygon,
};
use super::super::dungeoning_ids::CHAMBERS_CORRIDORS;
use super::super::dungeoning_utils::{carve_external_wall_doorways, extend_occupied_gpos, queue_room_spawn_instance_message, ExternalDoorwayConfig, DeleteOtherTilesConfigMap, TerrGenDisableConfigMap};
use super::sg_cha_types::*;

const FLOOR_NONE: u8 = 0;
const FLOOR_MAIN: u8 = 1;
const FLOOR_B: u8 = 2;

fn seal_structure_border_band_typed(
    floor_map: &mut [u8],
    hazard_map: &mut [bool],
    map_width: usize,
    map_height: usize,
    border_band: usize,
) {
    if border_band == 0 || map_width == 0 || map_height == 0 {
        return;
    }
    if border_band * 2 >= map_width || border_band * 2 >= map_height {
        floor_map.fill(FLOOR_NONE);
        hazard_map.fill(false);
        return;
    }

    let end_x = map_width - border_band;
    let end_y = map_height - border_band;
    for y in 0..map_height {
        for x in 0..map_width {
            if x < border_band || x >= end_x || y < border_band || y >= end_y {
                let idx = y * map_width + x;
                floor_map[idx] = FLOOR_NONE;
                hazard_map[idx] = false;
            }
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct RoomDraft {
    x: i32,
    y: i32,
    w: i32,
    h: i32,
}

impl RoomDraft {
    fn area(self) -> i32 {
        self.w * self.h
    }

    fn as_room(self) -> Room {
        Room { x: self.x, y: self.y, w: self.w, h: self.h, shape: RoomShape::Rectangle }
    }
}

fn pick_weighted_room_spec(candidates: &[(RoomSpec, f32)], rng: &mut impl Rng) -> Option<RoomSpec> {
    let total_weight: f32 = candidates.iter().map(|candidate| candidate.1).sum();
    if total_weight <= 0.0 {
        return None;
    }

    let mut roll = rng.random_range(0.0..total_weight);
    for (spec, score) in candidates.iter().copied() {
        if roll <= score {
            return Some(spec);
        }
        roll -= score;
    }

    candidates.last().copied().map(|(spec, _)| spec)
}

fn sample_room_spec_for_leaf_with_limit(
    room: &RoomDraft,
    args: &game_common::game_common_components::ArgsDict,
    size_configs: &HashMap<RoomShape, RoomSizeConfig>,
    global_size_config: &RoomSizeConfig,
    max_area: Option<i32>,
    allow_oversize_fallback: bool,
    rng: &mut impl Rng,
) -> Option<RoomSpec> {
    let mut bounded_candidates: Vec<(RoomSpec, f32)> = Vec::with_capacity(5);
    let mut all_candidates: Vec<(RoomSpec, f32)> = Vec::with_capacity(5);
    let room_as_room = room.as_room();

    for shape in [RoomShape::Rectangle, RoomShape::Ellipse, RoomShape::Trapezoid, RoomShape::RegularPolygon, RoomShape::Pentacle] {
        let legacy_weight_key = match shape {
            RoomShape::Rectangle => "room_shape_weight_rectangle",
            RoomShape::Ellipse => "room_shape_weight_circle",
            RoomShape::Trapezoid => "room_shape_weight_triangle",
            RoomShape::RegularPolygon => "room_shape_weight_polygon",
            RoomShape::Pentacle => "room_shape_weight_pentacle",
        };
        let weight = args
            .get(&format!("{}_weight", shape.as_str()))
            .or_else(|| args.get(legacy_weight_key))
            .and_then(|v| v.first())
            .and_then(|s| s.parse::<f32>().ok())
            .unwrap_or(if let RoomShape::Pentacle = shape { 0.05 } else { 1. });

        if weight <= 0.0 {
            continue;
        }

        let size_cfg = size_configs.get(&shape).unwrap_or(global_size_config);
        if !shape.can_fit(&room_as_room, size_cfg) {
            continue;
        }

        let Some((rw, rh)) = shape.sample_dimensions(&room_as_room, size_cfg, rng) else { continue; };
        let spec = RoomSpec { shape, w: rw, h: rh };
        let score = weight.max(0.0) * spec.area().max(1) as f32;
        if score > 0.0 {
            all_candidates.push((spec, score));
            if max_area.map_or(true, |limit| spec.area() <= limit) {
                bounded_candidates.push((spec, score));
            }
        }
    }

    if !bounded_candidates.is_empty() {
        return pick_weighted_room_spec(&bounded_candidates, rng);
    }

    if !allow_oversize_fallback {
        return None;
    }

    let min_area = all_candidates.iter().map(|(spec, _)| spec.area()).min()?;
    let smallest_candidates: Vec<(RoomSpec, f32)> = all_candidates
        .into_iter()
        .filter(|(spec, _)| spec.area() == min_area)
        .collect();
    pick_weighted_room_spec(&smallest_candidates, rng)
}

fn sample_trapezoid_vertices_for_room(room: &Room, rng: &mut impl Rng) -> ((i32, i32), (i32, i32), (i32, i32), (i32, i32)) {
    let left = room.x;
    let right = room.x + room.w - 1;
    let top = room.y;
    let bottom = room.y + room.h - 1;
    let min_shrink_w = ((room.w - 2) / 2).max(1);
    let min_shrink_h = ((room.h - 2) / 2).max(1);

    match rng.random_range(0..4) {
        0 => {
            let shrink = rng.random_range(1..=min_shrink_w);
            let left_pad = rng.random_range(0..=shrink);
            let right_pad = shrink - left_pad;
            ((left + left_pad, top), (right - right_pad, top), (right, bottom), (left, bottom))
        }
        1 => {
            let shrink = rng.random_range(1..=min_shrink_w);
            let left_pad = rng.random_range(0..=shrink);
            let right_pad = shrink - left_pad;
            ((left, top), (right, top), (right - right_pad, bottom), (left + left_pad, bottom))
        }
        2 => {
            let shrink = rng.random_range(1..=min_shrink_h);
            let top_pad = rng.random_range(0..=shrink);
            let bottom_pad = shrink - top_pad;
            ((left, top + top_pad), (right, top), (right, bottom), (left, bottom - bottom_pad))
        }
        _ => {
            let shrink = rng.random_range(1..=min_shrink_h);
            let top_pad = rng.random_range(0..=shrink);
            let bottom_pad = shrink - top_pad;
            ((left, top), (right, top + top_pad), (right, bottom - bottom_pad), (left, bottom))
        }
    }
}

fn split_room_draft_horizontally(room: RoomDraft, split_at: i32, keep_left: bool) -> (RoomDraft, Option<RoomDraft>) {
    let right_width = room.w - split_at;
    if keep_left {
        (
            RoomDraft { x: room.x, y: room.y, w: split_at, h: room.h },
            Some(RoomDraft { x: room.x + split_at, y: room.y, w: right_width, h: room.h }),
        )
    } else {
        (
            RoomDraft { x: room.x + right_width, y: room.y, w: split_at, h: room.h },
            Some(RoomDraft { x: room.x, y: room.y, w: right_width, h: room.h }),
        )
    }
}

fn split_room_draft_vertically(room: RoomDraft, split_at: i32, keep_top: bool) -> (RoomDraft, Option<RoomDraft>) {
    let bottom_height = room.h - split_at;
    if keep_top {
        (
            RoomDraft { x: room.x, y: room.y, w: room.w, h: split_at },
            Some(RoomDraft { x: room.x, y: room.y + split_at, w: room.w, h: bottom_height }),
        )
    } else {
        (
            RoomDraft { x: room.x, y: room.y + bottom_height, w: room.w, h: split_at },
            Some(RoomDraft { x: room.x, y: room.y, w: room.w, h: bottom_height }),
        )
    }
}

fn room_center(room: &Room) -> (i32, i32) {
    (room.x + room.w / 2, room.y + room.h / 2)
}

fn sample_corridor_child_slots(rng: &mut impl Rng) -> usize {
    match rng.random_range(0..100) {
        0..=14 => 0,
        15..=84 => 1,
        85..=95 => 2,
        _ => 3,
    }
}

fn pick_weighted_open_parent_index(open_parents: &[(usize, usize)], rng: &mut impl Rng) -> Option<usize> {
    let total_slots = open_parents.iter().map(|(_, slots)| *slots).sum::<usize>();
    if total_slots == 0 {
        return None;
    }

    let mut roll = rng.random_range(0..total_slots);
    for (idx, (_, slots)) in open_parents.iter().enumerate() {
        if roll < *slots {
            return Some(idx);
        }
        roll -= *slots;
    }

    None
}

fn pick_nearest_room_index(parent_idx: usize, candidate_indices: &[usize], rooms: &[Room], rng: &mut impl Rng) -> Option<usize> {
    let parent_center = room_center(&rooms[parent_idx]);
    let mut best_indices = Vec::new();
    let mut best_distance = i32::MAX;

    for &candidate_idx in candidate_indices {
        let candidate_center = room_center(&rooms[candidate_idx]);
        let distance = (parent_center.0 - candidate_center.0).abs() + (parent_center.1 - candidate_center.1).abs();
        if distance < best_distance {
            best_distance = distance;
            best_indices.clear();
            best_indices.push(candidate_idx);
        } else if distance == best_distance {
            best_indices.push(candidate_idx);
        }
    }

    if best_indices.is_empty() {
        return None;
    }

    best_indices.get(rng.random_range(0..best_indices.len())).copied()
}

inventory::submit! {
    crate::regioning::dungeoning::dungeoning_ids::StructureGeneratorDescriptor {
        structure_hash_id: CHAMBERS_CORRIDORS,
    }
}

#[allow(unused_parens, )]
pub fn corridor_dungeon_building_system(
    mut reader: MessageReader<SgcPrepareTilesOrder>,
    structured_gens: Query<(&StructuredGenConfig,),()>,
    mut writer: MessageWriter<StructureBuildCompliance>,
    templs_map: Res<TileEntityMap>,
    mut room_pack_spawn: super::super::dungeoning_utils::DungeonRoomPackSpawnSystemParams,
    templ_size_query: Query<&SizeInTiles, (With<game_common::game_common_components::Templ>, common::AnyDisabling)>,
    settings: Query<&GlobalGenSettings>,
    mut compliances_to_emit: Local<Vec<StructureBuildCompliance>>,
    mut room_drafts: Local<Vec<RoomDraft>>,
    _tiles_buffer: Local<Vec<(GlobalTilePos, TileRef, Option<DeleteOtherTilesInSamePos>)>>,
) {
    let Ok(settings) = settings.single() else {
        error_once!("Failed to get global gen settings");
        return;
    };
    compliances_to_emit.clear();
    room_pack_spawn.begin_pass();
    room_drafts.clear();
    
    for build_order in reader.read() {

        let Ok((structured_gen_cfg,)) = structured_gens.get(build_order.structured_gen_cfg_ent) else { continue; };

        if structured_gen_cfg.structure_hash_id() != CHAMBERS_CORRIDORS {
            continue;
        }
        let room_spawn_config = super::super::dungeoning_utils::DungeonRoomPackSpawnConfig::from_typed_args(
            &structured_gen_cfg.typed_args,
            &room_pack_spawn.command_registry,
            structured_gen_cfg.structure_id().as_str(),
        );
        let mut beings_remaining = structured_gen_cfg.max_being_count.unwrap_or(u32::MAX);
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

        let delete_other_tiles_by_tile_id = DeleteOtherTilesConfigMap::from_args(&structured_gen_cfg.args);
        let terrgen_disable_by_tile_id = TerrGenDisableConfigMap::from_args(&structured_gen_cfg.args);

        let Ok(floor_entity_ent) = templs_map.0.get_cloned(floor_tile_id) else {
            error!(target: DUNGEONING_SYSTEM, "TileTempl with id '{:?}' not found in TileEntityMap when making ChambersCorridorsDungeon, skipping structure spawn", floor_tile_id);
            continue;
        };
        let Ok(_wall_entity_ent) = templs_map.0.get_cloned(wall_tile_id) else {
            error!(target: DUNGEONING_SYSTEM, "TileTempl with id '{:?}' not found in TileEntityMap when making ChambersCorridorsDungeon, skipping structure spawn", wall_tile_id);
            continue;
        };

        let floor_entity = TileRef(floor_tile_id);
        let wall_entity = TileRef(wall_tile_id);

        let chunk_positions = &build_order.chunks_pos;
        if chunk_positions.is_empty() {
            continue;
        }

        let min_chunk_x = chunk_positions.iter().map(|chunk| chunk.x()).min().unwrap();
        let max_chunk_x = chunk_positions.iter().map(|chunk| chunk.x()).max().unwrap();
        let min_chunk_y = chunk_positions.iter().map(|chunk| chunk.y()).min().unwrap();
        let max_chunk_y = chunk_positions.iter().map(|chunk| chunk.y()).max().unwrap();

        let chunk_width = (max_chunk_x - min_chunk_x + 1) as usize;
        let chunk_height = (max_chunk_y - min_chunk_y + 1) as usize;
        let map_width = chunk_width * ChunkPos::CHUNK_SIZE.x as usize;
        let map_height = chunk_height * ChunkPos::CHUNK_SIZE.y as usize;
        if map_width == 0 || map_height == 0 {
            continue;
        }

        let origin_chunk = ChunkPos::new(min_chunk_x, min_chunk_y);
        let origin_tile = origin_chunk.to_tilepos();

        let mut max_rooms = 30;
        let mut border_band = 1;
        let mut corridor_detour_chance = 0.0_f32;
        let mut corridor_detour_max_offset = 0_i32;
        let mut corridor_radius = 1_i32;
        let mut room_density_min: f32 = 0.1;
        let mut room_density_max: f32 = 0.8;
        if let Some(v) = structured_gen_cfg.args.get("max_rooms") {
            if let Some(s) = v.first() {
                max_rooms = s.parse::<u32>().unwrap_or(max_rooms);
            }
        }
        if let Some(v) = structured_gen_cfg.args.get("border_band") {
            if let Some(s) = v.first() {
                border_band = s.parse::<usize>().unwrap_or(1);
            }
        }
        if let Some(v) = structured_gen_cfg.args.get("corridor_detour_chance") {
            if let Some(s) = v.first() {
                corridor_detour_chance = s.parse::<f32>().unwrap_or(0.0).clamp(0.0, 1.0);
            }
        }
        if let Some(v) = structured_gen_cfg.args.get("corridor_detour_max_offset") {
            if let Some(s) = v.first() {
                corridor_detour_max_offset = s.parse::<i32>().unwrap_or(0).max(0);
            }
        }
        if let Some(v) = structured_gen_cfg.args.get("corridor_radius") {
            if let Some(s) = v.first() {
                corridor_radius = s.parse::<i32>().unwrap_or(1).max(1);
            }
        }
        if let Some(v) = structured_gen_cfg.args.get("room_density.min") {
            if let Some(s) = v.first() {
                room_density_min = s.parse::<f32>().unwrap_or(room_density_min).clamp(0.0, 1.0);
            }
        }
        if let Some(v) = structured_gen_cfg.args.get("room_density.max") {
            if let Some(s) = v.first() {
                room_density_max = s.parse::<f32>().unwrap_or(room_density_max).clamp(0.0, 1.0);
            }
        }
        if room_density_max < room_density_min {
            room_density_max = room_density_min;
        }

        let mut floor_map = vec![FLOOR_NONE; map_width * map_height];
        let mut hazard_map = vec![false; map_width * map_height];

        let mut size_configs: HashMap<RoomShape, RoomSizeConfig> = HashMap::default();
        let global_size_config = RoomSizeConfig::default_global();
        let mut min_leaf_w = global_size_config.width.min;
        let mut min_leaf_h = global_size_config.height.min;
        if let Some(v) = structured_gen_cfg.args.get("min_leaf_w") {
            if let Some(s) = v.first() {
                min_leaf_w = s.parse::<i32>().unwrap_or(min_leaf_w).max(7);
            }
        }
        if let Some(v) = structured_gen_cfg.args.get("min_leaf_h") {
            if let Some(s) = v.first() {
                min_leaf_h = s.parse::<i32>().unwrap_or(min_leaf_h).max(7);
            }
        }

        for shape in [RoomShape::Rectangle, RoomShape::Ellipse, RoomShape::Trapezoid, RoomShape::RegularPolygon, RoomShape::Pentacle] {
            let shape_str = shape.as_str();
            let mut cfg = global_size_config;
            let width_min_key = format!("room_size.{}.width.min", shape_str);
            let width_max_key = format!("room_size.{}.width.max", shape_str);
            let width_min_alias_key = format!("room_size.{}.min_w", shape_str);
            let width_max_alias_key = format!("room_size.{}.max_w", shape_str);
            let height_min_key = format!("room_size.{}.height.min", shape_str);
            let height_max_key = format!("room_size.{}.height.max", shape_str);
            let height_min_alias_key = format!("room_size.{}.min_h", shape_str);
            let height_max_alias_key = format!("room_size.{}.max_h", shape_str);

            if let Some(v) = structured_gen_cfg.args.get(&width_min_key).or_else(|| structured_gen_cfg.args.get(&width_min_alias_key)) {
                if let Some(s) = v.first() { cfg.width.min = s.parse().unwrap_or(cfg.width.min); }
            }
            if let Some(v) = structured_gen_cfg.args.get(&width_max_key).or_else(|| structured_gen_cfg.args.get(&width_max_alias_key)) {
                if let Some(s) = v.first() { cfg.width.max = s.parse().ok(); }
            }
            if let Some(v) = structured_gen_cfg.args.get(&height_min_key).or_else(|| structured_gen_cfg.args.get(&height_min_alias_key)) {
                if let Some(s) = v.first() { cfg.height.min = s.parse().unwrap_or(cfg.height.min); }
            }
            if let Some(v) = structured_gen_cfg.args.get(&height_max_key).or_else(|| structured_gen_cfg.args.get(&height_max_alias_key)) {
                if let Some(s) = v.first() { cfg.height.max = s.parse().ok(); }
            }
            if let RoomShape::Pentacle = shape {
                cfg.width.min = cfg.width.min.max(15);
                cfg.height.min = cfg.height.min.max(15);
            }
            size_configs.insert(shape, cfg);
        }

        let seed = origin_chunk.hash_value(&settings, build_order.dimension_ref.0, build_order.i);
        let mut rng = Pcg64Mcg::seed_from_u64(seed);
        let claimed_area = map_width.saturating_mul(map_height);
        let room_density = rng.random_range(room_density_min..=room_density_max);
        let room_area_budget = ((claimed_area as f32) * room_density).round().max(1.0) as i32;
        debug!(target: DUNGEONING_SYSTEM, "structure={} room_density={:.3} room_area_budget={} claimed_area={} density_range=[{:.3}, {:.3}]", structured_gen_cfg.structure_id(), room_density, room_area_budget, claimed_area, room_density_min, room_density_max);
        room_drafts.clear();
        room_drafts.push(RoomDraft { x: 0, y: 0, w: map_width as i32, h: map_height as i32 });

        let mut actual_rooms = Vec::new();
        let mut used_room_area = 0_i32;
        while actual_rooms.len() < max_rooms as usize && !room_drafts.is_empty() && used_room_area < room_area_budget {
            let remaining_area_budget = room_area_budget.saturating_sub(used_room_area);
            let total_area_weight: usize = room_drafts.iter().map(|room| room.area().max(1) as usize).sum();
            let mut room_pick = rng.random_range(0..total_area_weight);
            let mut room_idx = 0;
            for (idx, room) in room_drafts.iter().enumerate() {
                let room_weight = room.area().max(1) as usize;
                if room_pick < room_weight {
                    room_idx = idx;
                    break;
                }
                room_pick -= room_weight;
            }
            let mut room = room_drafts.swap_remove(room_idx);
            let Some(plan) = sample_room_spec_for_leaf_with_limit(&room, &structured_gen_cfg.args, &size_configs, &global_size_config, Some(remaining_area_budget), actual_rooms.is_empty(), &mut rng) else { break; };
            let plan_area = plan.area().max(1);

            loop {
                if room.w == plan.w && room.h == plan.h {
                    actual_rooms.push(plan.into_room(room.x, room.y));
                    used_room_area = used_room_area.saturating_add(plan_area);
                    break;
                }

                if room.w < plan.w || room.h < plan.h {
                    break;
                }

                let surplus_w = room.w - plan.w;
                let surplus_h = room.h - plan.h;
                let split_horizontally = if surplus_h == 0 {
                    true
                } else if surplus_w == 0 {
                    false
                } else {
                    surplus_w >= surplus_h
                };

                if split_horizontally {
                    let keep_left = rng.random_bool(0.5);
                    let (next_room, remainder) = split_room_draft_horizontally(room, plan.w, keep_left);
                    room = next_room;
                    if let Some(remainder) = remainder {
                        if remainder.w >= min_leaf_w && remainder.h >= min_leaf_h {
                            room_drafts.push(remainder);
                        }
                    }
                } else {
                    let keep_top = rng.random_bool(0.5);
                    let (next_room, remainder) = split_room_draft_vertically(room, plan.h, keep_top);
                    room = next_room;
                    if let Some(remainder) = remainder {
                        if remainder.w >= min_leaf_w && remainder.h >= min_leaf_h {
                            room_drafts.push(remainder);
                        }
                    }
                }
            }
        }

        for room in &actual_rooms {
            let rx = room.x;
            let ry = room.y;

            match room.shape {
                RoomShape::Rectangle => {
                    carve_room_rectangle(&mut floor_map, map_width, map_height, rx, ry, room.w, room.h, FLOOR_MAIN);
                }
                RoomShape::Ellipse => {
                    carve_room_ellipse(&mut floor_map, map_width, map_height, rx, ry, room.w, room.h, FLOOR_MAIN);
                }
                RoomShape::Trapezoid => {
                    let (v0, v1, v2, v3) = sample_trapezoid_vertices_for_room(room, &mut rng);
                    carve_room_trapezoid(&mut floor_map, map_width, map_height, v0, v1, v2, v3, FLOOR_MAIN);
                }
                RoomShape::RegularPolygon => {
                    let sides = rng.random_range(5..=8);
                    carve_room_regular_polygon(&mut floor_map, map_width, map_height, rx, ry, room.w, room.h, sides, 0.0, FLOOR_MAIN);
                }
                RoomShape::Pentacle => {
                    carve_room_pentacle(&mut floor_map, map_width, map_height, rx, ry, room.w, room.h, FLOOR_MAIN, FLOOR_B);
                }
            }
        }

        let mut corridor_map = vec![false; map_width * map_height];
        if actual_rooms.len() > 1 {
            let mut remaining_rooms: Vec<usize> = (0..actual_rooms.len()).collect();
            let root_idx = remaining_rooms.swap_remove(rng.random_range(0..remaining_rooms.len()));
            let mut open_parents: Vec<(usize, usize)> = Vec::with_capacity(actual_rooms.len());
            let mut connected_rooms: Vec<usize> = Vec::with_capacity(actual_rooms.len());
            connected_rooms.push(root_idx);

            let root_slots = sample_corridor_child_slots(&mut rng).max(1);
            open_parents.push((root_idx, root_slots));

            while !remaining_rooms.is_empty() {
                if open_parents.is_empty() {
                    let fallback_idx = connected_rooms[rng.random_range(0..connected_rooms.len())];
                    open_parents.push((fallback_idx, 1));
                }

                let Some(parent_slot_idx) = pick_weighted_open_parent_index(&open_parents, &mut rng) else { continue; };
                let parent_idx = open_parents[parent_slot_idx].0;
                let Some(child_idx) = pick_nearest_room_index(parent_idx, &remaining_rooms, &actual_rooms, &mut rng) else { continue; };
                let child_pos = remaining_rooms.iter().position(|candidate_idx| *candidate_idx == child_idx).unwrap_or(0);
                let child_idx = remaining_rooms.swap_remove(child_pos);

                let c1 = room_center(&actual_rooms[parent_idx]);
                let c2 = room_center(&actual_rooms[child_idx]);
                let rad = Some(corridor_radius);
                if rng.random_range(0.0..1.0) < corridor_detour_chance && corridor_detour_max_offset > 0 {
                    let min_x = (c1.0.min(c2.0) - corridor_detour_max_offset).clamp(0, map_width.saturating_sub(1) as i32);
                    let max_x = (c1.0.max(c2.0) + corridor_detour_max_offset).clamp(0, map_width.saturating_sub(1) as i32);
                    let min_y = (c1.1.min(c2.1) - corridor_detour_max_offset).clamp(0, map_height.saturating_sub(1) as i32);
                    let max_y = (c1.1.max(c2.1) + corridor_detour_max_offset).clamp(0, map_height.saturating_sub(1) as i32);
                    let detour_x = rng.random_range(min_x..=max_x);
                    let detour_y = rng.random_range(min_y..=max_y);

                    carve_corridor_horizontal(&mut rng, &mut floor_map, FLOOR_MAIN, &mut corridor_map, map_width, map_height, rad, None, None, c1.1, c1.0, detour_x);
                    carve_corridor_vertical(&mut rng, &mut floor_map, FLOOR_MAIN, &mut corridor_map, map_width, map_height, rad, None, None, detour_x, c1.1, detour_y);
                    carve_corridor_horizontal(&mut rng, &mut floor_map, FLOOR_MAIN, &mut corridor_map, map_width, map_height, rad, None, None, detour_y, detour_x, c2.0);
                    carve_corridor_vertical(&mut rng, &mut floor_map, FLOOR_MAIN, &mut corridor_map, map_width, map_height, rad, None, None, c2.0, detour_y, c2.1);
                } else if rng.random_bool(0.5) {
                    carve_corridor_horizontal(&mut rng, &mut floor_map, FLOOR_MAIN, &mut corridor_map, map_width, map_height, rad, None, None, c1.1, c1.0, c2.0);
                    carve_corridor_vertical(&mut rng, &mut floor_map, FLOOR_MAIN, &mut corridor_map, map_width, map_height, rad, None, None, c2.0, c1.1, c2.1);
                } else {
                    carve_corridor_vertical(&mut rng, &mut floor_map, FLOOR_MAIN, &mut corridor_map, map_width, map_height, rad, None, None, c1.0, c1.1, c2.1);
                    carve_corridor_horizontal(&mut rng, &mut floor_map, FLOOR_MAIN, &mut corridor_map, map_width, map_height, rad, None, None, c2.1, c1.0, c2.0);
                }

                let parent_slots = &mut open_parents[parent_slot_idx].1;
                *parent_slots = parent_slots.saturating_sub(1);
                if *parent_slots == 0 {
                    open_parents.swap_remove(parent_slot_idx);
                }

                connected_rooms.push(child_idx);
                let child_slots = if remaining_rooms.is_empty() { 0 } else { sample_corridor_child_slots(&mut rng) };
                if child_slots > 0 {
                    open_parents.push((child_idx, child_slots));
                }
            }
        }

        seal_structure_border_band_typed(&mut floor_map, &mut hazard_map, map_width, map_height, border_band);

        let mut floor_map_bool = floor_map.iter().map(|&floor| floor != FLOOR_NONE).collect::<Vec<_>>();
        let mut wall_map = vec![false; map_width * map_height];
        for y in 0..map_height {
            for x in 0..map_width {
                let idx = y * map_width + x;
                if !floor_map_bool[idx] && !hazard_map[idx] { continue; }
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
                    if ny < map_height && nx < map_width {
                        let nidx = ny * map_width + nx;
                        if !floor_map_bool[nidx] && !hazard_map[nidx] {
                            wall_map[nidx] = true;
                        }
                    }
                }
            }
        }

        let opened_doorways = carve_external_wall_doorways(
            &mut floor_map_bool,
            Some(&mut hazard_map),
            &mut wall_map,
            map_width,
            map_height,
            ExternalDoorwayConfig::from_args(&structured_gen_cfg.args),
            &mut rng,
        );
        trace!(target: DUNGEONING_SYSTEM, "structure={} opened_external_doorways={}", structured_gen_cfg.structure_id(), opened_doorways);

        let floor_delete_other_tiles = delete_other_tiles_by_tile_id.get("floor_tile_id");
        let wall_delete_other_tiles = delete_other_tiles_by_tile_id.get("wall_tile_id");
        let disable_floor_terrgen = terrgen_disable_by_tile_id.should_disable_for("floor_tile_id");
        let floor_template_size = templ_size_query.get(floor_entity_ent).copied().unwrap_or_default().inner();

        let mut chunk_tiles: Vec<(ChunkPos, TilesFromBuilder)> = Vec::with_capacity(build_order.chunks_pos.len());
        let mut terrgen_disabled_gpos_for_chunks = TerrGenDisabledGposForChunks::default();
        let mut tiles4chunk: TilesFromBuilder = Vec::new();
        for &chunk_pos in &build_order.chunks_pos {
            tiles4chunk.clear();
            let mut blocked_gpos = TerrGenBlockedGposMask::default();
            for tile_pos in chunk_pos.get_tilepositions_within_chunk() {
                let local_tile = tile_pos.0 - origin_tile.0;
                if local_tile.x < 0 || local_tile.y < 0 { continue; }
                let idx_x = local_tile.x as usize;
                let idx_y = local_tile.y as usize;
                if idx_x >= map_width || idx_y >= map_height { continue; }
                let map_idx = idx_y * map_width + idx_x;
                if floor_map_bool[map_idx] {
                    tiles4chunk.push((tile_pos, floor_entity, floor_delete_other_tiles.clone()));
                    if disable_floor_terrgen {
                        extend_occupied_gpos(&mut blocked_gpos, chunk_pos, tile_pos, floor_template_size);
                    }
                } else if wall_map[map_idx] {
                    tiles4chunk.push((tile_pos, wall_entity, wall_delete_other_tiles.clone()));
                }
            }
            chunk_tiles.push((chunk_pos, std::mem::take(&mut tiles4chunk)));
            terrgen_disabled_gpos_for_chunks.insert_for_chunk(chunk_pos, blocked_gpos);
        }

        for room in &actual_rooms {
            let Some(anchor_gpos) = room.sample_spawn_anchor(&floor_map, &hazard_map, map_width, map_height, origin_tile, &mut rng, FLOOR_NONE) else { continue; };
            let queued = queue_room_spawn_instance_message(
                room.shape.as_str(),
                anchor_gpos,
                build_order.dimension_ref,
                None,
                &mut beings_remaining,
                &room_spawn_config,
                &room_pack_spawn.source_lookup,
                &mut room_pack_spawn.pending_messages,
                &mut rng,
            );
            if !queued {
                continue;
            }
            trace!(target: DUNGEONING_SYSTEM, "Queued room_spawn InstancePack for structure={} shape={} at {}", structured_gen_cfg.structure_id(), room.shape.as_str(), anchor_gpos);
        }

        compliances_to_emit.push(StructureBuildCompliance {
            i: build_order.i,
            structure_gen_cfg_ent: build_order.structured_gen_cfg_ent,
            dimension_ref: build_order.dimension_ref,
            chunks: chunk_tiles,
            terrgen_disabled_gpos_for_chunks,
            terrgen_disabled_for_chunks: Vec::new(),
            forced_chunk_biomes: Vec::new(),
        });
    }

    writer.write_batch(compliances_to_emit.drain(..));
    room_pack_spawn.finish_pass();
}
