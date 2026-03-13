use bevy::prelude::*;
use common::log_targets;
use game_common::game_common_components::{EntityZero, EntityZeroRef};
use sprite_shared::prelude::ScsToBuild;
use tilemap_shared::DimensionRef;

use crate::Item;

pub fn clone_item_from_ezero(cmd: &mut Commands, ezero_ref: EntityZeroRef, dimension_ref: DimensionRef) -> Entity {
    let item_instance = cmd
        .entity(ezero_ref.0)
        .clone_and_spawn_with_opt_out(|builder| {
            builder.deny::<crate::ToDenyOnItemClone>();
        })
        .id();
    cmd.entity(item_instance)
        .insert((Item, EntityZeroRef(ezero_ref.0), dimension_ref));
    item_instance
}

pub fn dropped_scs_to_build(
    item_cfg_query: &Query<&crate::ItemSpritesConfig, (With<Item>, With<EntityZero>)>,
    ezero_ref: EntityZeroRef,
) -> Option<ScsToBuild> {
    let dropped_sprite_cfg = item_cfg_query
        .get(ezero_ref.0)
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
            "No dropped sprite cfg for item ezero {:?}",
            ezero_ref.0,
        );
        return None;
    };
    let mut scs_to_build = ScsToBuild::with_capacity(1);
    scs_to_build.0.insert(cfg_ent);
    Some(scs_to_build)
}
