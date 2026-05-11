use bevy::ecs::entity::EntityHashSet;
use bevy_ecs_tilemap::{DrawTilemap, anchor::TilemapAnchor};
#[allow(unused_imports)]
use bevy::prelude::*;
#[allow(unused_imports)]
use bevy_replicon::prelude::*;
use game_common::game_common_components::TemplEntiRef;
use ::sprite_shared::*;
use tilemap_shared::{GlobalTilePos, ZSettings};

pub type Ysortable = (Or<(With<Sprite>, With<TilemapAnchor>, With<Mesh2d>/*Or-END*/)>, With<Visibility>, Without<HeldSprites>, 
//Without<LightOccluder2d>, 
);

#[allow(unused_parens, )]
pub fn y_sort_system(
    y_sort_settings: Query<&ZSettings>,
    sprite_holders: Query<&HeldSprites, Changed<GlobalTilePos>, >,
    changed_query: Query<Entity,
        (Or<(Changed<TemplEntiRef>, Changed<GlobalTilePos>, Changed<YSortOrigin>, Changed<AcZ>,
            Changed<ChildOf>, Added<Sprite>, Added<Mesh2d>,)/*Or-END*/>,
        Ysortable)>,
    mut process_query: Query<(Entity, &mut Transform, &GlobalTransform, Option<&TemplEntiRef>, Has<TilemapAnchor>, &ChildOf, ),>,
    acz_query: Query<&AcZ, ()>,
    y_sort_query: Query<&YSortOrigin, ()>,

    parent_sprite_query: Query<&Sprite, (common::AnyDisabling,)>,
    camera_query: Query<Ref<GlobalTilePos>, With<Camera>>,
    all_ysortable_query: Query<Entity, (Ysortable)>,

    mut mw_draw_tmap: MessageWriter<DrawTilemap>,
    mut draw_tmaps: Local<Vec<DrawTilemap>>,
    mut ents_to_process: Local<EntityHashSet>,
) {
    let Ok(y_sort_settings) = y_sort_settings.single() else {
        return;
    };

    ents_to_process.extend(changed_query.iter());

    for held_sprites in sprite_holders.iter() {
        ents_to_process.reserve(held_sprites.len());
        for held in held_sprites.iter() {
            if all_ysortable_query.get(held).is_ok() {
                ents_to_process.insert(held);
            }
        }
    }

    let (camera_y, camera_moved) = camera_query.iter().next()
        .map(|cam| (cam.to_pixelpos().y, cam.is_changed()))
        .unwrap_or((0.0, false));

    if camera_moved {
        ents_to_process.extend(all_ysortable_query.iter());
    }
    let mut iter = process_query.iter_many_mut(ents_to_process.drain());
    while let Some((ent, mut transform, global_transform, templ_ref, is_tilemap, child_of)) = iter.fetch_next() {
        let has_parent_sprite = parent_sprite_query.get(child_of.parent()).is_ok();
        let ent_ysort_origin = y_sort_query.get(ent).ok();
        let anim_ac_z = acz_query.get(ent).ok();

        let (base_z, maybe_ysort_origin) = if let Some(templ_ref) = templ_ref
        {
            let templ_ac_z = acz_query.get(templ_ref.0).ok();
            let templ_ysort_origin = y_sort_query.get(templ_ref.0).ok();
            let base_z = if let Some(anim_ac_z) = anim_ac_z.copied() {
                if anim_ac_z.0.is_finite() {
                    anim_ac_z.used_float(&y_sort_settings)
                } else {
                    templ_ac_z.copied().unwrap_or_default().used_float(&y_sort_settings)
                }
            } else {
                templ_ac_z.copied().unwrap_or_default().used_float(&y_sort_settings)
            };
            (base_z, ent_ysort_origin.copied().or(templ_ysort_origin.copied()))
        } else {
            (anim_ac_z.cloned().unwrap_or_default().used_float(&y_sort_settings), ent_ysort_origin.copied())
        };

        let y = global_transform.translation().y;
        let origin_y = maybe_ysort_origin.unwrap_or_default().0;
        if !base_z.is_finite() || !y.is_finite() || !origin_y.is_finite() {
            continue;
        }

        let y_pos = y - origin_y;
        let use_y_sort = (maybe_ysort_origin.is_some() && !has_parent_sprite) as i32 as f32;
        let y_distance_to_camera = camera_y - y_pos;
        let target_z = base_z + use_y_sort * y_distance_to_camera * y_sort_settings.y_sort_mult;

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