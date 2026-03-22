use crate::being_components::{Being, Chasing};
use ::being_shared::*;
use bevy_northstar::{CardinalGrid, grid::GridSettingsBuilder, nav::Nav};
use ::tilemap_shared::{ChunkPos, GlobalTilePos, LoadedChunks, LoadChunksAround};
use bevy::{
    ecs::entity::{EntityHashMap, EntityHashSet},
    platform::collections::HashMap,
    prelude::*,
};
use param_sets::BlockingTileParamSet;

use super::being_nav_resources::AiNavGrids;
use super::being_nav_structs::AiNavGridCache;



pub fn sync_ai_nav_grids(
    time: Res<Time>,
    loaded_chunks: Res<LoadedChunks>,
    chunk_range: Res<LoadChunksAround>,
    mut param_set: BlockingTileParamSet,
    chasers_query: Query<
        (
            &GlobalTilePos,
            &::tilemap_shared::DimensionRef,
            Option<&ComputedBy>,
            &Chasing,
        ),
        With<Being>,
    >,
    beings_query: Query<(Entity, &GlobalTilePos, &::tilemap_shared::DimensionRef), With<Being>>,
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

    for (gpos, dim_ref, controlled_by, _to_chase) in chaser_iter {
        if let Some(controlled_by) = controlled_by {
            if controlled_by.human_dc_input {
                continue;
            }
        }
        needed_dims.insert(dim_ref.0);
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
        for (being_ent, gpos, dim_ref) in being_iter {
            if dim_ref.0 != dim {
                continue;
            }
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
