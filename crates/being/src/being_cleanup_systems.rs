use std::collections::HashMap;

use bevy::prelude::*;
use common::log_targets::BEING_SYSTEM;
use faction_shared::BelongsToAPlayerFaction;
use tilemap_shared::GlobalTilePos;
use tilemap_shared::DimensionRef;

use ::being_shared::*;

pub const MAX_LOADED_BEINGS: usize = 30;

#[allow(unused_parens, )]
pub fn cull_loaded_beings_far_from_humans(
    mut cmd: Commands,
    non_hc_beings_query: Query<(Entity, &GlobalTilePos, &DimensionRef, ), (With<Being>, Without<Unloaded>, Without<BelongsToAPlayerFaction>, Without<HumanControlled>),>,
    hc_beings_query: Query<(&GlobalTilePos, &DimensionRef, ), (With<Being>, With<HumanControlled>, Without<Unloaded>),>,
    mut human_positions_by_dim: Local<HashMap<DimensionRef, Vec<GlobalTilePos>>>,
) {
    let non_hc_iter = non_hc_beings_query.iter();
    let non_hc_count = non_hc_iter.size_hint().1.unwrap_or(non_hc_iter.size_hint().0);
    let hc_iter = hc_beings_query.iter();
    let hc_count = hc_iter.size_hint().1.unwrap_or(hc_iter.size_hint().0);
    let loaded_count = non_hc_count + hc_count;
    if loaded_count <= MAX_LOADED_BEINGS {
        return;
    }

    human_positions_by_dim.clear();
    for (human_gpos, human_dim, ) in hc_beings_query.iter() {
        human_positions_by_dim.entry(*human_dim).or_default().push(*human_gpos);
    }

    let mut farthest_non_hc: Option<(Entity, f32, GlobalTilePos, DimensionRef)> = None;
    for (being_ent, being_gpos, being_dim, ) in non_hc_beings_query.iter() {
        let Some(human_positions) = human_positions_by_dim.get(being_dim) else {
            continue;
        };
        let Ok(human_count_u32) = u32::try_from(human_positions.len()) else {
            continue;
        };
        if human_count_u32 == 0 {
            continue;
        }
        let avg_distance = human_positions
            .iter()
            .map(|human_gpos| being_gpos.taxicab_tile_distance(*human_gpos))
            .sum::<f32>()
            / human_count_u32 as f32;
        if farthest_non_hc
            .as_ref()
            .map(|(_, farthest_avg, _, _)| avg_distance > *farthest_avg)
            .unwrap_or(true)
        {
            farthest_non_hc = Some((being_ent, avg_distance, *being_gpos, *being_dim));
        }
    }

    let Some((being_ent, avg_distance, being_gpos, being_dim)) = farthest_non_hc else {
        return;
    };
    debug!(target: BEING_SYSTEM, "Despawning loaded non-human being {:?} at {:?} in {:?}; loaded_count={} exceeded threshold={} and avg_distance_to_humans={:.2}", being_ent, being_gpos, being_dim, loaded_count, MAX_LOADED_BEINGS, avg_distance);
    cmd.entity(being_ent).try_despawn();
}
