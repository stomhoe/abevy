use bevy::{platform::collections::HashMap, prelude::*};
use common::{common_components::{HashId, Tag}, common_tag_components::TagSet};
use game_common::{game_common_components::ArgsDict, game_common_samplers::EntityWeightedSampler};
use ::tilemap_shared::{GlobalGenSettings, GlobalTilePos};
use crate::tile::{tile_components::DeleteOtherTilesInSamePos, tile_sampler_components::TileWeightedSampler};

fn parse_delete_other_tiles_tags(selector: &str) -> Vec<Tag> {
    let selector = selector.trim();
    if selector.is_empty() {
        return Vec::new();
    }
    let Some(selector) = selector
        .strip_prefix('[')
        .and_then(|selector| selector.strip_suffix(']'))
    else {
        return vec![Tag::trunc(selector)];
    };
    selector
        .split(',')
        .map(str::trim)
        .filter(|tag| !tag.is_empty())
        .map(Tag::trunc)
        .collect()
}

#[derive(Default)]
pub struct DeleteOtherTilesConfigMap {
    global: Option<DeleteOtherTilesInSamePos>,
    by_used_tile_tag: HashMap<Tag, DeleteOtherTilesInSamePos>,
    by_tile_id: HashMap<HashId, DeleteOtherTilesInSamePos>,
}
impl DeleteOtherTilesConfigMap {
    pub fn get(&self, tile_id: &HashId, used_tile_tags: Option<&TagSet>) -> Option<DeleteOtherTilesInSamePos> {
        self.get_for_tile(Some(tile_id), used_tile_tags)
    }

    pub fn get_for_tile(
        &self,
        tile_id: Option<&HashId>,
        used_tile_tags: Option<&TagSet>,
    ) -> Option<DeleteOtherTilesInSamePos> {
        let mut merged = self.global.clone().unwrap_or_default();
        let mut has_any = self.global.is_some();
        if let Some(used_tile_tags) = used_tile_tags {
            for (tag, spec) in &self.by_used_tile_tag {
                if used_tile_tags.contains(tag.clone()) {
                    merged.merge_from(spec);
                    has_any = true;
                }
            }
        }
        if let Some(tile_id) = tile_id {
            if let Some(specific) = self.by_tile_id.get(tile_id) {
                merged.merge_from(specific);
                has_any = true;
            }
        }
        has_any.then_some(merged)
    }
}

impl DeleteOtherTilesConfigMap {
    pub fn from_args(args: &ArgsDict) -> DeleteOtherTilesConfigMap {
        let mut out = DeleteOtherTilesConfigMap::default();
        for (key, values) in args.iter() {
            let Some(key) = key.as_str().strip_prefix("delete_other_tiles.") else {
                let Some(key) = key
                    .as_str()
                    .strip_prefix("delete_other_tiles_tag.")
                    .or_else(|| key.as_str().strip_prefix("delete_other_tiles_tag-"))
                else {
                    continue;
                };
                let Some((selector, field)) = key.rsplit_once('.') else {
                    continue;
                };
                let tags = parse_delete_other_tiles_tags(selector);
                if tags.is_empty() {
                    continue;
                }
                let mut parsed_spec = DeleteOtherTilesInSamePos::default();
                if !parsed_spec.apply_delete_other_tiles_field(field, values) {
                    continue;
                }
                for tag in tags {
                    let spec = out.by_used_tile_tag.entry(tag).or_default();
                    spec.merge_from(&parsed_spec);
                }
                continue;
            };
            let Some((tile_id, field)) = key.rsplit_once('.') else {
                continue;
            };
            if tile_id.trim().is_empty() {
                continue;
            }
            let mut parsed_spec = DeleteOtherTilesInSamePos::default();
            if !parsed_spec.apply_delete_other_tiles_field(field, values) {
                continue;
            }
            if tile_id == "*" {
                let spec = out.global.get_or_insert_with(DeleteOtherTilesInSamePos::default);
                spec.merge_from(&parsed_spec);
            } else {
                let spec = out.by_tile_id.entry(HashId::hash(tile_id)).or_default();
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tag_rules_still_apply_without_tile_hash() {
        let mut args = ArgsDict::default();
        args.insert(
            "delete_other_tiles.*.spared_tags".to_string(),
            vec!["boulder".to_string()],
        );
        args.insert(
            "delete_other_tiles_tag.boulder.spared_tags".to_string(),
            vec!["floor".to_string(), "dungeon_floor".to_string()],
        );

        let config = DeleteOtherTilesConfigMap::from_args(&args);
        let tags = TagSet::new(["boulder"]);
        let spec = config
            .get_for_tile(None, Some(&tags))
            .expect("expected tag-based rule to be built without a tile hash");

        assert!(spec.spared_tags.contains("boulder"));
        assert!(spec.spared_tags.contains("floor"));
        assert!(spec.spared_tags.contains("dungeon_floor"));
    }

    #[test]
    fn multi_tag_selector_applies_to_each_tag() {
        let mut args = ArgsDict::default();
        args.insert(
            "delete_other_tiles_tag-[dungeon_floor, floor].targeted_tags".to_string(),
            vec!["lava".to_string()],
        );

        let config = DeleteOtherTilesConfigMap::from_args(&args);
        let dungeon_floor_tags = TagSet::new(["dungeon_floor"]);
        let floor_tags = TagSet::new(["floor"]);

        let dungeon_floor_spec = config
            .get_for_tile(None, Some(&dungeon_floor_tags))
            .expect("expected rule for dungeon_floor selector");
        let floor_spec = config
            .get_for_tile(None, Some(&floor_tags))
            .expect("expected rule for floor selector");

        assert!(dungeon_floor_spec.targeted_tags.contains("lava"));
        assert!(floor_spec.targeted_tags.contains("lava"));
    }

    #[test]
    fn unknown_fields_do_not_create_delete_other_tiles_components() {
        let mut args = ArgsDict::default();
        args.insert(
            "delete_other_tiles_tag.dungeon_floor.targeted".to_string(),
            vec!["floor".to_string()],
        );

        let config = DeleteOtherTilesConfigMap::from_args(&args);
        let dungeon_floor_tags = TagSet::new(["dungeon_floor"]);

        assert!(config.get_for_tile(None, Some(&dungeon_floor_tags)).is_none());
    }
}
