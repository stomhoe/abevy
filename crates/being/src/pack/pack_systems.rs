use bevy::prelude::*;
use bevy::platform::collections::HashMap;
use game_common::game_common_components::EntityZero;
use tilemap_shared::{DimensionRef, GlobalTilePos, };
use ::being_shared::*;

use crate::being_inst_template::being_inst_template_resources::BitRef;
use crate::race::race_resources::RaceRef;
use crate::pack::pack_components::{Pack, PackCenterPos, CenterRankMultipliers, GlobalCenterRankWeightMultiplier, };

#[allow(unused_parens, )]
pub fn cleanup_empty_packs(
    mut cmd: Commands,
    pack_query: Query<(Entity,), (With<Pack>, Without<EntityZero>, Without<SquadMembers>, )>,
) {
    for (pack_ent, ) in pack_query.iter() {
        cmd.entity(pack_ent).try_despawn();
    }
}

#[allow(unused_parens, )]
pub fn update_pack_center_pos(
    mut cmd: Commands,
    pack_query: Query<(Entity, &SquadMembers, &MemberRanks, Option<&GlobalCenterRankWeightMultiplier>, Option<&CenterRankMultipliers>, ), (Without<EntityZero>, )>,
    member_pos_query: Query<(&DimensionRef, &GlobalTilePos, Option<&BitRef>, Option<&RaceRef>, ), (Without<EntityZero>, ),>,
) {
    for (pack_ent, members, member_ranks, global_weight_multiplier, center_rank_multipliers, ) in pack_query.iter() {
        let mut centers: HashMap<DimensionRef, (Vec2, f32)> = HashMap::default();
        for member_ent in members.iter() {
            let Ok((member_dim_ref, member_gpos, member_bit_ref, member_race_ref, )) = member_pos_query.get(member_ent) else {
                continue;
            };
            let Some(&member_rank) = member_ranks.0.get(&member_ent) else {
                continue;
            };
            let member_multiplier = member_bit_ref
                .and_then(|bit_ref| {
                    center_rank_multipliers
                        .and_then(|multipliers| multipliers.0.get(&bit_ref.0).copied())
                })
                .or_else(|| {
                    member_race_ref.and_then(|race_ref| {
                        center_rank_multipliers
                            .and_then(|multipliers| multipliers.0.get(&race_ref.0).copied())
                    })
                })
                .unwrap_or(1.0)
                .max(0.0);
            let global_multiplier = global_weight_multiplier.map(|m| m.0).unwrap_or(1.0).max(0.0);
            let rank_weight = (1.0 + member_rank * member_multiplier * global_multiplier).max(0.0);
            if rank_weight <= 0.0 {
                continue;
            }
            let entry = centers.entry(*member_dim_ref).or_default();
            entry.0 += member_gpos.0.as_vec2() * rank_weight;
            entry.1 += rank_weight;
        }
        if centers.is_empty() {
            cmd.entity(pack_ent).try_remove::<PackCenterPos>();
            continue;
        }
        let mut pack_centers = HashMap::default();
        for (dim_ref, (sum, weight_sum)) in centers {
            if weight_sum <= 0.0 {
                continue;
            }
            pack_centers.insert(dim_ref, GlobalTilePos((sum / weight_sum).round().as_ivec2()));
        }
        if pack_centers.is_empty() {
            cmd.entity(pack_ent).try_remove::<PackCenterPos>();
            continue;
        }
        cmd.entity(pack_ent).try_insert(PackCenterPos(pack_centers));
    }
}
