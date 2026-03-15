use bevy::{platform::collections::{HashMap, HashSet}, prelude::*};
use common::common_components::HashId;
use common::log_targets::DUNGEONING_SYSTEM;
use game_common::{game_common_components::ArgsDict, game_common_samplers::EntityWeightedSampler};
use ::tilemap_shared::{GlobalGenSettings, GlobalTilePos};
use crate::tile::{tile_components::DeleteOtherTilesInSamePos, tile_sampler_components::TileWeightedSampler};

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
    blocked_gpos: &mut HashSet<GlobalTilePos>,
    anchor_gpos: GlobalTilePos,
    size: UVec2,
) {
    for y in anchor_gpos.0.y..(anchor_gpos.0.y + size.y as i32) {
        for x in anchor_gpos.0.x..(anchor_gpos.0.x + size.x as i32) {
            blocked_gpos.insert(GlobalTilePos::new(x, y));
        }
    }
}
impl DeleteOtherTilesConfigMap {
    pub fn get(&self, placeholder_id: &str) -> Option<DeleteOtherTilesInSamePos> {
        let mut merged = self.global.clone().unwrap_or_default();
        let mut has_any = self.global.is_some();
        let key = placeholder_id.trim();
        let Some(spec) = self.by_placeholder_id.get(key) else {
            if !has_any {
                debug!(target: DUNGEONING_SYSTEM, "No delete rule matched for placeholder {:?}", key);
            }
            return has_any.then_some(merged);
        };
        debug!(target: DUNGEONING_SYSTEM, "Matched delete_other_tiles placeholder {:?}: {:?}", key, spec);
        merged.merge_from(spec);
        has_any = true;
        if !has_any {
            debug!(target: DUNGEONING_SYSTEM, "No delete_other_tiles rule matched for placeholder {:?}", key);
        }
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
