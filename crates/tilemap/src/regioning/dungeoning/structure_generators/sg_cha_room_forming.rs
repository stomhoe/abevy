use bevy::{platform::collections::*, prelude::*};
use rand::Rng;
use std::collections::VecDeque;

use super::sg_cha_types::*;

const ROOM_INTERIOR_CHANCE: f32 = 0.5;
const SUBROOM_CHANCE: f32 = 0.5;
const SUBROOM_MIN_LEAF_DIM: i32 = 4;
const SUBROOM_AREA_PER_LEAF: f32 = 100.0;
const SUBROOM_MIN_TARGETS: usize = 3;
const SUBROOM_MAX_TARGETS: usize = 10;
const RECTANGLE_CIRCUMFERENCE_CHANCE: f32 = 0.25;

#[derive(Clone, Copy, Debug)]
enum RectangularRoomDecoration {
    Subrooms,
    Circumference,
    Pillars,
}

impl RectangularRoomDecoration {
    fn pick(rng: &mut impl Rng) -> Self {
        let roll = rng.random_range(0.0..1.0);
        if roll < RECTANGLE_CIRCUMFERENCE_CHANCE {
            RectangularRoomDecoration::Circumference
        } else if rng.random_bool(0.6) {
            RectangularRoomDecoration::Subrooms
        } else {
            RectangularRoomDecoration::Pillars
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct RoomDraft {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

impl RoomDraft {
    pub fn area(self) -> i32 {
        self.w * self.h
    }

    pub fn as_room(self) -> Room {
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

pub fn sample_room_spec_for_leaf_with_limit(
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

pub fn sample_trapezoid_vertices_for_room(room: &Room, rng: &mut impl Rng) -> ((i32, i32), (i32, i32), (i32, i32), (i32, i32)) {
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

pub fn maybe_add_rectangular_subrooms(
    rng: &mut impl Rng,
    room_x: i32,
    room_y: i32,
    room_w: i32,
    room_h: i32,
    mut clear_tile: impl FnMut(usize, usize),
) {
    if room_w < 10 || room_h < 10 {
        return;
    }
    if rng.random_range(0.0..1.0) >= SUBROOM_CHANCE {
        return;
    }

    let area = room_w.saturating_mul(room_h).max(1) as usize;
    let max_subrooms = ((area as f32 / SUBROOM_AREA_PER_LEAF).round() as usize).clamp(SUBROOM_MIN_TARGETS, SUBROOM_MAX_TARGETS);
    let target_subrooms = rng.random_range(SUBROOM_MIN_TARGETS..=max_subrooms);
    let mut regions = VecDeque::from([RectRegion { x: room_x, y: room_y, w: room_w, h: room_h }]);
    let mut finalized_regions = 0_usize;
    let prefer_center_gap = true;

    while finalized_regions + regions.len() < target_subrooms {
        let Some(region) = regions.pop_front() else {
            break;
        };

        let Some((first, second, partition)) = split_region(region, rng, prefer_center_gap) else {
            finalized_regions = finalized_regions.saturating_add(1);
            continue;
        };

        carve_partition(partition, &mut clear_tile);
        if rng.random_bool(0.5) {
            regions.push_back(first);
            regions.push_back(second);
        } else {
            regions.push_back(second);
            regions.push_back(first);
        }
    }
}

pub fn split_room_draft_horizontally(room: RoomDraft, split_at: i32, keep_left: bool) -> (RoomDraft, Option<RoomDraft>) {
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

pub fn split_room_draft_vertically(room: RoomDraft, split_at: i32, keep_top: bool) -> (RoomDraft, Option<RoomDraft>) {
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

pub fn room_center(room: &Room) -> (i32, i32) {
    (room.x + room.w / 2, room.y + room.h / 2)
}

pub fn sample_corridor_child_slots(rng: &mut impl Rng) -> usize {
    match rng.random_range(0..100) {
        0..=14 => 0,
        15..=84 => 1,
        85..=95 => 2,
        _ => 3,
    }
}

pub fn pick_weighted_open_parent_index(open_parents: &[(usize, usize)], rng: &mut impl Rng) -> Option<usize> {
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

pub fn pick_nearest_room_index(parent_idx: usize, candidate_indices: &[usize], rooms: &[Room], rng: &mut impl Rng) -> Option<usize> {
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

#[derive(Clone, Copy)]
struct RectRegion {
    x: i32,
    y: i32,
    w: i32,
    h: i32,
}

#[derive(Clone, Copy)]
enum PartitionAxis {
    Vertical,
    Horizontal,
}

fn pick_balanced_split(min: i32, max: i32, center: i32, rng: &mut impl Rng) -> i32 {
    let span = max - min;
    if span <= 0 {
        return min;
    }

    let max_offset = (span / 4).clamp(1, 3);
    let lower = (center - max_offset).max(min);
    let upper = (center + max_offset).min(max);
    if lower <= upper {
        rng.random_range(lower..=upper)
    } else {
        rng.random_range(min..=max)
    }
}

fn split_region(
    region: RectRegion,
    rng: &mut impl Rng,
    use_center_corridor: bool,
) -> Option<(RectRegion, RectRegion, Partition)> {
    let split_vertical = if region.w > region.h {
        true
    } else if region.h > region.w {
        false
    } else {
        rng.random_bool(0.5)
    };

    if split_vertical {
        let min_split_x = region.x + SUBROOM_MIN_LEAF_DIM;
        let max_split_x = region.x + region.w - SUBROOM_MIN_LEAF_DIM - 1;
        if max_split_x < min_split_x {
            return None;
        }
        let split_x = pick_balanced_split(min_split_x, max_split_x, region.x + region.w / 2, rng);
        let gap_y = pick_partition_gap(region.y + 1, region.y + region.h - 1, region.y + region.h / 2, use_center_corridor, rng);
        let first = RectRegion { x: region.x, y: region.y, w: split_x - region.x, h: region.h };
        let second = RectRegion { x: split_x + 1, y: region.y, w: region.x + region.w - split_x - 1, h: region.h };
        if first.w < SUBROOM_MIN_LEAF_DIM || second.w < SUBROOM_MIN_LEAF_DIM {
            return None;
        }
        Some((first, second, Partition { axis: PartitionAxis::Vertical, x: split_x, y: gap_y, line_min: region.y + 1, line_max: region.y + region.h - 2 }))
    } else {
        let min_split_y = region.y + SUBROOM_MIN_LEAF_DIM;
        let max_split_y = region.y + region.h - SUBROOM_MIN_LEAF_DIM - 1;
        if max_split_y < min_split_y {
            return None;
        }
        let split_y = pick_balanced_split(min_split_y, max_split_y, region.y + region.h / 2, rng);
        let gap_x = pick_partition_gap(region.x + 1, region.x + region.w - 1, region.x + region.w / 2, use_center_corridor, rng);
        let first = RectRegion { x: region.x, y: region.y, w: region.w, h: split_y - region.y };
        let second = RectRegion { x: region.x, y: split_y + 1, w: region.w, h: region.y + region.h - split_y - 1 };
        if first.h < SUBROOM_MIN_LEAF_DIM || second.h < SUBROOM_MIN_LEAF_DIM {
            return None;
        }
        Some((first, second, Partition { axis: PartitionAxis::Horizontal, x: gap_x, y: split_y, line_min: region.x + 1, line_max: region.x + region.w - 2 }))
    }
}

#[derive(Clone, Copy)]
struct Partition {
    axis: PartitionAxis,
    x: i32,
    y: i32,
    line_min: i32,
    line_max: i32,
}

fn pick_partition_gap(
    min: i32,
    max: i32,
    center: i32,
    use_center_corridor: bool,
    rng: &mut impl Rng,
) -> i32 {
    let lower = min.max(center - 1);
    let upper = (max - 1).min(center + 1);
    if use_center_corridor && lower <= upper {
        return rng.random_range(lower..=upper);
    }

    let mut gap = rng.random_range(min..max);
    if gap == center {
        gap = if gap + 1 < max { gap + 1 } else { gap - 1 };
    }
    gap
}

fn carve_partition(partition: Partition, clear_tile: &mut impl FnMut(usize, usize)) {
    match partition.axis {
        PartitionAxis::Vertical => {
            for y in partition.line_min..=partition.line_max {
                if y == partition.y {
                    continue;
                }
                clear_tile(partition.x as usize, y as usize);
            }
            clear_tile(partition.x as usize, partition.y as usize);
        }
        PartitionAxis::Horizontal => {
            for x in partition.line_min..=partition.line_max {
                if x == partition.x {
                    continue;
                }
                clear_tile(x as usize, partition.y as usize);
            }
            clear_tile(partition.x as usize, partition.y as usize);
        }
    }
}

pub fn maybe_add_room_interior(
    rng: &mut impl Rng,
    room_shape: RoomShape,
    room_x: i32,
    room_y: i32,
    room_w: i32,
    room_h: i32,
    mut clear_tile: impl FnMut(usize, usize),
) {
    if room_w < 7 || room_h < 7 {
        return;
    }
    if rng.random_range(0.0..1.0) >= ROOM_INTERIOR_CHANCE {
        return;
    }

    match room_shape {
        RoomShape::Rectangle => {
            if room_w >= 10 && room_h >= 10 {
                match RectangularRoomDecoration::pick(rng) {
                    RectangularRoomDecoration::Subrooms => {
                        maybe_add_rectangular_subrooms(rng, room_x, room_y, room_w, room_h, clear_tile);
                    }
                    RectangularRoomDecoration::Circumference => {
                        maybe_add_room_circumference(rng, room_x, room_y, room_w, room_h, &mut clear_tile);
                    }
                    RectangularRoomDecoration::Pillars => {
                        maybe_add_room_pillars(rng, room_x, room_y, room_w, room_h, clear_tile);
                    }
                }
            } else {
                maybe_add_room_pillars(rng, room_x, room_y, room_w, room_h, clear_tile);
            }
        }
        RoomShape::RegularPolygon => {
            maybe_add_room_pillars(rng, room_x, room_y, room_w, room_h, clear_tile);
        }
        _ => {
            maybe_add_room_pillars(rng, room_x, room_y, room_w, room_h, clear_tile);
        }
    }
}

fn maybe_add_room_pillars(
    rng: &mut impl Rng,
    room_x: i32,
    room_y: i32,
    room_w: i32,
    room_h: i32,
    mut clear_tile: impl FnMut(usize, usize),
) {
    match rng.random_range(0..2) {
        0 => add_circumference_pillars(room_x, room_y, room_w, room_h, &mut clear_tile),
        _ => add_inset_grid_pillars(room_x, room_y, room_w, room_h, &mut clear_tile),
    }
}

fn add_circumference_pillars(
    room_x: i32,
    room_y: i32,
    room_w: i32,
    room_h: i32,
    clear_tile: &mut impl FnMut(usize, usize),
) {
    let pillar_count = ((room_w.max(room_h) as usize) / 2).clamp(8, 16);
    let center_x = room_x as f32 + (room_w as f32 - 1.0) * 0.5;
    let center_y = room_y as f32 + (room_h as f32 - 1.0) * 0.5;
    let radius_x = (room_w as f32 * 0.66 * 0.5).max(2.0);
    let radius_y = (room_h as f32 * 0.66 * 0.5).max(2.0);

    for i in 0..pillar_count {
        let angle = (i as f32 / pillar_count as f32) * std::f32::consts::TAU;
        let x = (center_x + radius_x * angle.cos()).round() as i32;
        let y = (center_y + radius_y * angle.sin()).round() as i32;
        if x >= room_x && x < room_x + room_w && y >= room_y && y < room_y + room_h {
            clear_tile(x as usize, y as usize);
        }
    }
}

fn maybe_add_room_circumference(
    rng: &mut impl Rng,
    room_x: i32,
    room_y: i32,
    room_w: i32,
    room_h: i32,
    clear_tile: &mut impl FnMut(usize, usize),
) {
    let min_dimension = room_w.min(room_h);
    if min_dimension < 9 {
        return;
    }

    let inset = rng.random_range(2..=((min_dimension / 3).max(2)));
    let inner_x = room_x + inset;
    let inner_y = room_y + inset;
    let inner_w = room_w - inset * 2;
    let inner_h = room_h - inset * 2;
    if inner_w < 4 || inner_h < 4 {
        return;
    }

    let opening_side = rng.random_range(0..4);
    let opening_pos = match opening_side {
        0 | 1 => inner_x + inner_w / 2,
        _ => inner_y + inner_h / 2,
    };

    for x in inner_x..inner_x + inner_w {
        if opening_side != 0 || x != opening_pos {
            clear_tile(x as usize, inner_y as usize);
        }
        if opening_side != 1 || x != opening_pos {
            clear_tile(x as usize, (inner_y + inner_h - 1) as usize);
        }
    }

    for y in inner_y..inner_y + inner_h {
        if opening_side != 2 || y != opening_pos {
            clear_tile(inner_x as usize, y as usize);
        }
        if opening_side != 3 || y != opening_pos {
            clear_tile((inner_x + inner_w - 1) as usize, y as usize);
        }
    }
}

fn add_inset_grid_pillars(
    room_x: i32,
    room_y: i32,
    room_w: i32,
    room_h: i32,
    clear_tile: &mut impl FnMut(usize, usize),
) {
    let inset_x = (room_w / 4).clamp(2, 6);
    let inset_y = (room_h / 4).clamp(2, 6);
    let start_x = room_x + inset_x;
    let end_x = room_x + room_w - inset_x - 1;
    let start_y = room_y + inset_y;
    let end_y = room_y + room_h - inset_y - 1;
    if end_x <= start_x || end_y <= start_y {
        return;
    }

    let step = if room_w.min(room_h) >= 18 { 7 } else { 7 };
    let mut y = start_y;
    while y <= end_y {
        let mut x = start_x;
        while x <= end_x {
            clear_tile(x as usize, y as usize);
            x += step;
        }
        y += step;
    }
}