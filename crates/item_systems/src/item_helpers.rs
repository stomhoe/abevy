use bevy::prelude::*;
use common::log_targets;
use game_common::game_common_components::{EntityZero, EntityZeroRef};
use item_shared::Item;
use param_sets::BlockingTileParamSet;
use sprite_shared::prelude::{AcZ, ScsToBuild};
use std::collections::HashSet;
use tilemap_shared::{DimensionRef, GlobalTilePos, TileGatheringParamSet};

pub fn dropped_scs_to_build(
    item_cfg_query: &Query<&item_shared::ItemSpritesConfig, (With<Item>, With<EntityZero>)>,
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

pub fn materialize_item_on_ground(
    cmd: &mut Commands,
    item_cfg_query: &Query<&item_shared::ItemSpritesConfig, (With<Item>, With<EntityZero>)>,
    tile_gathering: &TileGatheringParamSet,
    ac_z_query: &Query<&AcZ>,
    to_drain: &mut Vec<Entity>,
    item_ent: Entity,
    item_ezero_ref: EntityZeroRef,
    dest: (DimensionRef, GlobalTilePos),
) {
    to_drain.clear();
    tile_gathering.gather_tiles_at(to_drain, dest.0, dest.1);
    let z = to_drain
        .iter()
        .filter_map(|&ent| {
            let Ok(&AcZ(z)) = ac_z_query.get(ent) else {
                return None;
            };
            Some(z)
        })
        .max_by(f32::total_cmp)
        .unwrap_or_default()
        + 0.001;
    let mut item_cmd = cmd.entity(item_ent);
    item_cmd.insert((
        Transform::from_translation(dest.1.to_translation(z)),
        dest.0,
        GlobalTransform::default(),
        dest.1,
        ChildOf(dest.0.0),
    ));
    let Some(scs_to_build) = dropped_scs_to_build(item_cfg_query, item_ezero_ref) else {
        return;
    };
    item_cmd.insert(scs_to_build);
}

pub fn find_stackable_drop_gpos(
    blocking_tiles: &BlockingTileParamSet,
    to_drain: &mut Vec<Entity>,
    dim_ref: Option<DimensionRef>,
    drop_gpos: GlobalTilePos,
) -> GlobalTilePos {
    let Some(dim_ref) = dim_ref else {
        return drop_gpos;
    };
    for dist in 0..i32::MAX {
        for dx in -dist..=dist {
            let dy_abs = dist - dx.abs();
            let cand_a = drop_gpos + GlobalTilePos::new(dx, dy_abs);
            if !blocking_tiles.is_blocked_at_tiles_only(to_drain, dim_ref, cand_a, Entity::PLACEHOLDER) {
                return cand_a;
            }
            if dy_abs == 0 {
                continue;
            }
            let cand_b = drop_gpos + GlobalTilePos::new(dx, -dy_abs);
            if !blocking_tiles.is_blocked_at_tiles_only(to_drain, dim_ref, cand_b, Entity::PLACEHOLDER) {
                return cand_b;
            }
        }
    }
    drop_gpos
}

pub fn find_nonstackable_drop_gpos(
    blocking_tiles: &BlockingTileParamSet,
    to_drain: &mut Vec<Entity>,
    occupied_nonstackable: &mut HashSet<GlobalTilePos>,
    dim_ref: Option<DimensionRef>,
    drop_gpos: GlobalTilePos,
) -> GlobalTilePos {
    let mut next = drop_gpos;
    if let Some(dim_ref) = dim_ref {
        'nonstack_search: for dist in 0..i32::MAX {
            for dx in -dist..=dist {
                let dy_abs = dist - dx.abs();
                let cand_a = drop_gpos + GlobalTilePos::new(dx, dy_abs);
                if !occupied_nonstackable.contains(&cand_a)
                    && !blocking_tiles.is_blocked_at_tiles_only(to_drain, dim_ref, cand_a, Entity::PLACEHOLDER)
                {
                    next = cand_a;
                    break 'nonstack_search;
                }
                if dy_abs == 0 {
                    continue;
                }
                let cand_b = drop_gpos + GlobalTilePos::new(dx, -dy_abs);
                if !occupied_nonstackable.contains(&cand_b)
                    && !blocking_tiles.is_blocked_at_tiles_only(to_drain, dim_ref, cand_b, Entity::PLACEHOLDER)
                {
                    next = cand_b;
                    break 'nonstack_search;
                }
            }
        }
    } else {
        for dist in 0..i32::MAX {
            for dx in -dist..=dist {
                let dy_abs = dist - dx.abs();
                let cand_a = drop_gpos + GlobalTilePos::new(dx, dy_abs);
                if !occupied_nonstackable.contains(&cand_a) {
                    next = cand_a;
                    break;
                }
                if dy_abs == 0 {
                    continue;
                }
                let cand_b = drop_gpos + GlobalTilePos::new(dx, -dy_abs);
                if !occupied_nonstackable.contains(&cand_b) {
                    next = cand_b;
                    break;
                }
            }
            if !occupied_nonstackable.contains(&next) {
                break;
            }
        }
    }
    occupied_nonstackable.insert(next);
    next
}
