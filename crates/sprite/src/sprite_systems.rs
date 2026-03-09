use std::collections::HashSet;

#[allow(unused_imports)] use bevy::prelude::*;
use bevy::ecs::entity::EntityHashSet;
use bevy_ecs_tilemap::{DrawTilemap, anchor::TilemapAnchor};
#[allow(unused_imports)] use bevy_replicon::prelude::*;
use bevy::ecs::entity_disabling::Disabled;
use common::{common_components::Prefix, log_targets::Z_SORT_SYSTEM, prelude::StrId};
use game_common::game_common_components::{EntityZero, EntityZeroRef, };
use tilemap_shared::{Dimension, GlobalTilePos};
use ::tilemap_shared::directions::*;

use ::sprite_shared::{sprite_scale_offset::*, *};

#[allow(unused_imports, )]
use crate::sprite_components::*;


#[derive(Message, Debug, Clone, Hash, PartialEq, Eq)]
pub struct SpriteChanged(pub Entity);

fn effective_acz(
    maybe_z_index: Option<&AcZ>,
    ezero_ref: Option<&EntityZeroRef>,
    ezero_query: &Query<(&AcZ, Option<&YSortOrigin>)>,
) -> f32 {
    if let Some(ezero_ref) = ezero_ref
        && let Ok((ezero_z_index, _)) = ezero_query.get(ezero_ref.0)
    {
        return ezero_z_index.used_float();
    }
    maybe_z_index.cloned().unwrap_or_default().used_float()
}

fn dropped_item_override_z(
    ent: Entity,
    global_transform: &GlobalTransform,
    child_of: &ChildOf,
    base_holder_query: &Query<(&Prefix, Has<GlobalTilePos>, &ChildOf), Without<EntityZero>>,
    dimension_query: &Query<(), With<Dimension>>,
    z_candidates_query: &Query<(Entity, &GlobalTransform, Option<&AcZ>, Option<&EntityZeroRef>), Or<(With<Sprite>, With<TilemapAnchor>)>>,
    ezero_query: &Query<(&AcZ, Option<&YSortOrigin>)>,
) -> Option<f32> {
    let Ok((prefix, has_gpos, base_child_of)) = base_holder_query.get(child_of.parent()) else {
        return None;
    };
    if !prefix.eq_str("Item") || !has_gpos || dimension_query.get(base_child_of.parent()).is_err() {
        return None;
    }
    let gpos = GlobalTilePos::from(global_transform.translation().xy());
    let mut max_acz: f32 = 0.0;
    for (other_ent, other_transform, other_z_index, other_ezero_ref) in z_candidates_query.iter() {
        if other_ent == ent || GlobalTilePos::from(other_transform.translation().xy()) != gpos {
            continue;
        }
        max_acz = max_acz.max(effective_acz(other_z_index, other_ezero_ref, ezero_query));
    }
    Some(max_acz + AcZ::Z_MULTIPLIER * 0.1)
}

#[allow(unused_parens)]
pub fn sprite_change_detection(
    sprite_query: Query<(Entity), (Or<(Changed<Scale2D>, Changed<ScaleLookUpDown>, Changed<ScaleSideways>, Changed<EntityZeroRef> , Changed<Offset2D>, Added<Sprite>)>)>,
    baseholder_query: Query<(&HeldSprites), (Or<(Changed<CardinalDirection>, Changed<HeldSprites>, Changed<HeldSprites>)>)>,
    mut writer: MessageWriter<SpriteChanged>,
    mut changed: Local<HashSet<SpriteChanged>>,
)
{
    for sprite_ent in sprite_query.iter() {
        changed.insert(SpriteChanged(sprite_ent));
    }
    for held_sprites in baseholder_query.iter() {
        for &sprite_ent in held_sprites.entities() {
            changed.insert(SpriteChanged(sprite_ent));
        }
    }
    writer.write_batch(changed.drain());
}


#[allow(unused_parens)]
pub fn disable_children_sprites_of_disabled(mut cmd: Commands,
    ezero_bases: Query<(&HeldSprites),(With<EntityZero>, Added<Disabled>)>,
    non_ezero_bases: Query<(&HeldSprites),(Without<EntityZero>,)>,
    mut removed: RemovedComponents<Disabled>,
) {
    let mut disableds = Vec::new();
    for (held_sprites) in ezero_bases.iter() {
        for &sprite_ent in held_sprites.entities() {
            disableds.push((sprite_ent, Disabled));
            //trace!(target: "sprite_systems", "Disabled sprite entity {:?} as its base entity was disabled", sprite_ent);
        }
    }
    for ent in removed.read() {
        if let Ok((held_sprites)) = non_ezero_bases.get(ent) {
            for &sprite_ent in held_sprites.entities() {
                cmd.entity(sprite_ent).try_remove::<Disabled>();
                //trace!(target: "sprite_systems","Re-enabled sprite entity {:?} as its base entity {:?} was re-enabled", sprite_ent, ent);
            }
        }
    }
    cmd.try_insert_batch(disableds);
}
#[allow(unused_parens, )]
pub fn z_sort_system(
    changed_query: Query<Entity,
        (Or<(Changed<EntityZeroRef>, Changed<GlobalTransform>, Changed<YSortOrigin>, Changed<AcZ>, Changed<ChildOf>,)>,
        Or<(With<Sprite>, With<TilemapAnchor>, )>)>,
    mut process_query: Query<(Entity, &mut Transform, &GlobalTransform, Option<&YSortOrigin>,
        Option<&AcZ>, Option<&EntityZeroRef>, Has<TilemapAnchor>, &ChildOf, ),>,

    parent_sprite_query: Query<Has<Sprite>, (common::AnyDisabling,)>,
    base_holder_query: Query<(&Prefix, Has<GlobalTilePos>, &ChildOf), Without<EntityZero>>,
    dimension_query: Query<(), With<Dimension>>,
    z_candidates_query: Query<(Entity, &GlobalTransform, Option<&AcZ>, Option<&EntityZeroRef>), Or<(With<Sprite>, With<TilemapAnchor>)>>,

    ezero_query: Query<(&AcZ, Option<&YSortOrigin>), ()>,

    strid_query: Query<&StrId, ()>,
    gpos_query: Query<&GlobalTilePos, ()>,

    mut mw_draw_tmap: MessageWriter<DrawTilemap>,
    mut draw_tmaps: Local<Vec<DrawTilemap>>,
    mut ents_to_process: Local<EntityHashSet>,
) {
    draw_tmaps.clear();
    ents_to_process.clear();
    for ent in changed_query.iter() {
        ents_to_process.insert(ent);
    }
    let mut iter = process_query.iter_many_mut(ents_to_process.iter().copied());
    while let Some((ent, mut transform, global_transform, ysort_origin, maybe_z_index, ezero_ref, is_tilemap, child_of)) = iter.fetch_next() {
        let Ok(has_parent_sprite) = parent_sprite_query.get(child_of.parent()) else {
            continue;
        };

        let (base_z, maybe_ysort_origin) = if let Some(override_z) = dropped_item_override_z(
            ent,
            global_transform,
            child_of,
            &base_holder_query,
            &dimension_query,
            &z_candidates_query,
            &ezero_query,
        ) {
            (override_z, None)
        } else if let Some(ezero_ref) = ezero_ref
            && let Ok((ezero_z_index, ezero_ysort_origin)) = ezero_query.get(ezero_ref.0)
        {
            (ezero_z_index.used_float(), ezero_ysort_origin.copied().or(ysort_origin.copied()))
        } else {
            (maybe_z_index.cloned().unwrap_or_default().used_float(), ysort_origin.copied())
        };

        let y = global_transform.translation().y;
        let origin_y = maybe_ysort_origin.unwrap_or_default().0;
        if !base_z.is_finite() || !y.is_finite() || !origin_y.is_finite() {
            continue;
        }

        let y_pos = y - origin_y;
        const TILE_FRAC_SENS: f32 = 0.3;
        let y_pos_tiles = TILE_FRAC_SENS * y_pos / GlobalTilePos::TILE_SIZE_PXS.y as f32;
        let use_y_sort = (maybe_ysort_origin.is_some() && !has_parent_sprite) as i32 as f32;
        let sigmoid = 1.0f32 / (1.0f32 + 2.0f32.powf(-0.01 * y_pos_tiles));
        let signed_sigmoid = (0.5 - sigmoid) * 2.0;
        let target_z = base_z + use_y_sort * signed_sigmoid * (AcZ::Z_MULTIPLIER * 0.49);
        if !target_z.is_finite() {
            continue;
        }

        let strid = strid_query.get(ent).ok()
            .or_else(|| ezero_ref.and_then(|r| strid_query.get(r.0).ok()))
            .map(|s| s.as_str()).unwrap_or("");
        let gpos = gpos_query.get(ent).ok()
            .or_else(|| ezero_ref.and_then(|r| gpos_query.get(r.0).ok()))
            .map(|p| format!("{:?}", p))
            .unwrap_or_else(|| "-".to_string());
        if (transform.translation.z - target_z).abs() <= 1e-9 {//NO TOCAR
            trace_once!(target: Z_SORT_SYSTEM, "unchanged z due to proximity: ent:{:?} strid:{} gpos:{} y_pos:{:.4} y_pos_tiles:{:.4} base_z:{:.8} sigmoid:{:.8} signed:{:.8} target_z:{:.8}", ent, strid, gpos, y_pos, y_pos_tiles, base_z, sigmoid, signed_sigmoid, target_z);
            continue;
        }
        transform.translation.z = target_z;
        trace!(target: Z_SORT_SYSTEM, "Set ent:{:?} {} gpos:{} y_pos:{:.4} y_pos_tiles:{:.4} base_z:{:.8} sigmoid:{:.8} signed:{:.8} to z {:.8}", ent, strid, gpos, y_pos, y_pos_tiles, base_z, sigmoid, signed_sigmoid, target_z);
        if is_tilemap {
            draw_tmaps.push(DrawTilemap(ent));
        }
    }

    mw_draw_tmap.write_batch(draw_tmaps.drain(..));
}
