use std::{collections::HashSet, };

#[allow(unused_imports)] use bevy::prelude::*;
use bevy::ecs::entity::EntityHashSet;
use bevy_ecs_tilemap::{DrawTilemap, anchor::TilemapAnchor};
#[allow(unused_imports)] use bevy_replicon::prelude::*;
use bevy::ecs::entity_disabling::Disabled;
use game_common::game_common_components::{TemplEnti, TemplEntiRef, };
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
    baseholder_query: Query<&HeldSprites, (Or<(Changed<CardinalDirection>, Changed<HeldSprites>, Added<GlobalTilePos>)>)>,
    mut removed_disabled: RemovedComponents<Disabled>,
    mut writer: MessageWriter<SpriteChanged>,
    mut changed: Local<HashSet<SpriteChanged>>,
)
{
    for sprite_ent in sprite_query.iter() {
        changed.insert(SpriteChanged(sprite_ent));
    }
    for sprite_ent in removed_disabled.read() {
        changed.insert(SpriteChanged(sprite_ent));
    }
    for held_sprites in baseholder_query.iter() {
        for sprite_ent in held_sprites.iter() {
            changed.insert(SpriteChanged(sprite_ent));
        }
    }
    writer.write_batch(changed.drain());
}


#[allow(unused_parens)]
pub fn disable_children_sprites_of_disabled(
    mut cmd: Commands,
    templ_bases: Query<(&HeldSprites),(With<TemplEnti>, Added<Disabled>)>,
    non_templ_bases: Query<(&HeldSprites),(Without<TemplEnti>,)>,
    mut removed: RemovedComponents<Disabled>,
) {
    let mut disableds = Vec::new();
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
    for ent in changed_query.iter() {
        ents_to_process.insert(ent);
    }
    let (camera_y, camera_moved) = camera_query.iter().next()
        .map(|cam| (cam.to_pixelpos().y, cam.is_changed()))
        .unwrap_or((0.0, false));

    if camera_moved {
        for ent in all_spriteable_query.iter() {
            ents_to_process.insert(ent);
        }
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
