
use crate::being_messages::NavOrder;
use ::being_shared::*;
use bevy_northstar::{CardinalGrid, grid::GridSettingsBuilder, nav::Nav};
use ::tilemap_shared::{BlacklistedTags, ChunkPos, GlobalTilePos, LoadedChunks, LoadChunksAround};
use bevy::{
    ecs::entity::{EntityHashMap, EntityHashSet},
    platform::collections::HashMap,
    prelude::*,
};
use common::log_targets::BEING_SYSTEM;
use param_sets::BlockingTileParamSet;
use movement::movement_components::{InputMaxSpeed, InputSpeedThrottleMult};

use super::being_nav_resources::AiNavGrids;
use super::being_nav_structs::AiNavGridCache;

#[allow(unused_parens, )]
pub fn ensure_loaded_beings_have_nav_state(
    mut cmd: Commands,
    beings: Query<(Entity, Has<Wandering>, Has<Chasing>, Has<Fleeing>, ), (With<Being>, LoadedBeing, )>,
) {
    for (being_ent, has_wandering, has_chasing, has_fleeing, ) in beings.iter() {
        if has_wandering || has_chasing || has_fleeing {
            continue;
        }
        cmd.entity(being_ent).try_insert_if_new(Wandering);
    }
}

#[allow(unused_parens, )]
pub fn update_goto_from_chasing(
    mut writer: MessageWriter<NavOrder>,
    mut messages: Local<Vec<NavOrder>>,
    chasing_query: Query<(Entity, &::tilemap_shared::DimensionRef, &Chasing, ), (With<Being>, LocalAiControlled, )>,
    targets_query: Query<(Entity, &GlobalTilePos, &::tilemap_shared::DimensionRef), >,
) {
    for (being_ent, &being_dim, chasing, ) in chasing_query.iter() {
        let order = chasing
            .chase_target_pos(being_ent, being_dim, &targets_query)
            .map(|target_pos| {
                NavOrder::new(
                    being_ent,
                    200,
                    NavOrderSource::Chasing,
                    Some(GoTo::new(target_pos, chasing.stop_distance)),
                )
            })
            .unwrap_or_else(|| {
                NavOrder::new(
                    being_ent,
                    200,
                    NavOrderSource::Chasing,
                    None,
                )
            });
        messages.push(order);
    }
    writer.write_batch(messages.drain(..));
}

fn choose_flee_target_pos(
    blocking_tiles: &mut BlockingTileParamSet,
    being_ent: Entity,
    being_dim: ::tilemap_shared::DimensionRef,
    being_gpos: GlobalTilePos,
    flee_from_gpos: GlobalTilePos,
    avoid_tile_tags: &BlacklistedTags,
) -> Option<GlobalTilePos> {
    let away = being_gpos.0 - flee_from_gpos.0;
    let primary_step = if away == IVec2::ZERO {
        IVec2::X
    } else if away.x.abs() >= away.y.abs() {
        IVec2::new(away.x.signum(), 0)
    } else {
        IVec2::new(0, away.y.signum())
    };
    let lateral_step = IVec2::new(-primary_step.y, primary_step.x);
    for step in [primary_step, lateral_step, -lateral_step, -primary_step] {
        if step == IVec2::ZERO {
            continue;
        }
        let mut best = None;
        for dist in 1..=8 {
            let candidate = GlobalTilePos(being_gpos.0 + step * dist);
            if blocking_tiles.is_blocked_at(being_dim, candidate, being_ent) {
                break;
            }
            if !avoid_tile_tags.is_empty() && !blocking_tiles.allowed_at(
                being_dim,
                candidate,
                being_ent,
                &::tilemap_shared::WhitelistedSpawnTileTags::default(),
                &::tilemap_shared::BlacklistedSpawnTileTags(avoid_tile_tags.clone()),
            ) {
                continue;
            }
            best = Some(candidate);
        }
        if best.is_some() {
            return best;
        }
    }
    None
}

fn resolve_flee_wander_cfg(
    member_of: Option<&SquadMemberOf>,
    bit_ref: Option<&BitRef>,
    race_ref: Option<&RaceRef>,
    wander_cfg_query: &Query<&WanderConfig>,
) -> WanderConfig {
    if let Some(member_of) = member_of {
        if let Ok(cfg) = wander_cfg_query.get(member_of.0) {
            return cfg.clone();
        }
    }
    if let Some(bit_ref) = bit_ref {
        if let Ok(cfg) = wander_cfg_query.get(bit_ref.0) {
            return cfg.clone();
        }
    }
    if let Some(race_ref) = race_ref {
        if let Ok(cfg) = wander_cfg_query.get(race_ref.0) {
            return cfg.clone();
        }
    }
    WanderConfig::default()
}

#[allow(unused_parens, )]
pub fn update_goto_from_fleeing(
    mut writer: MessageWriter<NavOrder>,
    mut blocking_tiles: BlockingTileParamSet,
    flee_query: Query<(Entity, &::tilemap_shared::DimensionRef, &Fleeing, Option<&SquadMemberOf>, ), (With<Being>, LocalAiControlled, )>,
    flee_from_query: Query<(&::tilemap_shared::DimensionRef, ), (With<Being>, )>,
    wander_cfg_query: Query<&WanderConfig>,
    mut messages: Local<Vec<NavOrder>>,
) {
    for (being_ent, &being_dim, fleeing, member_of, ) in flee_query.iter() {
        let Ok(&being_gpos) = blocking_tiles.gpos_query.get(being_ent) else {
            continue;
        };
        let Ok((flee_from_dim, )) = flee_from_query.get(fleeing.flee_from()) else {
            messages.push(NavOrder::new(
                being_ent,
                255,
                NavOrderSource::Fleeing,
                None,
            ));
            continue;
        };
        let Ok(&flee_from_gpos) = blocking_tiles.gpos_query.get(fleeing.flee_from()) else {
            messages.push(NavOrder::new(
                being_ent,
                255,
                NavOrderSource::Fleeing,
                None,
            ));
            continue;
        };
        if *flee_from_dim != being_dim {
            messages.push(NavOrder::new(
                being_ent,
                255,
                NavOrderSource::Fleeing,
                None,
            ));
            continue;
        }
        let bit_ref = blocking_tiles.get_being_bit_ref(being_ent);
        let race_ref = blocking_tiles.get_being_race_ref(being_ent);
        let cfg = resolve_flee_wander_cfg(member_of, bit_ref, race_ref, &wander_cfg_query);
        let avoid_tile_tags = BlacklistedTags::new(&cfg.avoid_tile_tags);
        let Some(target_pos) = choose_flee_target_pos(
            &mut blocking_tiles,
            being_ent,
            being_dim,
            being_gpos,
            flee_from_gpos,
            &avoid_tile_tags,
        ) else {
            messages.push(NavOrder::new(
                being_ent,
                255,
                NavOrderSource::Fleeing,
                None,
            ));
            continue;
        };
        messages.push(NavOrder::new(
            being_ent,
            255,
            NavOrderSource::Fleeing,
            Some(GoTo::new(target_pos, 0.0)),
        ));
    }
    writer.write_batch(messages.drain(..));
}

#[allow(unused_parens, )]
pub fn apply_nav_orders(
    mut cmd: Commands,
    time: Res<Time>,
    mut reader: MessageReader<NavOrder>,
    mut selected_by_ent: Local<EntityHashMap<NavOrder>>,
    mut go_to_query: Query<Option<&mut GoTo>, >,
    mut input_speed_query: Query<(&mut InputSpeedThrottleMult, &mut InputMaxSpeed, ), (), >,
) {
    selected_by_ent.clear();
    for order in reader.read() {
        let should_replace = selected_by_ent
            .get(&order.being_ent)
            .map(|curr| {
                order.priority > curr.priority
                    || (order.priority == curr.priority
                        && order.source.tie_break_rank() > curr.source.tie_break_rank())
            })
            .unwrap_or(true);
        if should_replace {
            selected_by_ent.insert(order.being_ent, order.clone());
        }
    }
    let tick = time.elapsed().as_millis() as u32;
    for (being_ent, order) in selected_by_ent.drain() {
        if let Some(go_to) = order.go_to {
            if let Ok(Some(mut curr_go_to)) = go_to_query.get_mut(being_ent) {
                curr_go_to.pos = go_to.pos;
                curr_go_to.stop_distance = go_to.stop_distance;
                curr_go_to.source = Some(order.source);
                curr_go_to.updated_tick = tick;
            } else {
                cmd.entity(being_ent).try_insert(GoTo::with_source(
                    go_to.pos.expect("NavOrder::go_to should always contain a target"),
                    go_to.stop_distance,
                    order.source,
                    tick,
                ));
            }
        } else {
            if let Ok(Some(mut curr_go_to)) = go_to_query.get_mut(being_ent) {
                curr_go_to.pos = None;
                curr_go_to.source = None;
                curr_go_to.updated_tick = 0;
            }
        }
        let speed_throttle_mult = order.speed_throttle_mult.clamp(0.0, 1.0);
        let Ok((mut input_speed_throttle_mult, mut input_max_speed)) = input_speed_query.get_mut(being_ent) else {
            continue;
        };
        input_speed_throttle_mult.0 = speed_throttle_mult;
        input_max_speed.0 = order.max_speed.max(0.0);
        trace!(target: BEING_SYSTEM, "NavOrder winner {:?}: source={:?} priority={} goto={:?} throttle={:.2}", being_ent, order.source, order.priority, order.go_to, speed_throttle_mult);
    }
}

#[allow(unused_parens, )]
pub fn clear_nav_outputs_for_beings_without_nav_state(
    mut query: Query<(Entity, Has<Wandering>, Has<Chasing>, Has<Fleeing>, Option<&mut GoTo>, ), (With<Being>, LocalAiControlled, )>,
    mut input_speed_query: Query<(&mut InputSpeedThrottleMult, &mut InputMaxSpeed, ), (), >,
) {
    for (being_ent, has_wandering, has_chasing, has_fleeing, go_to, ) in query.iter_mut() {
        if has_wandering || has_chasing || has_fleeing {
            continue;
        }
        let Ok((mut input_speed_throttle_mult, mut input_max_speed)) = input_speed_query.get_mut(being_ent) else {
            continue;
        };
        if let Some(mut go_to) = go_to {
            go_to.pos = None;
            go_to.source = None;
            go_to.updated_tick = 0;
        }
        input_speed_throttle_mult.0 = 1.0;
        input_max_speed.0 = f32::MAX;
    }
}



pub fn sync_ai_nav_grids(
    time: Res<Time>,
    loaded_chunks: Res<LoadedChunks>,
    chunk_range: Res<LoadChunksAround>,
    mut param_set: BlockingTileParamSet,
    chasers_query: Query<
        (
            Entity,
            &::tilemap_shared::DimensionRef,
            Option<&ComputedBy>,
            Option<&GoTo>,
        ),
        With<Being>,
    >,
    beings_query: Query<(Entity, &::tilemap_shared::DimensionRef), With<Being>>,
    mut grids: ResMut<AiNavGrids>,
    mut needed_dims: Local<EntityHashSet>,
    mut dim_centers: Local<EntityHashMap<IVec2>>,
    mut dim_center_counts: Local<EntityHashMap<i32>>,
) {
    let chaser_iter = chasers_query.iter();
    let chaser_count = chaser_iter.size_hint().1.unwrap_or(chaser_iter.size_hint().0);
    needed_dims.clear();
    needed_dims.reserve(chaser_count);
    dim_centers.clear();
    dim_centers.reserve(chaser_count);
    dim_center_counts.clear();
    dim_center_counts.reserve(chaser_count);

    for (being_ent, dim_ref, controlled_by, goto, ) in chaser_iter {
        let Some(goto) = goto else {
            continue;
        };
        if goto.pos.is_none() {
            continue;
        };
        if let Some(controlled_by) = controlled_by {
            if controlled_by.human_dc_input {
                continue;
            }
        }
        needed_dims.insert(dim_ref.0);
        let Ok(&gpos) = param_set.gpos_query.get(being_ent) else {
            continue;
        };
        let pos = gpos.0;
        let center = dim_centers.entry(dim_ref.0).or_insert(IVec2::ZERO);
        *center += pos;
        *dim_center_counts.entry(dim_ref.0).or_insert(0) += 1;
    }

    grids.by_dim.retain(|dim, _| needed_dims.contains(dim));
    grids
        .center_by_dim
        .retain(|dim, _| needed_dims.contains(dim));

    let max_side = (((chunk_range.discovery_range as i32 * 2) - 1).max(1) as u32)
        * ChunkPos::CHUNK_SIZE.x.max(1);
    let should_rebuild = grids.rebuild_timer.tick(time.delta()).just_finished();

    for dim in needed_dims.iter().copied() {
        let mut min_tile: Option<IVec2> = None;
        let mut max_tile: Option<IVec2> = None;

        for (&(dim_ref, chunk_pos), _) in loaded_chunks.0.iter() {
            if dim_ref.0 != dim {
                continue;
            }
            let cmin = chunk_pos.to_tilepos().0;
            let cmax = cmin + ChunkPos::CHUNK_SIZE.as_ivec2() - IVec2::ONE;
            min_tile = Some(min_tile.map(|m| m.min(cmin)).unwrap_or(cmin));
            max_tile = Some(max_tile.map(|m| m.max(cmax)).unwrap_or(cmax));
        }

        let Some(mut min_tile) = min_tile else {
            continue;
        };
        let Some(max_tile) = max_tile else {
            continue;
        };

        let center = dim_centers
            .get(&dim)
            .zip(dim_center_counts.get(&dim))
            .map(|(sum, count)| *sum / count.max(&1))
            .unwrap_or((min_tile + max_tile) / 2);

        let mut width = (max_tile.x - min_tile.x + 1).max(3) as u32;
        let mut height = (max_tile.y - min_tile.y + 1).max(3) as u32;
        if width > max_side {
            let half = (max_side as i32) / 2;
            min_tile.x = center.x - half;
            width = max_side;
        }
        if height > max_side {
            let half = (max_side as i32) / 2;
            min_tile.y = center.y - half;
            height = max_side;
        }

        let center_changed = grids
            .center_by_dim
            .get(&dim)
            .map(|prev| (*prev - center).abs().max_element() >= ChunkPos::CHUNK_SIZE.x as i32)
            .unwrap_or(true);
        let needs_new_grid = !grids.by_dim.contains_key(&dim);
        let rebuild_grid = needs_new_grid || should_rebuild || center_changed;

        if rebuild_grid {
            let mut grid = CardinalGrid::new(
                &GridSettingsBuilder::new_2d(width, height)
                    .chunk_size(8)
                    .build(),
            );
            for y in 0..height {
                for x in 0..width {
                    let world = GlobalTilePos(min_tile + IVec2::new(x as i32, y as i32));
                    if param_set.is_blocked_at_tiles_only(
                        ::tilemap_shared::DimensionRef(dim),
                        world,
                        Entity::PLACEHOLDER,
                    ) {
                        grid.set_nav(UVec3::new(x, y, 0), Nav::Impassable);
                    }
                }
            }
            grid.build();
            grids.by_dim.insert(
                dim,
                AiNavGridCache {
                    min: min_tile,
                    grid,
                    occupied: HashMap::default(),
                },
            );
            grids.center_by_dim.insert(dim, center);
        }

        let Some(cache) = grids.by_dim.get_mut(&dim) else {
            continue;
        };
        cache.occupied.clear();
        let being_iter = beings_query.iter();
        cache.occupied.reserve(being_iter.size_hint().1.unwrap_or(being_iter.size_hint().0));
        for (being_ent, dim_ref) in being_iter {
            if dim_ref.0 != dim {
                continue;
            }
            let Ok(&gpos) = param_set.gpos_query.get(being_ent) else {
                continue;
            };
            let max_grid = cache.min
                + IVec2::new(
                    cache.grid.width() as i32 - 1,
                    cache.grid.height() as i32 - 1,
                );
            if gpos.0.x < cache.min.x
                || gpos.0.y < cache.min.y
                || gpos.0.x > max_grid.x
                || gpos.0.y > max_grid.y
            {
                continue;
            }
            let local = (gpos.0 - cache.min).as_uvec2();
            cache
                .occupied
                .insert(UVec3::new(local.x, local.y, 0), being_ent);
        }
    }
}
