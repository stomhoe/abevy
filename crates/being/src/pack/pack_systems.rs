use bevy::prelude::*;
use bevy::platform::collections::HashMap;
use game_common::game_common_components::Templ;
use tilemap_shared::{DimensionRef, GlobalTilePos, };
use ::being_shared::*;

#[allow(unused_parens, )]
pub fn despawn_empty_squads(
    mut cmd: Commands,
    query: Query<(), (Without<Templ>, Without<SquadMembers>, Without<faction_shared::Faction>, Without<PreventCleanup>)>,
    mut removed_squad_members: RemovedComponents<SquadMembers>,
    pack_query: Query<(Entity, &SquadMembers), (Without<Templ>, Without<faction_shared::Faction>, Without<PreventCleanup>)>,
) {
    for squad_ent in removed_squad_members.read() {
        if query.get(squad_ent).is_ok() {
            cmd.entity(squad_ent).try_despawn();
        }
    }

    for (pack_ent, members) in pack_query.iter() {
        if members.len() > 1 {
            continue;
        }
        cmd.entity(pack_ent).try_despawn();
    }
}

#[allow(unused_parens, )]
pub fn update_pack_center_pos(
    mut cmd: Commands,
    mut pack_query: Query<(Entity, &SquadMembers, Option<&MemberRanks>, Option<&GlobalCenterRankWeightMultiplier>, Option<&CenterWeightRankBasedMultiplier>, Option<&mut SquadAvgCenterPerDim>, ), (Without<Templ>, )>,
    member_pos_query: Query<(&DimensionRef, &GlobalTilePos, Option<&BitRef>, Option<&RaceRef>, ), (Without<Templ>, ),>,
    bit_map: Res<BeingInstTemplateEntityMap>,
    race_map: Res<RaceEntityMap>,
    mut centers: Local<HashMap<DimensionRef, (Vec2, f32)>>,
) {
    for (pack_ent, members, member_ranks, global_weight_multiplier, center_rank_multipliers, pack_center_pos, ) in pack_query.iter_mut() {
        centers.clear();
        for member_ent in members.iter() {
            let Ok((member_dim_ref, member_gpos, member_bit_ref, member_race_ref, )) = member_pos_query.get(member_ent) else {
                continue;
            };
            let member_rank = member_ranks
                .and_then(|member_ranks| member_ranks.0.get(&member_ent).copied())
                .unwrap_or(0.0);
            let member_bit_ent = member_bit_ref.and_then(|bit_ref| bit_map.0.get_cloned(bit_ref.0).ok());
            let member_race_ent = member_race_ref.and_then(|race_ref| race_map.0.get_cloned(race_ref.0).ok());
            let member_multiplier = member_bit_ref
                .and_then(|_| {
                    center_rank_multipliers
                        .and_then(|multipliers| member_bit_ent.and_then(|bit_ent| multipliers.0.get(&bit_ent).copied()))
                })
                .or_else(|| {
                    member_race_ref.and_then(|_| {
                        center_rank_multipliers
                            .and_then(|multipliers| member_race_ent.and_then(|race_ent| multipliers.0.get(&race_ent).copied()))
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

        let mut pack_center_pos_new: SquadAvgCenterPerDim = Default::default();
        for (&dim_ref, &(sum, weight_sum)) in centers.iter() {
            if weight_sum <= 0.0 {
                continue;
            }
            pack_center_pos_new
                .0
                .insert(dim_ref, GlobalTilePos(((sum / weight_sum).round().as_ivec2())));
        }
        if pack_center_pos_new.0.is_empty() {
            cmd.entity(pack_ent).try_remove::<SquadAvgCenterPerDim>();
            continue;
        }
        if let Some(mut pack_center_pos) = pack_center_pos {
            pack_center_pos.0 = pack_center_pos_new.0;
        } else {
            cmd.entity(pack_ent).try_insert(pack_center_pos_new);
        }
    }
}
