use bevy::ecs::entity::EntityHashMap;
use bevy::prelude::*;
use common::log_targets;
use game_common::game_common_components::{Dead, EntityZero, EntityZeroRef};
use item_shared::{Item, ItemsGeneratedOnDeath, ToDenyOnItemClone};
use modifier_shared::modifier_components::{CurrEffectiveValue, ModifierTarget};
use modifier_shared::modifier_item_types::StackLimit;
use param_sets::BlockingTileParamSet;
use sprite::prelude::ScsToBuild;
use std::collections::HashSet;
use tilemap_shared::{DimensionRef, GlobalTilePos, ItemsAtGpos};

pub fn spawn_items_on_death(
    mut cmd: Commands,
    mut items_at_gpos: Option<ResMut<ItemsAtGpos>>,
    blocking_tiles: BlockingTileParamSet,
    query: Query<
        (
            Has<Dead>,
            Option<&DimensionRef>,
            Option<&GlobalTransform>,
            Option<&GlobalTilePos>,
            Option<&ItemsGeneratedOnDeath>,
            Option<&EntityZeroRef>,
        ),
        (Without<EntityZero>, Added<Dead>),
    >,
    ezero_drop_query: Query<&ItemsGeneratedOnDeath, With<EntityZero>>,
    item_cfg_query: Query<&item_shared::ItemSpritesConfig, (With<Item>, With<EntityZero>)>,
    stack_limit_query: Query<(&ModifierTarget, &CurrEffectiveValue), With<StackLimit>>,
    mut to_drain: Local<Vec<Entity>>,
    mut occupied_nonstackable: Local<HashSet<GlobalTilePos>>,
) {
    let mut stack_limit_by_item = EntityHashMap::default();
    for (&ModifierTarget(target), &CurrEffectiveValue(limit)) in stack_limit_query.iter() {
        stack_limit_by_item.insert(target, limit.max(1.0));
    }
    let mut rng = rand::rng();
    for (is_dead, dim_ref, global_transform, tile_pos, generated_on_death, ezero_ref) in query.iter() {
        if !is_dead {
            continue;
        }
        let generated_on_death = generated_on_death
            .cloned()
            .or_else(|| ezero_ref.and_then(|ezero_ref| ezero_drop_query.get(ezero_ref.0).ok()).cloned());
        let Some(generated_on_death) = generated_on_death else {
            continue;
        };
        let Some(item_counts) = generated_on_death.sampler.sample_with_rng(&mut rng) else {
            info!(
                target: log_targets::ITEM_SYSTEM,
                "Entity death ({:?}) produced no sampled drops (tile_pos={:?}, dim_ref={:?})",
                ezero_ref.map(|e| e.0),
                tile_pos,
                dim_ref.map(|d| d.0),
            );
            continue;
        };
        info!(target: log_targets::ITEM_SYSTEM, "Entity death ({:?}) at {:?}: generating {} item types", ezero_ref.map(|e| e.0), tile_pos, item_counts.len());
        let drop_pos = global_transform
            .map(|transform| transform.translation().xy())
            .or_else(|| tile_pos.map(GlobalTilePos::to_pixelpos))
            .unwrap_or_default();
        let drop_z = global_transform
            .map(|transform| transform.translation().z)
            .unwrap_or_default();
        let drop_gpos = tile_pos
            .copied()
            .unwrap_or_else(|| GlobalTilePos::from(drop_pos));
        occupied_nonstackable.clear();
        let mut stackable_drop_gpos = drop_gpos;
        let count_multiplier = generated_on_death.count_multiplier.max(0.0);

        if let Some(&dim_ref) = dim_ref {
            'stackable_search: for dist in 0..i32::MAX {
                for dx in -dist..=dist {
                    let dy_abs = dist - dx.abs();
                    let cand_a = drop_gpos + GlobalTilePos::new(dx, dy_abs);
                    if !blocking_tiles.is_blocked_at_tiles_only_except_dead_despawning(&mut *to_drain, dim_ref, cand_a, Entity::PLACEHOLDER) {
                        stackable_drop_gpos = cand_a;
                        break 'stackable_search;
                    }
                    if dy_abs == 0 {
                        continue;
                    }
                    let cand_b = drop_gpos + GlobalTilePos::new(dx, -dy_abs);
                    if !blocking_tiles.is_blocked_at_tiles_only_except_dead_despawning(&mut *to_drain, dim_ref, cand_b, Entity::PLACEHOLDER) {
                        stackable_drop_gpos = cand_b;
                        break 'stackable_search;
                    }
                }
            }
        }

        for (item_ezero, base_count) in item_counts {
            let dropped_sprite_cfg = item_cfg_query
                .get(item_ezero)
                .ok()
                .and_then(|cfg| {
                    if cfg.dropped_sprite_cfg.0 != Entity::PLACEHOLDER {
                        return Some(cfg.dropped_sprite_cfg.0);
                    }
                    if cfg.icon_sprite_cfg.0 != Entity::PLACEHOLDER {
                        info!(
                            target: log_targets::ITEM_SYSTEM,
                            " - {:?} has no dropped_sprite_cfg; falling back to icon_sprite_cfg {:?}",
                            item_ezero,
                            cfg.icon_sprite_cfg.0,
                        );
                        return Some(cfg.icon_sprite_cfg.0);
                    }
                    None
                });
            let stack_limit = stack_limit_by_item.get(&item_ezero).copied().unwrap_or(1.0);
            let is_nonstackable = stack_limit <= 1.0;
            let scaled_count = base_count as f32 * count_multiplier;
            let drop_count = scaled_count.round() as u32;
            if drop_count == 0 {
                info!(
                    target: log_targets::ITEM_SYSTEM,
                    " - Skipping {:?}: base_count={}, multiplier={}, rounded_drop_count=0",
                    item_ezero,
                    base_count,
                    count_multiplier,
                );
                continue;
            }
            if drop_count > 0 {
                info!(target: log_targets::ITEM_SYSTEM, " - Spawning {} x {:?} at {:?}", drop_count, item_ezero, drop_gpos);
            }
            for _ in 0..drop_count {
                let spawn_gpos = if is_nonstackable {
                    let mut next = drop_gpos;
                    if let Some(&dim_ref) = dim_ref {
                        'nonstack_search: for dist in 0..i32::MAX {
                            for dx in -dist..=dist {
                                let dy_abs = dist - dx.abs();
                                let cand_a = drop_gpos + GlobalTilePos::new(dx, dy_abs);
                                if !occupied_nonstackable.contains(&cand_a)
                                    && !blocking_tiles.is_blocked_at_tiles_only_except_dead_despawning(&mut *to_drain, dim_ref, cand_a, Entity::PLACEHOLDER)
                                {
                                    next = cand_a;
                                    break 'nonstack_search;
                                }
                                if dy_abs == 0 {
                                    continue;
                                }
                                let cand_b = drop_gpos + GlobalTilePos::new(dx, -dy_abs);
                                if !occupied_nonstackable.contains(&cand_b)
                                    && !blocking_tiles.is_blocked_at_tiles_only_except_dead_despawning(&mut *to_drain, dim_ref, cand_b, Entity::PLACEHOLDER)
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
                } else {
                    stackable_drop_gpos
                };

                let item_instance = cmd
                    .entity(item_ezero)
                    .clone_and_spawn_with_opt_out(|builder| {
                        builder.deny::<ToDenyOnItemClone>();
                    })
                    .id();
                let mut item_cmd = cmd.entity(item_instance);
                info!(
                    target: log_targets::ITEM_SYSTEM,
                    "   -> cloned item instance {:?} from ezero {:?}, spawn_gpos={:?}, drop_z={}",
                    item_instance,
                    item_ezero,
                    spawn_gpos,
                    drop_z,
                );
                item_cmd.insert((
                    Item,
                    EntityZeroRef(item_ezero),
                    Transform::from_translation(spawn_gpos.to_translation(drop_z)),
                    GlobalTransform::default(),
                    spawn_gpos,
                ));
                if let Some(cfg_ent) = dropped_sprite_cfg {
                    let mut scs_to_build = ScsToBuild::with_capacity(1);
                    scs_to_build.0.insert(cfg_ent);
                    item_cmd.insert(scs_to_build);
                    info!(
                        target: log_targets::ITEM_SYSTEM,
                        "   -> inserted ScsToBuild for {:?} with dropped cfg {:?}",
                        item_instance,
                        cfg_ent,
                    );
                } else {
                    warn!(
                        target: log_targets::ITEM_SYSTEM,
                        "   -> no dropped sprite cfg for ezero {:?}; item {:?} may be invisible",
                        item_ezero,
                        item_instance,
                    );
                }
                let Some(&dim_ref) = dim_ref else {
                    warn!(
                        target: log_targets::ITEM_SYSTEM,
                        "   -> missing DimensionRef on dead entity; item {:?} cannot be attached to a dimension or ItemsAtGpos",
                        item_instance,
                    );
                    continue;
                };
                item_cmd.insert((dim_ref, ChildOf(dim_ref.0)));
                info!(
                    target: log_targets::ITEM_SYSTEM,
                    "   -> inserted DimensionRef {:?} and ChildOf({:?}) for {:?}",
                    dim_ref.0,
                    dim_ref.0,
                    item_instance,
                );
                let Some(items_at_gpos) = items_at_gpos.as_mut() else {
                    warn!(
                        target: log_targets::ITEM_SYSTEM,
                        "   -> ItemsAtGpos resource missing; spawned item {:?} not tracked in map",
                        item_instance,
                    );
                    continue;
                };
                items_at_gpos.insert_item(dim_ref, spawn_gpos, item_instance);
                info!(
                    target: log_targets::ITEM_SYSTEM,
                    "   -> ItemsAtGpos insert dim={:?} gpos={:?} item={:?}",
                    dim_ref.0,
                    spawn_gpos,
                    item_instance,
                );
            }
        }
    }
}
