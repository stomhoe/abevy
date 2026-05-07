use bevy::{platform::collections::HashMap, prelude::*};
use bevy::ecs::system::SystemParam;
use std::collections::VecDeque;
use ::being_shared::*;
use common::common_components::StrId;
use common::common_components::HashId;
use common::log_targets::{DUNGEONING_SYSTEM, SGC_INIT};
use tilemap_shared::ArgsDict;
use game_common::game_common_components::Templ;
use rand::{seq::SliceRandom, Rng};
use rand::RngExt;
use tilemap::chunking::macro_chunk_components::BiomeTagWeightAtMacrochunk;
use tilemap::terrain::biome::biome_resources::BiomeEntityMap;
use tilemap::tile::tile_sampler_components::TileWeightedSampler;
use tilemap::tile::{TileEntityMap, TileWeightedSamplerEntityMap};
use ::tilemap_shared::*;

#[derive(Default, Debug)]
pub struct DeleteOtherTilesConfigMap {
    global: Option<DeleteOtherTilesInSamePos>,
    by_placeholder_id: HashMap<String, DeleteOtherTilesInSamePos>,
}

#[derive(Default, Debug)]
pub struct TerrGenDisableConfigMap {
    global: Option<bool>,
    by_placeholder_id: HashMap<String, bool>,
}
impl TerrGenDisableConfigMap {
    pub fn should_disable_for(&self, placeholder_id: &str) -> bool {
        let key = placeholder_id.trim();
        self.by_placeholder_id
            .get(key)
            .copied()
            .or(self.global)
            .unwrap_or(true)
    }

    pub fn from_args(args: &ArgsDict) -> TerrGenDisableConfigMap {
        let mut out = TerrGenDisableConfigMap::default();
        for (key, values) in args.iter() {
            let Some(selector) = key.as_str().strip_prefix("disable_terrgen.") else { continue; };
            let selector = selector.trim();
            if selector.is_empty() {
                continue;
            }
            let Some(disabled) = parse_bool_arg(values) else {
                continue;
            };
            if selector == "*" {
                out.global = Some(disabled);
            } else {
                out.by_placeholder_id.insert(selector.to_string(), disabled);
            }
        }
        out
    }
}

fn parse_bool_arg(value: &SgcArgValue) -> Option<bool> {
    let value = value.first()?.trim();
    match value {
        "true" | "1" | "yes" | "on" => Some(true),
        "false" | "0" | "no" | "off" => Some(false),
        _ => None,
    }
}

fn sgc_arg_value_to_strings(value: &SgcArgValue) -> Vec<String> {
    let Some(list) = value.as_list() else {
        return value.as_scalar_string().into_iter().collect();
    };
    let mut out = Vec::with_capacity(list.len());
    for item in list {
        let Some(item) = item.as_scalar_string() else {
            continue;
        };
        out.push(item);
    }
    out
}

pub fn extend_occupied_gpos(
    blocked_gpos: &mut ChunkGposMask,
    chunk_pos: ChunkPos,
    anchor_gpos: GlobalTilePos,
    size: UVec2,
) {
    for y in anchor_gpos.0.y..(anchor_gpos.0.y + size.y as i32) {
        for x in anchor_gpos.0.x..(anchor_gpos.0.x + size.x as i32) {
            blocked_gpos.set_gpos(chunk_pos, GlobalTilePos::new(x, y));
        }
    }
}

pub fn seal_structure_border_band(
    floor_map: &mut [bool],
    hazard_map: Option<&mut [bool]>,
    tile_width: usize,
    tile_height: usize,
    border_band: usize,
) {
    if border_band == 0 || tile_width == 0 || tile_height == 0 {
        return;
    }
    if border_band * 2 >= tile_width || border_band * 2 >= tile_height {
        floor_map.fill(false);
        if let Some(hazard_map) = hazard_map {
            hazard_map.fill(false);
        }
        return;
    }

    let end_x = tile_width - border_band;
    let end_y = tile_height - border_band;

    let clear_idx = |map: &mut [bool], x: usize, y: usize| {
        map[y * tile_width + x] = false;
    };

    for y in 0..tile_height {
        for x in 0..tile_width {
            if x < border_band || x >= end_x || y < border_band || y >= end_y {
                clear_idx(floor_map, x, y);
            }
        }
    }

    if let Some(hazard_map) = hazard_map {
        for y in 0..tile_height {
            for x in 0..tile_width {
                if x < border_band || x >= end_x || y < border_band || y >= end_y {
                    clear_idx(hazard_map, x, y);
                }
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DoorwayWallAxisPreference {
    Any,
    Horizontal,
    Vertical,
}

#[derive(Clone, Copy, Debug)]
pub struct ExternalDoorwayConfig {
    pub frequency: f32,
    pub width_min: usize,
    pub width_max: usize,
    pub wall_axis_preference: DoorwayWallAxisPreference,
}
impl ExternalDoorwayConfig {
    pub fn from_args(args: &ArgsDict) -> Self {
        let frequency: f32 = args.parse_arg("external_doorway_frequency", 0.0);
        let width_min = args.parse_arg("external_doorway_width_min", 1).clamp(1, 16);
        let width_max = args.parse_arg("external_doorway_width_max", 1).clamp(1, 16);
        let (width_min, width_max) = if width_min <= width_max {
            (width_min, width_max)
        } else {
            (width_max, width_min)
        };
        let wall_axis_preference = args
            .get("external_doorway_wall_axis_preference")
            .and_then(|values| values.first())
            .map(|value| value)
            .map(|value| match value {
                "horizontal" => DoorwayWallAxisPreference::Horizontal,
                "vertical" => DoorwayWallAxisPreference::Vertical,
                _ => DoorwayWallAxisPreference::Any,
            })
            .unwrap_or(DoorwayWallAxisPreference::Any);
        Self {
            frequency: frequency.clamp(0.0, 1.0),
            width_min,
            width_max,
            wall_axis_preference,
        }
    }
}

#[derive(Clone, Copy)]
enum DoorwayAxis {
    HorizontalWall,
    VerticalWall,
}

#[derive(Clone, Copy)]
struct DoorwayCandidate {
    idx: usize,
    axis: DoorwayAxis,
    inward_dx: i32,
    inward_dy: i32,
    outward_dx: i32,
    outward_dy: i32,
}

fn doorway_outward_path_reaches_edge(
    start_x: usize,
    start_y: usize,
    outward_dx: i32,
    outward_dy: i32,
    tile_width: usize,
    tile_height: usize,
    floor_map: &[bool],
    wall_map: &[bool],
    hazard_mask: &[bool],
    outside_map: &[bool],
) -> bool {
    let mut x = start_x as i32 + outward_dx;
    let mut y = start_y as i32 + outward_dy;

    while x >= 0 && y >= 0 && x < tile_width as i32 && y < tile_height as i32 {
        let idx = y as usize * tile_width + x as usize;
        if floor_map[idx] || wall_map[idx] || hazard_mask[idx] || !outside_map[idx] {
            return false;
        }
        if x == 0 || y == 0 || x == tile_width as i32 - 1 || y == tile_height as i32 - 1 {
            return true;
        }
        x += outward_dx;
        y += outward_dy;
    }

    false
}

pub fn carve_external_wall_doorways(
    floor_map: &mut [bool],
    hazard_map: Option<&mut [bool]>,
    wall_map: &mut [bool],
    tile_width: usize,
    tile_height: usize,
    config: ExternalDoorwayConfig,
    rng: &mut impl Rng,
) -> usize {
    if config.frequency <= 0.0 || tile_width < 3 || tile_height < 3 {
        return 0;
    }

    let mut hazard_map = hazard_map;
    let tile_count = tile_width * tile_height;
    let mut hazard_mask = vec![false; tile_count];
    if let Some(hazard_map_ref) = hazard_map.as_deref() {
        hazard_mask.copy_from_slice(hazard_map_ref);
    }
    let mut outside_map = vec![false; tile_count];
    let mut queue = VecDeque::new();

    let push_outside_seed = |x: usize, y: usize, queue: &mut VecDeque<usize>, outside_map: &mut [bool]| {
        let idx = y * tile_width + x;
        if outside_map[idx] || floor_map[idx] || wall_map[idx] || hazard_mask[idx] {
            return;
        }
        outside_map[idx] = true;
        queue.push_back(idx);
    };

    for x in 0..tile_width {
        push_outside_seed(x, 0, &mut queue, &mut outside_map);
        push_outside_seed(x, tile_height - 1, &mut queue, &mut outside_map);
    }
    for y in 0..tile_height {
        push_outside_seed(0, y, &mut queue, &mut outside_map);
        push_outside_seed(tile_width - 1, y, &mut queue, &mut outside_map);
    }

    while let Some(idx) = queue.pop_front() {
        let x = idx % tile_width;
        let y = idx / tile_width;
        let neighbors = [
            (x as i32 - 1, y as i32),
            (x as i32 + 1, y as i32),
            (x as i32, y as i32 - 1),
            (x as i32, y as i32 + 1),
        ];
        for (nx, ny) in neighbors {
            if nx < 0 || ny < 0 || nx >= tile_width as i32 || ny >= tile_height as i32 {
                continue;
            }
            let nidx = ny as usize * tile_width + nx as usize;
            if outside_map[nidx] || floor_map[nidx] || wall_map[nidx] || hazard_mask[nidx] {
                continue;
            }
            outside_map[nidx] = true;
            queue.push_back(nidx);
        }
    }

    let mut candidates = Vec::with_capacity(tile_width.saturating_add(tile_height).saturating_mul(2));
    for y in 1..tile_height - 1 {
        for x in 1..tile_width - 1 {
            let idx = y * tile_width + x;
            if !wall_map[idx] {
                continue;
            }
            let up = idx - tile_width;
            let down = idx + tile_width;
            let left = idx - 1;
            let right = idx + 1;

            let outside_up = outside_map[up];
            let outside_down = outside_map[down];
            let outside_left = outside_map[left];
            let outside_right = outside_map[right];

            if outside_up
                && floor_map[down]
                && !hazard_mask[down]
                && doorway_outward_path_reaches_edge(
                    x,
                    y,
                    0,
                    -1,
                    tile_width,
                    tile_height,
                    floor_map,
                    wall_map,
                    &hazard_mask,
                    &outside_map,
                )
            {
                candidates.push(DoorwayCandidate {
                    idx,
                    axis: DoorwayAxis::HorizontalWall,
                    inward_dx: 0,
                    inward_dy: 1,
                    outward_dx: 0,
                    outward_dy: -1,
                });
                continue;
            }
            if outside_down
                && floor_map[up]
                && !hazard_mask[up]
                && doorway_outward_path_reaches_edge(
                    x,
                    y,
                    0,
                    1,
                    tile_width,
                    tile_height,
                    floor_map,
                    wall_map,
                    &hazard_mask,
                    &outside_map,
                )
            {
                candidates.push(DoorwayCandidate {
                    idx,
                    axis: DoorwayAxis::HorizontalWall,
                    inward_dx: 0,
                    inward_dy: -1,
                    outward_dx: 0,
                    outward_dy: 1,
                });
                continue;
            }
            if outside_left
                && floor_map[right]
                && !hazard_mask[right]
                && doorway_outward_path_reaches_edge(
                    x,
                    y,
                    -1,
                    0,
                    tile_width,
                    tile_height,
                    floor_map,
                    wall_map,
                    &hazard_mask,
                    &outside_map,
                )
            {
                candidates.push(DoorwayCandidate {
                    idx,
                    axis: DoorwayAxis::VerticalWall,
                    inward_dx: 1,
                    inward_dy: 0,
                    outward_dx: -1,
                    outward_dy: 0,
                });
                continue;
            }
            if outside_right
                && floor_map[left]
                && !hazard_mask[left]
                && doorway_outward_path_reaches_edge(
                    x,
                    y,
                    1,
                    0,
                    tile_width,
                    tile_height,
                    floor_map,
                    wall_map,
                    &hazard_mask,
                    &outside_map,
                )
            {
                candidates.push(DoorwayCandidate {
                    idx,
                    axis: DoorwayAxis::VerticalWall,
                    inward_dx: -1,
                    inward_dy: 0,
                    outward_dx: 1,
                    outward_dy: 0,
                });
                continue;
            }
        }
    }

    let mut filtered = Vec::with_capacity(candidates.len());
    filtered.extend(candidates.into_iter().filter(|candidate| match (candidate.axis, config.wall_axis_preference) {
        (_, DoorwayWallAxisPreference::Any) => true,
        (DoorwayAxis::HorizontalWall, DoorwayWallAxisPreference::Horizontal) => true,
        (DoorwayAxis::VerticalWall, DoorwayWallAxisPreference::Vertical) => true,
        _ => false,
    }));
    if filtered.is_empty() {
        return 0;
    }

    filtered.shuffle(rng);
    let mut desired_doorways = ((filtered.len() as f32) * config.frequency).round() as usize;
    if desired_doorways == 0 {
        desired_doorways = 1;
    }
    desired_doorways = desired_doorways.min(filtered.len());

    let mut blocked = vec![false; tile_count];
    let mut opened = 0usize;
    for candidate in filtered {
        if opened >= desired_doorways {
            break;
        }
        if blocked[candidate.idx] {
            continue;
        }

        let width = rng.random_range(config.width_min..=config.width_max);
        let center_x = (candidate.idx % tile_width) as i32;
        let center_y = (candidate.idx / tile_width) as i32;
        let (tangent_dx, tangent_dy) = match candidate.axis {
            DoorwayAxis::HorizontalWall => (1, 0),
            DoorwayAxis::VerticalWall => (0, 1),
        };
        let half = (width as i32) / 2;
        let start = -half;
        let end = start + width as i32 - 1;

        let mut footprint = Vec::with_capacity(width);
        let mut can_carve = true;
        for step in start..=end {
            let x = center_x + tangent_dx * step;
            let y = center_y + tangent_dy * step;
            if x <= 0 || y <= 0 || x >= tile_width as i32 - 1 || y >= tile_height as i32 - 1 {
                can_carve = false;
                break;
            }
            let idx = y as usize * tile_width + x as usize;
            if blocked[idx] || !wall_map[idx] {
                can_carve = false;
                break;
            }
            let inward_x = x + candidate.inward_dx;
            let inward_y = y + candidate.inward_dy;
            let outward_x = x + candidate.outward_dx;
            let outward_y = y + candidate.outward_dy;
            let inward_idx = inward_y as usize * tile_width + inward_x as usize;
            let outward_idx = outward_y as usize * tile_width + outward_x as usize;
            if !outside_map[outward_idx] || !floor_map[inward_idx] || hazard_mask[inward_idx] {
                can_carve = false;
                break;
            }
            footprint.push(idx);
        }
        if !can_carve {
            continue;
        }

        for idx in footprint.iter().copied() {
            wall_map[idx] = false;
            floor_map[idx] = true;
            if let Some(hazard_map_ref) = hazard_map.as_deref_mut() {
                hazard_map_ref[idx] = false;
            }
            hazard_mask[idx] = false;
            blocked[idx] = true;
        }
        opened += 1;
    }

    opened
}

fn parse_chunk_pos_key(key: &str) -> Option<ChunkPos> {
    let key = key.trim().trim_matches(|c| matches!(c, '(' | ')' | '[' | ']'));
    let mut parts = key
        .split([',', ':', ' '])
        .filter(|part| !part.trim().is_empty());
    let x = parts.next()?.trim().parse::<i32>().ok()?;
    let y = parts.next()?.trim().parse::<i32>().ok()?;
    if parts.next().is_some() {
        return None;
    }
    Some(ChunkPos::new(x, y))
}

fn parse_biome_tag_weights(
    value: &SgcArgValue,
    biome_map: &BiomeEntityMap,
) -> Vec<BiomeTagWeightAtMacrochunk> {
    let entries: Vec<&SgcArgValue> = match value {
        SgcArgValue::List(list) => list.iter().collect(),
        SgcArgValue::Map(map) => {
            if let Some(tags) = map.get("biome_tags").and_then(SgcArgValue::as_list) {
                tags.iter().collect()
            } else {
                vec![value]
            }
        }
        _ => Vec::new(),
    };

    let mut out = Vec::with_capacity(entries.len());
    for entry in entries {
        let Some(entry_map) = entry.as_map() else {
            continue;
        };
        let Some(biome_name) = entry_map
            .get("biome")
            .and_then(SgcArgValue::as_str)
            .or_else(|| entry_map.get("biome_tag").and_then(SgcArgValue::as_str))
            .or_else(|| entry_map.get("tag").and_then(SgcArgValue::as_str))
        else {
            continue;
        };
        let Ok(_) = biome_map.0.get_cloned(biome_name.trim()) else {
            warn!(target: "dungeoning", "Forced biome '{}' not found in BiomeEntityMap", biome_name.trim());
            continue;
        };
        let weight = entry_map.get("weight").and_then(SgcArgValue::as_f32).unwrap_or(1.0);
        if !weight.is_finite() || weight <= 0.0 {
            continue;
        }
        let pack_count_multiplier_mean = entry_map
            .get("pack_count_multiplier_mean")
            .and_then(SgcArgValue::as_f32)
            .unwrap_or(1.0)
            .max(0.0);
        let pack_count_multiplier_std_dev = entry_map
            .get("pack_count_multiplier_std_dev")
            .and_then(SgcArgValue::as_f32)
            .unwrap_or(0.0)
            .max(0.0);
        out.push(BiomeTagWeightAtMacrochunk {
            biome: HashId::from(biome_name.trim()),
            weight,
            pack_count_multiplier_mean,
            pack_count_multiplier_std_dev,
        });
    }
    out
}

pub fn forced_chunk_biomes_from_args(
    args: &SgcArgsDict,
    biome_map: &BiomeEntityMap,
) -> Vec<ForcedChunkBiomeConfig> {
    let Some(forced_map) = args.get_map("forced_chunk_biomes") else {
        return Vec::new();
    };

    let mut out = Vec::with_capacity(forced_map.len());
    for (chunk_key, biome_value) in forced_map {
        let Some(chunk_pos) = parse_chunk_pos_key(chunk_key) else {
            warn!(target: "dungeoning", "Invalid forced_chunk_biomes chunk key '{}'", chunk_key);
            continue;
        };
        let biome_tags = parse_biome_tag_weights(biome_value, biome_map);
        if biome_tags.is_empty() {
            continue;
        }
        out.push(ForcedChunkBiomeConfig { chunk_pos, biome_tags });
    }
    out
}
impl DeleteOtherTilesConfigMap {
    pub fn get(&self, placeholder_id: &str) -> Option<DeleteOtherTilesInSamePos> {
        let mut merged = self.global.clone().unwrap_or_default();
        let mut has_any = self.global.is_some();
        let key = placeholder_id.trim();
        let Some(spec) = self.by_placeholder_id.get(key) else {
            return has_any.then_some(merged);
        };
        merged.merge_from(spec);
        has_any = true;
        has_any.then_some(merged)
    }

    pub fn from_args(args: &ArgsDict) -> DeleteOtherTilesConfigMap {
        let mut out = DeleteOtherTilesConfigMap::default();
        for (key, values) in args.iter() {
            let Some(key) = key.as_str().strip_prefix("delete.") else { continue; };
            let Some((selector, field)) = key.rsplit_once('.') else {
                continue;
            };
            let selector = selector.trim();
            if selector.is_empty() {
                continue;
            }
            let mut parsed_spec = DeleteOtherTilesInSamePos::default();
            let values = sgc_arg_value_to_strings(values);
            if !parsed_spec.apply_delete_other_tiles_field(field, &values) {
                continue;
            }
            if selector == "*" {
                let spec = out.global.get_or_insert_with(DeleteOtherTilesInSamePos::default);
                spec.merge_from(&parsed_spec);
            } else {
                let placeholder_id = selector.to_string();
                let spec = out.by_placeholder_id.entry(placeholder_id).or_default();
                spec.merge_from(&parsed_spec);
            }
        }
        out
    }
}

pub fn resolve_sampled_tile_entity_from_sampler(
    root_sampler: &HashIdWeightedSampler,
    sampler_query: &Query<&HashIdWeightedSampler, (With<TileWeightedSampler>, common::AnyDisabling)>,
    sampler_map: &TileWeightedSamplerEntityMap,
    tile_map: &TileEntityMap,
    anchor_gpos: GlobalTilePos,
    settings: &GlobalGenSettings,
    dimension_hash: HashId,
) -> Option<Entity> {
    let mut current_sampler = root_sampler;
    let mut depth = 0u8;
    while depth < 8 {
        let sampled_hash_id = current_sampler.sample_with_pos(anchor_gpos, settings, dimension_hash)?;
        if let Some(sampled_sampler_ent) = sampler_map.0.get_opt(sampled_hash_id).copied() {
            let Ok(next_sampler) = sampler_query.get(sampled_sampler_ent) else {
                return None;
            };
            current_sampler = next_sampler;
            depth += 1;
            continue;
        }
        return tile_map.0.get_opt(sampled_hash_id).copied();
    }
    None
}

#[derive(Debug, Clone, PartialEq)]
pub struct DungeonRoomPackSpawnSpec {
    pub source_template_id: String,
    pub override_being_count: Option<u16>,
    pub pack_spawn_radius: Option<u8>,
    pub radius_min_pct: i32,
    pub radius_max_pct: i32,
    pub weight: f32,
}

define_weightedsampler!(DungeonRoomPackSpawnSampler, DungeonRoomPackSpawnSpec, "DungeonRoomPackSpawnSampler");

#[derive(Default, Debug, Clone)]
pub struct DungeonRoomPackSpawnConfig(pub HashMap<String, DungeonRoomPackSpawnSampler>);

fn is_room_spawn_shape_allowed(
    structure_id: &str,
    allowed_shapes: Option<&HashSet<String>>,
    shape_key: &str,
) -> bool {
    if structure_id == "chamberscorridors" {
        return true;
    }
    if let Some(allowed_shapes) = allowed_shapes {
        if allowed_shapes.contains(shape_key) {
            return true;
        }
        return false;
    }
    true
}

impl DungeonRoomPackSpawnConfig {

    pub fn sample_room_spawn_spec<'a>(
        &'a self,
        room_spawn_key: &str,
        rng: &mut impl Rng,
    ) -> Option<DungeonRoomPackSpawnSpec> {
        self.sample_room_spawn_spec_for_radius_pct(room_spawn_key, None, rng)
    }

    pub fn sample_room_spawn_spec_for_radius_pct<'a>(
        &'a self,
        room_spawn_key: &str,
        radius_pct: Option<i32>,
        rng: &mut impl Rng,
    ) -> Option<DungeonRoomPackSpawnSpec> {
        let sampler = self.0.get(room_spawn_key)?;
        let Some(radius_pct) = radius_pct else {
            return sampler.sample_with_rng(rng);
        };
        let radius_pct = radius_pct.clamp(0, 100);
        let mut matching_weights = Vec::with_capacity(sampler.len());
        for (spec, weight) in sampler.iter() {
            if spec.radius_min_pct <= radius_pct && radius_pct <= spec.radius_max_pct {
                matching_weights.push((spec.clone(), *weight));
            }
        }
        if matching_weights.is_empty() {
            debug!(
                target: DUNGEONING_SYSTEM,
                "room_spawn key={} has no spec matching radius_pct={}, falling back to unfiltered sampling",
                room_spawn_key,
                radius_pct,
            );
            return None;
        }
        let (sampler, negative_items) = DungeonRoomPackSpawnSampler::new(&matching_weights);
        for negative_item in negative_items {
            error!(target: "dungeoning_utils", "Weighted sampler {} encountered a negative weight for value {:?}; rejected", room_spawn_key, negative_item);
        }
        sampler.sample_with_rng(rng)
    }

    pub fn from_typed_args(
        args: &SgcArgsDict,
        command_registry: &SgcCommandRegistry,
        structure_id: &str,
    ) -> Self {
        let mut out = Self::default();
        let allowed_shapes = command_registry.allowed_room_spawn_shapes_for(structure_id);

        if let Some(room_spawn_map) = args.get_map("room_spawn") {
            for (shape_key, spec_value) in room_spawn_map {
                if !is_room_spawn_shape_allowed(structure_id, allowed_shapes, shape_key.as_str()) {
                    continue;
                }
                let Some(sampler) = parse_room_spawn_sampler(spec_value) else {
                    continue;
                };
                let entry = out.0.entry(shape_key.clone()).or_default();
                for (spec, weight) in sampler.iter().cloned() {
                    if let Err(negative_item) = entry.insert(spec, weight) {
                        error!(target: "dungeoning_utils", "Weighted sampler {} encountered a negative weight for value {:?}; rejected", shape_key, negative_item);
                    }
                }
            }
        }

        let mut dotted_specs: HashMap<String, DungeonRoomPackSpawnSpec> = HashMap::default();
        for (key, value) in args.iter() {
            let Some((_, rest)) = key.split_once("room_spawn.") else {
                continue;
            };
            // Support compact entries like:
            // room_spawn.some_key = [{...}, {...}]
            // alongside legacy dotted fields:
            // room_spawn.some_key.source_id = foo
            if !rest.contains('.') {
                if rest.is_empty() {
                    continue;
                }
                if !is_room_spawn_shape_allowed(structure_id, allowed_shapes, rest) {
                    continue;
                }
                let Some(sampler) = parse_room_spawn_sampler(value) else {
                    continue;
                };
                let entry = out.0.entry(rest.to_string()).or_default();
                for (spec, weight) in sampler.iter().cloned() {
                    if let Err(negative_item) = entry.insert(spec, weight) {
                        error!(target: "dungeoning_utils", "Weighted sampler {} encountered a negative weight for value {:?}; rejected", rest, negative_item);
                    }
                }
                continue;
            }
            let Some((shape_key, field)) = rest.split_once('.') else {
                continue;
            };
            if shape_key.is_empty() || field.is_empty() {
                continue;
            }
            if !is_room_spawn_shape_allowed(structure_id, allowed_shapes, shape_key) {
                continue;
            }
            let entry = dotted_specs.entry(shape_key.to_string()).or_insert_with(|| DungeonRoomPackSpawnSpec {
                source_template_id: String::new(),
                override_being_count: None,
                pack_spawn_radius: None,
                radius_min_pct: 0,
                radius_max_pct: 100,
                weight: 1.0,
            });
            match field {
                "pack_id" | "source_id" | "source_template_id" => {
                    let Some(source_template_id) = value.as_scalar_string() else {
                        continue;
                    };
                    entry.source_template_id = source_template_id;
                }
                "override_being_count" | "count" => {
                    entry.override_being_count = match value {
                        SgcArgValue::Int(raw) => u16::try_from(*raw).ok(),
                        SgcArgValue::Str(raw) => raw.parse::<u16>().ok(),
                        _ => None,
                    };
                }
                "pack_spawn_radius" | "spawn_radius" => {
                    entry.pack_spawn_radius = match value {
                        SgcArgValue::Int(raw) => u8::try_from(*raw).ok(),
                        SgcArgValue::Str(raw) => raw.parse::<u8>().ok(),
                        _ => None,
                    };
                }
                "radius_min_pct" | "radius_min" | "min_radius_pct" => {
                    entry.radius_min_pct = match value {
                        SgcArgValue::Int(raw) => i32::try_from(*raw).ok().unwrap_or(entry.radius_min_pct),
                        SgcArgValue::Str(raw) => raw.parse::<i32>().unwrap_or(entry.radius_min_pct),
                        _ => entry.radius_min_pct,
                    };
                }
                "radius_max_pct" | "radius_max" | "max_radius_pct" => {
                    entry.radius_max_pct = match value {
                        SgcArgValue::Int(raw) => i32::try_from(*raw).ok().unwrap_or(entry.radius_max_pct),
                        SgcArgValue::Str(raw) => raw.parse::<i32>().unwrap_or(entry.radius_max_pct),
                        _ => entry.radius_max_pct,
                    };
                }
                "weight" => {
                    entry.weight = match value {
                        SgcArgValue::Int(raw) => (*raw as f32).max(0.0),
                        SgcArgValue::Str(raw) => raw.parse::<f32>().unwrap_or(1.0).max(0.0),
                        _ => entry.weight,
                    };
                }
                _ => {}
            }
        }

        for (shape_key, spec) in dotted_specs {
            if spec.source_template_id.trim().is_empty() {
                continue;
            }
            let entry = out.0.entry(shape_key.clone()).or_default();
            if let Err(negative_item) = entry.insert(spec.clone(), spec.weight) {
                error!(target: "dungeoning_utils", "Weighted sampler {} encountered a negative weight for value {:?}; rejected", shape_key, negative_item);
            }
        }

        out.0.retain(|_, sampler| !sampler.is_empty());
        if out.0.is_empty() {
            error!(
                target: SGC_INIT,
                "SGC structure={} has no valid room_spawn packs configured",
                structure_id
            );
        }
        out
    }
}

#[derive(SystemParam)]
pub struct DungeonRoomPackSpawnSystemParams<'w, 's> {
    pub command_registry: Res<'w, SgcCommandRegistry>,
    pub pack_templates: Query<'w, 's, (Entity, &'static StrId, ), (With<Pack>, With<Templ>, common::AnyDisabling),>,
    pub race_templates: Query<'w, 's, (Entity, &'static StrId, ), (With<Race>, With<Templ>, common::AnyDisabling),>,
    pub bit_templates: Query<'w, 's, (Entity, &'static StrId, ), (With<BeingInstTemplate>, With<Templ>, common::AnyDisabling),>,
    pub writer: MessageWriter<'w, InstantiateTemplPackEntity>,
    pub pending_messages: Local<'s, Vec<InstantiateTemplPackEntity>>,
    pub source_lookup: Local<'s, HashMap<StrId, Entity>>,
}

impl DungeonRoomPackSpawnSystemParams<'_, '_> {
    pub fn begin_pass(&mut self) {
        self.pending_messages.clear();
        self.source_lookup.clear();
        rebuild_spawn_source_lookup(
            &self.pack_templates,
            &self.race_templates,
            &self.bit_templates,
            &mut self.source_lookup,
        );
    }

    pub fn finish_pass(&mut self) {
        self.writer.write_batch(self.pending_messages.drain(..));
    }
}

fn parse_room_spawn_spec(value: &SgcArgValue) -> Option<DungeonRoomPackSpawnSpec> {
    let map = value.as_map()?;
    let source_template_id = map
        .get("pack_id")
        .or_else(|| map.get("source_id"))
        .or_else(|| map.get("source_template_id"))
        .and_then(SgcArgValue::as_scalar_string)?;
    let override_being_count = map
        .get("override_being_count")
        .or_else(|| map.get("count"))
        .and_then(|value| match value {
            SgcArgValue::Int(raw) => u16::try_from(*raw).ok(),
            SgcArgValue::Str(raw) => raw.parse::<u16>().ok(),
            _ => None,
        });
    let pack_spawn_radius = map
        .get("pack_spawn_radius")
        .or_else(|| map.get("spawn_radius"))
        .and_then(|value| match value {
            SgcArgValue::Int(raw) => u8::try_from(*raw).ok(),
            SgcArgValue::Str(raw) => raw.parse::<u8>().ok(),
            _ => None,
        });
    let radius_min_pct = map
        .get("radius_min_pct")
        .or_else(|| map.get("radius_min"))
        .or_else(|| map.get("min_radius_pct"))
        .and_then(|value| match value {
            SgcArgValue::Int(raw) => i32::try_from(*raw).ok(),
            SgcArgValue::Str(raw) => raw.parse::<i32>().ok(),
            _ => None,
        })
        .unwrap_or(0)
        .clamp(0, 100);
    let radius_max_pct = map
        .get("radius_max_pct")
        .or_else(|| map.get("radius_max"))
        .or_else(|| map.get("max_radius_pct"))
        .and_then(|value| match value {
            SgcArgValue::Int(raw) => i32::try_from(*raw).ok(),
            SgcArgValue::Str(raw) => raw.parse::<i32>().ok(),
            _ => None,
        })
        .unwrap_or(100)
        .clamp(0, 100);
    let (radius_min_pct, radius_max_pct) = if radius_min_pct <= radius_max_pct {
        (radius_min_pct, radius_max_pct)
    } else {
        (radius_max_pct, radius_min_pct)
    };
    let weight = map
        .get("weight")
        .and_then(|value| match value {
            SgcArgValue::Int(raw) => Some((*raw as f32).max(0.0)),
            SgcArgValue::Str(raw) => raw.parse::<f32>().ok().map(|weight| weight.max(0.0)),
            _ => None,
        })
        .unwrap_or(1.0);
    Some(DungeonRoomPackSpawnSpec {
        source_template_id,
        override_being_count,
        pack_spawn_radius,
        radius_min_pct,
        radius_max_pct,
        weight,
    })
}

pub fn rebuild_spawn_source_lookup(
    pack_templates: &Query<(Entity, &StrId, ), (With<Pack>, With<Templ>, common::AnyDisabling),>,
    race_templates: &Query<(Entity, &StrId, ), (With<Race>, With<Templ>, common::AnyDisabling),>,
    bit_templates: &Query<(Entity, &StrId, ), (With<BeingInstTemplate>, With<Templ>, common::AnyDisabling),>,
    source_lookup: &mut HashMap<StrId, Entity>,
) {
    source_lookup.clear();
    source_lookup.reserve(
        pack_templates.iter().len()
            + race_templates.iter().len()
            + bit_templates.iter().len(),
    );
    for (entity, str_id) in pack_templates.iter() {
        source_lookup.insert(str_id.clone(), entity);
    }
    for (entity, str_id) in race_templates.iter() {
        source_lookup.entry(str_id.clone()).or_insert(entity);
    }
    for (entity, str_id) in bit_templates.iter() {
        source_lookup.entry(str_id.clone()).or_insert(entity);
    }
}

pub fn queue_room_spawn_instance_message(
    room_spawn_key: &str,
    anchor_gpos: GlobalTilePos,
    dim_ref: DimensionRef,
    room_spawn_radius_pct: Option<i32>,
    beings_remaining: &mut u32,
    room_spawn_config: &DungeonRoomPackSpawnConfig,
    source_lookup: &HashMap<StrId, Entity>,
    pending_messages: &mut Vec<InstantiateTemplPackEntity>,
    rng: &mut impl Rng,
) -> bool {
    let room_spec = sample_room_spawn_spec_with_aliases(
        room_spawn_config,
        room_spawn_key,
        room_spawn_radius_pct,
        rng,
    );
    let Some(room_spec) = room_spec else {
        return false;
    };
    if *beings_remaining == 0 {
        return false;
    }
    let source_id = StrId::trunc(room_spec.source_template_id.as_str());
    let Some(&source_ent) = source_lookup.get(&source_id) else {
        return false;
    };
    let requested_being_count = room_spec.override_being_count.map(u32::from).unwrap_or(1);
    let final_being_count = requested_being_count.min(*beings_remaining);
    if final_being_count == 0 {
        return false;
    }
    *beings_remaining = beings_remaining.saturating_sub(final_being_count);
    let override_being_count = room_spec
        .override_being_count
        .and_then(|_| u16::try_from(final_being_count).ok());
    let mut message = InstantiateTemplPackEntity::new(
        source_ent,
        override_being_count,
        None,
        room_spec.pack_spawn_radius.or(Some(0)),
        dim_ref,
        [anchor_gpos],
    );
    message.only_same_island = true;
    pending_messages.push(message);
    true
}

fn parse_room_spawn_sampler(value: &SgcArgValue) -> Option<DungeonRoomPackSpawnSampler> {
    let specs = match value {
        SgcArgValue::Map(_) => {
            let spec = parse_room_spawn_spec(value)?;
            vec![spec]
        }
        SgcArgValue::List(list) => {
            let mut specs = Vec::with_capacity(list.len());
            for entry in list {
                let Some(spec) = parse_room_spawn_spec(entry) else {
                    continue;
                };
                specs.push(spec);
            }
            specs
        }
        _ => Vec::new(),
    };
    if specs.is_empty() {
        return None;
    }
    let weights = specs.iter().cloned().map(|spec| (spec.clone(), spec.weight)).collect::<Vec<_>>();
    Some({
        let (sampler, negative_items) = DungeonRoomPackSpawnSampler::new(&weights);
            for negative_item in negative_items {
                error!(target: "dungeoning_utils", "Weighted sampler {} encountered a negative weight for value {:?}; rejected", "room_spawn_sampler", negative_item);
            }
        sampler
    })
}


fn sample_room_spawn_spec_with_aliases(
    room_spawn_config: &DungeonRoomPackSpawnConfig,
    room_spawn_key: &str,
    room_spawn_radius_pct: Option<i32>,
    rng: &mut impl Rng,
) -> Option<DungeonRoomPackSpawnSpec> {
    let direct = match room_spawn_radius_pct {
        Some(room_spawn_radius_pct) => room_spawn_config.sample_room_spawn_spec_for_radius_pct(
            room_spawn_key,
            Some(room_spawn_radius_pct),
            rng,
        ),
        None => room_spawn_config.sample_room_spawn_spec(room_spawn_key, rng),
    };
    if direct.is_some() {
        return direct;
    }

    let alias_key = match room_spawn_key {
        "ellipse" => Some("circle"),
        "circle" => Some("ellipse"),
        "trapezoid" => Some("triangle"),
        "triangle" => Some("trapezoid"),
        "regular_polygon" => Some("polygon"),
        "polygon" => Some("regular_polygon"),
        _ => None,
    }?;

    match room_spawn_radius_pct {
        Some(room_spawn_radius_pct) => room_spawn_config.sample_room_spawn_spec_for_radius_pct(
            alias_key,
            Some(room_spawn_radius_pct),
            rng,
        ),
        None => room_spawn_config.sample_room_spawn_spec(alias_key, rng),
    }
}
