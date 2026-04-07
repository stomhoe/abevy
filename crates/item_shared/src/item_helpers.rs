use bevy::prelude::*;
use common::log_targets;
use common::common_components::HashId;
use game_common::game_common_components::{Templ, TemplEntiRef};
use sprite_shared::ScsToBuild;
use tilemap_shared::DimensionRef;

use crate::Item;

pub fn clone_item_from_templ(cmd: &mut Commands, templ_ref: TemplEntiRef, dimension_ref: DimensionRef) -> Entity {
    let item_instance = cmd
        .entity(templ_ref.0)
        .clone_and_spawn_with_opt_out(|builder| {
            builder.deny::<crate::ToDenyOnItemClone>();
        })
        .id();
    cmd.entity(item_instance)
        .insert((Item, TemplEntiRef(templ_ref.0), dimension_ref));
    item_instance
}

pub fn dropped_scs_to_build(
    item_cfg_query: &Query<&crate::ItemSpritesConfig, (With<Item>, With<Templ>)>,
    sprite_cfg_hash_query: &Query<&HashId, (With<::sprite_shared::SpriteConfig>, common::AnyDisabling)>,
    templ_ref: TemplEntiRef,
) -> Option<ScsToBuild> {
    let dropped_sprite_cfg = item_cfg_query
        .get(templ_ref.0)
        .ok()
        .and_then(|cfg| {
            if cfg.dropped_sprite_cfg.0 != Entity::PLACEHOLDER {
                return Some(cfg.dropped_sprite_cfg.0);
            }
            if cfg.icon_sprite_cfg.0 != Entity::PLACEHOLDER {
                return Some(cfg.icon_sprite_cfg.0);
            }
            None
        });
    let Some(cfg_ent) = dropped_sprite_cfg else {
        warn!(
            target: log_targets::ITEM_SYSTEM,
            "No dropped sprite cfg for item templ {:?}",
            templ_ref.0,
        );
        return None;
    };
    let Ok(&cfg_hash_id) = sprite_cfg_hash_query.get(cfg_ent) else {
        warn!(
            target: log_targets::ITEM_SYSTEM,
            "No HashId for dropped sprite cfg {:?} when building item templ {:?}",
            cfg_ent,
            templ_ref.0,
        );
        return None;
    };
    let mut scs_to_build = ScsToBuild::with_capacity(1);
    scs_to_build.0.insert(cfg_hash_id);
    Some(scs_to_build)
}
