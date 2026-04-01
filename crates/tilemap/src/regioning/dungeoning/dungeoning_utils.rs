use bevy::{platform::collections::HashMap, prelude::*};
use bevy::ecs::system::SystemParam;
use ::being_shared::*;
use common::common_components::StrId;
use common::common_components::HashId;
use game_common::game_common_components::ArgsDict;
use game_common::game_common_components::Templ;
use crate::tile::tile_sampler_components::TileWeightedSampler;
use crate::terrain::terrgen_async_resources::TerrGenBlockedGposMask;
use crate::regioning::regioning_resources::SgcCommandRegistry;
use crate::regioning::regioning_sgc_seris::{SgcArgValue, SgcArgsDict};
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

fn parse_bool_arg(values: &[String]) -> Option<bool> {
    let value = values.first()?.trim();
    match value {
        "true" | "1" | "yes" | "on" => Some(true),
        "false" | "0" | "no" | "off" => Some(false),
        _ => None,
    }
}

pub fn extend_occupied_gpos(
    blocked_gpos: &mut TerrGenBlockedGposMask,
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
            if !parsed_spec.apply_delete_other_tiles_field(field, values) {
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
    root_sampler: &EntityWeightedSampler,
    sampler_query: &Query<&EntityWeightedSampler, (With<TileWeightedSampler>, common::AnyDisabling)>,
    anchor_gpos: GlobalTilePos,
    settings: &GlobalGenSettings,
    dimension_hash: HashId,
) -> Option<Entity> {
    let mut current_sampler = root_sampler;
    let mut depth = 0u8;
    while depth < 8 {
        let sampled_ent = current_sampler.sample_with_pos(anchor_gpos, settings, dimension_hash)?;
        if let Ok(next_sampler) = sampler_query.get(sampled_ent) {
            current_sampler = next_sampler;
            depth += 1;
            continue;
        }
        return Some(sampled_ent);
    }
    None
}

#[derive(Debug, Clone)]
pub struct DungeonRoomPackSpawnSpec {
    pub source_template_id: String,
    pub override_being_count: Option<u16>,
    pub pack_spawn_radius: Option<u8>,
}

#[derive(Default, Debug, Clone)]
pub struct DungeonRoomPackSpawnConfig(pub HashMap<String, DungeonRoomPackSpawnSpec>);

impl DungeonRoomPackSpawnConfig {
    pub fn from_typed_args(
        args: &SgcArgsDict,
        command_registry: &SgcCommandRegistry,
        structure_id: &str,
    ) -> Self {
        let mut out = Self::default();
        let allowed_shapes = command_registry.allowed_room_spawn_shapes_for(structure_id);

        if let Some(room_spawn_map) = args.get_map("room_spawn") {
            for (shape_key, spec_value) in room_spawn_map {
                if let Some(allowed_shapes) = allowed_shapes
                    && !allowed_shapes.contains(shape_key)
                {
                    continue;
                }
                let Some(spec) = parse_room_spawn_spec(spec_value) else {
                    continue;
                };
                out.0.insert(shape_key.clone(), spec);
            }
        }

        for (key, value) in args.iter() {
            let Some((_, rest)) = key.split_once("room_spawn.") else {
                continue;
            };
            let Some((shape_key, field)) = rest.split_once('.') else {
                continue;
            };
            if shape_key.is_empty() || field.is_empty() {
                continue;
            }
            if let Some(allowed_shapes) = allowed_shapes
                && !allowed_shapes.contains(shape_key)
            {
                continue;
            }
            let entry = out.0.entry(shape_key.to_string()).or_insert_with(|| DungeonRoomPackSpawnSpec {
                source_template_id: String::new(),
                override_being_count: None,
                pack_spawn_radius: None,
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
                _ => {}
            }
        }

        out.0.retain(|_, spec| !spec.source_template_id.trim().is_empty());
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
    Some(DungeonRoomPackSpawnSpec {
        source_template_id,
        override_being_count,
        pack_spawn_radius,
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
        source_lookup.insert(str_id.clone(), entity);
    }
    for (entity, str_id) in bit_templates.iter() {
        source_lookup.insert(str_id.clone(), entity);
    }
}

pub fn queue_room_spawn_instance_message(
    room_shape_key: &str,
    anchor_gpos: GlobalTilePos,
    dim_ref: DimensionRef,
    room_spawn_config: &DungeonRoomPackSpawnConfig,
    source_lookup: &HashMap<StrId, Entity>,
    pending_messages: &mut Vec<InstantiateTemplPackEntity>,
) -> bool {
    let Some(room_spec) = room_spawn_config.0.get(room_shape_key) else {
        return false;
    };
    let source_id = StrId::trunc(room_spec.source_template_id.as_str());
    let Some(&source_ent) = source_lookup.get(&source_id) else {
        return false;
    };
    pending_messages.push(InstantiateTemplPackEntity::new(
        source_ent,
        room_spec.override_being_count,
        None,
        room_spec.pack_spawn_radius,
        dim_ref,
        [anchor_gpos],
    ));
    true
}
