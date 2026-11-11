use bevy::ecs::entity::EntityHashSet;
use bevy_ecs_tilemap::{DrawTilemap, anchor::TilemapAnchor};
#[allow(unused_imports)]
use bevy::prelude::*;
#[allow(unused_imports)]
use bevy_replicon::prelude::*;
use game_common::game_common_components::TemplEntiRef;
use ::sprite_shared::*;
use common::common_components::SettingsEntity;
use tilemap_shared::{GlobalTilePos, ZSettings};
use bevy::render::*;
use bevy::render::camera::DirtySpecializations;
use bevy::render::sync_world::MainEntity;

// Bevy 0.20 retains Mesh2d phase items, but its material invalidation query
// does not include transform changes. Requeue moved sprites so their retained
// Transparent2d sort keys use the newly extracted world depth.
#[allow(unused_parens, )]
pub fn extract_changed_sprite_depths(
    sprites: Extract<Query<(Entity, ), (Or<(With<Sprite>, With<Mesh2d>)>, Changed<GlobalTransform>, )>>,
    mut dirty: ResMut<DirtySpecializations>,
) {
    dirty.changed_renderables.extend(sprites.iter().map(|(entity, )| MainEntity::from(entity)));
}

pub type Ysortable = (Or<(With<Sprite>, With<TilemapAnchor>, With<Mesh2d>/*Or-END*/)>, With<Visibility>, Without<HeldSprites>,
//Without<LightOccluder2d>,
);

#[allow(unused_parens, )]
pub fn y_sort_system(
    y_sort_settings: Query<&ZSettings, With<SettingsEntity>>,
    sprite_holders: Query<&HeldSprites, Changed<GlobalTilePos>, >,
    changed_query: Query<Entity,
        (Or<(Changed<TemplEntiRef>, Changed<GlobalTilePos>, Changed<GlobalTransform>, Changed<YSortOrigin>, Changed<AcZ>,
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
    loop {
        let Some(result) = iter.fetch_next() else {
            break;
        };
        let Ok((ent, mut transform, global_transform, templ_ref, is_tilemap, child_of)) = result else { continue; };
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn world_depth_changes_invalidate_retained_sprite_phase_items() {
        let mut app = App::new();
        app.init_resource::<MainWorld>()
            .init_resource::<DirtySpecializations>()
            .add_systems(Update, extract_changed_sprite_depths);
        let sprite = app.world_mut().resource_mut::<MainWorld>()
            .spawn((Sprite::default(), Mesh2d::default(), GlobalTransform::from_xyz(0., 0., 10.))).id();
        app.update();
        app.world_mut().resource_mut::<DirtySpecializations>().changed_renderables.clear();
        {
            let mut main = app.world_mut().resource_mut::<MainWorld>();
            main.clear_trackers();
            main.increment_change_tick();
            *main.get_mut::<GlobalTransform>(sprite).unwrap() = GlobalTransform::from_xyz(0., 0., 9.999968);
        }
        app.update();
        assert!(app.world().resource::<DirtySpecializations>().changed_renderables.contains(&MainEntity::from(sprite)));
        app.world_mut().resource_mut::<DirtySpecializations>().changed_renderables.clear();
        app.world_mut().resource_mut::<MainWorld>().increment_change_tick();
        app.update();
        assert!(app.world().resource::<DirtySpecializations>().changed_renderables.is_empty());
    }

    #[test]
    fn stopping_with_a_following_camera_keeps_static_trees_sorted() {
        let mut app = App::new();
        app.add_plugins(bevy::transform::TransformPlugin)
            .add_message::<DrawTilemap>()
            .add_systems(PostUpdate, y_sort_system.after(bevy::transform::TransformSystems::Propagate));
        app.world_mut().spawn((SettingsEntity, ZSettings::default()));
        let camera = app.world_mut().spawn((Camera::default(), GlobalTilePos::default())).id();
        let config = app.world_mut().spawn((AcZ(1000.), YSortOrigin(8.))).id();
        let being = app.world_mut().spawn(Transform::from_xyz(291552., 14944., 0.)).id();
        let body = app.world_mut().spawn((Sprite::default(), Transform::from_xyz(0., 8., 0.), TemplEntiRef(config), ChildOf(being))).id();
        let tree_parent = app.world_mut().spawn(Transform::from_xyz(291552., 14976., 0.)).id();
        let tree = app.world_mut().spawn((Sprite::default(), Transform::from_xyz(0., 120., 0.), AcZ(1000.), YSortOrigin(110.), ChildOf(tree_parent))).id();
        for y in [14944., 15008., 14944.] {
            app.world_mut().get_mut::<Transform>(being).unwrap().translation.y = y;
            app.world_mut().entity_mut(camera).insert(GlobalTilePos::from(Vec2::new(291552., y)));
            // Animation switches to idle and removes animation-specific depth.
            app.world_mut().entity_mut(body).remove::<(AcZ, YSortOrigin)>();
            for _ in 0..5 {
                app.update();
            }
            let body_z = app.world().get::<GlobalTransform>(body).unwrap().translation().z;
            let tree_z = app.world().get::<GlobalTransform>(tree).unwrap().translation().z;
            assert_eq!(body_z, 10.);
            assert_eq!(body_z > tree_z, y < 14986., "body z={body_z:.9}, tree z={tree_z:.9}, y={y}");
        }
    }

    #[test]
    fn interpolated_body_motion_crosses_tree_depth_without_a_new_tile_position() {
        let mut app = App::new();
        app.add_plugins(bevy::transform::TransformPlugin)
            .add_message::<DrawTilemap>()
            .add_systems(PostUpdate, y_sort_system.after(bevy::transform::TransformSystems::Propagate));
        app.world_mut().spawn((SettingsEntity, ZSettings::default()));
        let body_config = app.world_mut().spawn((AcZ(1000.), YSortOrigin(8.))).id();
        let being = app.world_mut().spawn((Transform::from_xyz(0., -32., 0.), GlobalTilePos::default())).id();
        let body = app.world_mut().spawn((Sprite::default(), Transform::from_xyz(0., 8., 0.), TemplEntiRef(body_config), ChildOf(being), BaseHolderRef { base: being })).id();
        let tree_parent = app.world_mut().spawn(Transform::default()).id();
        let tree = app.world_mut().spawn((Sprite::default(), Transform::from_xyz(0., 120., 0.), AcZ(1000.), YSortOrigin(110.), ChildOf(tree_parent))).id();
        app.update();
        app.update();
        assert!(app.world().get::<GlobalTransform>(body).unwrap().translation().z > app.world().get::<GlobalTransform>(tree).unwrap().translation().z);

        // Grid movement interpolates Transform between tile-position changes.
        app.world_mut().get_mut::<Transform>(being).unwrap().translation.y = 32.;
        app.update();
        app.update();
        assert!(app.world().get::<GlobalTransform>(body).unwrap().translation().z < app.world().get::<GlobalTransform>(tree).unwrap().translation().z);
    }

    #[test]
    fn incomplete_entities_do_not_interrupt_north_south_sorting() {
        let mut app = App::new();
        app.add_message::<DrawTilemap>().add_systems(Update, y_sort_system);
        app.world_mut().spawn((SettingsEntity, ZSettings::default()));
        let parent = app.world_mut().spawn_empty().id();
        let mut sprites = Vec::with_capacity(33);
        for y in (-16..=16).map(|row| row as f32 * 32.) {
            // The middle sprite represents a bush; the others represent beings
            // to its north and south, all sharing the same world depth layer.
            let sprite = app.world_mut().spawn((Sprite::default(), Transform::default(), GlobalTransform::from_translation(Vec3::new(0., y, 0.)), ChildOf(parent), YSortOrigin(0.), AcZ(1000.))).id();
            sprites.push(sprite);
            // Such sprites qualify for sorting but cannot yet be processed.
            app.world_mut().spawn(Sprite::default());
        }
        app.update();
        let depths: Vec<_> = sprites.iter().map(|&entity| app.world().get::<Transform>(entity).unwrap().translation.z).collect();
        assert!(depths.windows(2).all(|pair| pair[0] > pair[1]));
        assert_eq!(depths[16], 10.);
    }
}
