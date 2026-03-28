use std::{collections::HashSet, };

use being_shared::Unloaded;
#[allow(unused_imports)] use bevy::prelude::*;
use bevy::ecs::entity::EntityHashSet;
use bevy_ecs_tilemap::{DrawTilemap, anchor::TilemapAnchor};
#[allow(unused_imports)] use bevy_replicon::prelude::*;
use bevy::ecs::entity_disabling::Disabled;
use game_common::game_common_components::{Templ, TemplEntiRef, };
use tilemap_shared::{GlobalTilePos};
use ::tilemap_shared::directions::*;

use ::sprite_shared::{sprite_scale_offset::*, };

#[allow(unused_imports, )]
use crate::sprite_components::*;


#[derive(Message, Debug, Clone, Hash, PartialEq, Eq)]
pub struct SpriteChanged(pub Entity);




#[allow(unused_parens)]
pub fn sprite_change_detection(
    sprite_query: Query<Entity, (Or<(Changed<Scale2D>, Changed<ScaleLookUpDown>, Changed<ScaleSideways>, Changed<TemplEntiRef>, Changed<Offset2D>, Changed<Sprite>, Changed<ChildOf>)>)>,
    baseholder_query: Query<&HeldSprites, (Or<(Changed<CardinalDirection>, Changed<HeldSprites>, Added<GlobalTilePos>, Changed<Visibility>)>, Without<Unloaded>, Without<Disabled>)>,
    mut removed_disabled: RemovedComponents<Disabled>,
    mut removed_unloaded: RemovedComponents<Unloaded>,
    mut writer: MessageWriter<SpriteChanged>,
    mut changed: Local<HashSet<SpriteChanged>>,
)
{
    changed.extend(sprite_query.iter().map(SpriteChanged));
    changed.extend(removed_disabled.read().map(SpriteChanged));
    changed.extend(baseholder_query.iter().flat_map(|held_sprites| held_sprites.iter().map(|sprite_ent| SpriteChanged(sprite_ent))));
    changed.extend(removed_unloaded.read().map(|removed| SpriteChanged(removed)));
    writer.write_batch(changed.drain());
}


#[allow(unused_parens)]
pub fn disable_children_sprites_of_disabled(
    mut cmd: Commands,
    templ_bases: Query<(&HeldSprites),(With<Templ>, Added<Disabled>)>,
    non_templ_bases: Query<(&HeldSprites),(Without<Templ>,)>,
    mut removed: RemovedComponents<Disabled>,
) {
    let iter = templ_bases.iter();
    let mut disableds = Vec::with_capacity(iter.size_hint().1.unwrap_or(iter.size_hint().0));
    for (held_sprites) in templ_bases.iter() {
        for sprite_ent in held_sprites.iter() {
            disableds.push((sprite_ent, Disabled));
        }
    }
    for ent in removed.read() {
        if let Ok((held_sprites)) = non_templ_bases.get(ent) {
            for sprite_ent in held_sprites.iter() {
                cmd.entity(sprite_ent).try_remove::<Disabled>();
            }
        }
    }
    cmd.try_insert_batch(std::mem::take(&mut disableds));
}

pub type Zsortable = (Or<(With<Sprite>, With<TilemapAnchor>, With<Mesh2d>)>, With<InheritedVisibility>, );

#[allow(unused_parens, )]
pub fn z_sort_system(
    changed_query: Query<Entity,
        (Or<(Changed<TemplEntiRef>, Changed<GlobalTilePos>, Changed<YSortOrigin>, Changed<AcZ>, Changed<ChildOf>, Added<Sprite>, Added<Mesh2d>,)>,
        Zsortable)>,
    mut process_query: Query<(Entity, &mut Transform, &GlobalTransform, Option<&YSortOrigin>,
        Option<&AcZ>, Option<&TemplEntiRef>, Has<TilemapAnchor>, &ChildOf, ),>,

    parent_sprite_query: Query<&Sprite, (common::AnyDisabling,)>,
    camera_query: Query<Ref<GlobalTilePos>, With<Camera>>,
    all_spriteable_query: Query<Entity, (Zsortable)>,

    templ_query: Query<(&AcZ, Option<&YSortOrigin>), ()>,

    mut mw_draw_tmap: MessageWriter<DrawTilemap>,
    mut draw_tmaps: Local<Vec<DrawTilemap>>,
    mut ents_to_process: Local<EntityHashSet>,
) {
    ents_to_process.extend(changed_query.iter());
    let (camera_y, camera_moved) = camera_query.iter().next()
        .map(|cam| (cam.to_pixelpos().y, cam.is_changed()))
        .unwrap_or((0.0, false));

    if camera_moved {
        ents_to_process.extend(all_spriteable_query.iter());
    }
    let mut iter = process_query.iter_many_mut(ents_to_process.drain());
    while let Some((ent, mut transform, global_transform, ysort_origin, maybe_z_index, templ_ref, is_tilemap, child_of)) = iter.fetch_next() {
        let has_parent_sprite = parent_sprite_query.get(child_of.parent()).is_ok();

        let (base_z, maybe_ysort_origin) = if let Some(templ_ref) = templ_ref
            && let Ok((templ_z_index, templ_ysort_origin)) = templ_query.get(templ_ref.0)
        {
            (templ_z_index.used_float(), templ_ysort_origin.copied().or(ysort_origin.copied()))
        } else {
            (maybe_z_index.cloned().unwrap_or_default().used_float(), ysort_origin.copied())
        };

        let y = global_transform.translation().y;
        let origin_y = maybe_ysort_origin.unwrap_or_default().0;
        if !base_z.is_finite() || !y.is_finite() || !origin_y.is_finite() {
            continue;
        }

        let y_pos = y - origin_y;
        let use_y_sort = (maybe_ysort_origin.is_some() && !has_parent_sprite) as i32 as f32;
        let y_distance_to_camera = camera_y - y_pos;
        let target_z = base_z + use_y_sort * y_distance_to_camera * AcZ::Z_SORT_MULT;

        if (transform.translation.z - target_z).abs() <= 1e-9 {//NO TOCAR
            continue;
        }
        transform.translation.z = target_z;
        if is_tilemap {
            draw_tmaps.push(DrawTilemap(ent));
        }
    }

    mw_draw_tmap.write_batch(draw_tmaps.drain(..));
}
