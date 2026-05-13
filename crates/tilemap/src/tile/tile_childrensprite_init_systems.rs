use ::game_common::*;
use ::sprite_shared::*;
#[allow(unused_imports)]
use bevy::{
    math::U16Vec2,
    prelude::*,
    ecs::entity::{EntityHashMap, EntityHashSet},
    platform::collections::HashSet,
};
use bevy_ecs_tilemap::prelude::*;
use bevy_lit::prelude::LightOccluder2d;
use common::{AnyDisabling, common_components::*, log_targets::{CHILDRENSPRITE_INIT, OCCLUDER_INIT}};
use common::common_resources::*;
use sprite_animation_shared::AcAnimationProgresses;

use crate::{
    tile::{
        tile_components::*,
        tile_resources::*,
        tile_seris::LightOccluderSeri,
        tile_shader::{tile_shader_components::*,},
    },
};


fn spawn_child_sprite_occluder_entity(
    cmd: &mut Commands,
    parent: Entity,
    occluder_mask: Handle<Image>,
    occluder_mesh: Handle<Mesh>,
    occluder_height: f32,
    offset: (f32, f32),
) {
    cmd.spawn((
        ChildOf(parent),
        BaseHolderRef { base: parent },
        Transform::default(),
        GlobalTransform::default(),
        Visibility::Inherited,
        TileChildSpriteOccluder,
        Mesh2d(occluder_mesh),
        Offset2D::from(offset),
        FlippedTransform { x: false, y: false },
        NegativizeRotationOnTileFlip,
        AutoCorrectOffsetBasedOnParentSizeResults,
        LightOccluder2d::with_height(occluder_mask, occluder_height),
    ));
}

fn spawn_child_sprite_occluder_stub(cmd: &mut Commands, parent: Entity, offset: (f32, f32)) {
    cmd.spawn((
        ChildOf(parent),
        BaseHolderRef { base: parent },
        Transform::default(),
        GlobalTransform::default(),
        Visibility::Inherited,
        TileChildSpriteOccluder,
        Offset2D::from(offset),
        FlippedTransform { x: false, y: false },
        NegativizeRotationOnTileFlip,
    ));
}

fn entity_label(entity: Entity, str_id_query: &Query<&StrId>) -> String {
    let strid = str_id_query.get(entity).ok().map(|id| id.as_str()).unwrap_or("");
    format!("\"{}\" {:?}", strid, entity)
}

#[allow(unused_parens)]
pub fn init_childrensprite(
    mut cmd: Commands,
    childrensprite_query: Query<
        (Entity, AnyOf<(&PathHolder, &TileRef)>, Has<Templ>),
        (
            With<TileChildSprite>,
            Or<(Added<TileChildSprite>, Changed<PathHolder>, Changed<TileRef>)>,
            Without<Sprite>,
            Without<AcAnimationProgresses>,
            Without<TilemapId>,
            Without<TileShader>,
            AnyDisabling,
        ),
    >,
    templ_img_path: Query<(Option<&PathHolder>, Has<SpriteConfig>), (With<Templ>,)>,
    tile_map: Res<TileEntityMap>,
    aserver: Res<AssetServer>,
) {
    let mut to_insert = Vec::new();
    for (childsprite_ent, (image_path_holder, templ_ref), is_templ) in childrensprite_query.iter() {
        if let Some(img_path_holder) = image_path_holder {
            trace!(target: CHILDRENSPRITE_INIT, "Inserting Sprite for entity {:?} with direct ImagePathHolder: {:?}", childsprite_ent, img_path_holder.path());
            let image = aserver.load(img_path_holder.path().clone());
            let visibility = if is_templ {
                Visibility::Hidden
            } else {
                Visibility::Inherited
            };
            to_insert.push((
                childsprite_ent,
                (Sprite {
                    image: image.clone(),
                    ..Default::default()
                },
                visibility,
            )
            ));
        } else if let Some(templ_ref) = templ_ref {
            let Ok(templ_ent) = tile_map.0.get_cloned(templ_ref.0) else {
                error!(target: CHILDRENSPRITE_INIT, "Entity {:?} has TileRef {:?} but the referenced tile entity doesn't exist", childsprite_ent, templ_ref.0);
                continue;
            };
            let Ok((img_path_holder, is_templ_a_spriteconfig)) = templ_img_path.get(templ_ent)
            else {
                error!(target: CHILDRENSPRITE_INIT, "Entity {:?} has TileRef {:?} but the referenced tile entity doesn't exist", childsprite_ent, templ_ref.0);
                continue;
            };
            if is_templ_a_spriteconfig {
                continue;
            }
            let Some(img_path_holder): Option<&PathHolder> = img_path_holder else {
                error!(target: CHILDRENSPRITE_INIT, "Entity {:?} has TileRef {:?} but the referenced tile entity has no ImagePathHolder", childsprite_ent, templ_ref.0);
                continue;
            };

            trace!(target: CHILDRENSPRITE_INIT, "Inserting Sprite for entity {:?} via TileRef {:?}, path: {:?}", childsprite_ent, templ_ref.0, img_path_holder.path());
            let image = aserver.load(img_path_holder.path().clone());
            let visibility = if is_templ {
                Visibility::Hidden
            } else {
                Visibility::Inherited
            };
            if is_templ {
                cmd.entity(childsprite_ent).try_insert_if_new(BaseHolderRef { base: templ_ent });
            }

            to_insert.push((
                childsprite_ent,
                (Sprite {
                    image: image.clone(),
                    ..Default::default()
                },
                visibility,)
            ));
        } else {
            error!(target: CHILDRENSPRITE_INIT, "Entity {:?} has neither ImagePathHolder nor TileRef", childsprite_ent);
        }
    }
    cmd.try_insert_batch(to_insert);
}

#[allow(unused_parens)]
pub fn init_templ_childrensprite_light_occluders(
    mut cmd: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
    mut pending_image_size_updates: ResMut<RegisteredImageSizeUpdateObservers>,
    childrensprite_query: Query<
        (Entity, AnyOf<(&PathHolder, &TileRef)>, &BaseHolderRef, ),
        (
            With<Templ>,
            With<TileChildSprite>,
            Or<(Added<TileChildSprite>, Changed<PathHolder>, Changed<TileRef>)>,
            Without<AcAnimationProgresses>,
            Without<TilemapId>,
            Without<TileShader>,
            AnyDisabling,
        ),
    >,
    templ_img_path: Query<(&PathHolder, ), (With<Templ>, Without<SpriteConfig>)>,
    templ_light_occluder: Query<&LightOccluderSeri, With<Templ>>,
    tile_map: Res<TileEntityMap>,
    aserver: Res<AssetServer>,
    mut occluder_cache: Local<EntityHashMap<(Handle<Image>, Handle<Mesh>)>>,
    str_id_query: Query<&StrId>,
) {
    for (childsprite_ent, (image_path_holder, templ_ref), base_holder_ref, ) in childrensprite_query.iter() {

        let Some(img_path_holder) = image_path_holder else {
            let Some(templ_ref) = templ_ref else {
                continue;
            };

            let Ok(templ_ent) = tile_map.0.get_cloned(templ_ref.0) else {
                continue;
            };
            let Ok(light_occluder) = templ_light_occluder.get(templ_ent) else {
                warn!(target: OCCLUDER_INIT, "Skipping templ child sprite {}: template {} has no LightOccluderSeri", entity_label(childsprite_ent, &str_id_query), entity_label(templ_ent, &str_id_query));
                continue;
            };
            if !light_occluder.enabled {
                continue;
            }
            if light_occluder.use_sprite {
                let Ok((img_path_holder, )) = templ_img_path.get(templ_ent) else {
                    warn!(target: OCCLUDER_INIT, "Skipping templ child sprite {}: template {} has sprite occluder enabled but no PathHolder", entity_label(childsprite_ent, &str_id_query), entity_label(templ_ent, &str_id_query));
                    continue;
                };
                let image = aserver.load(img_path_holder.path());
                pending_image_size_updates.register(image.id(), childsprite_ent);
                spawn_child_sprite_occluder_stub(
                    &mut cmd,
                    base_holder_ref.base,
                    light_occluder.offset,
                );
                trace!(target: OCCLUDER_INIT, "Spawned sprite occluder stub for child {} from template {} at base {} offset={:?}", entity_label(childsprite_ent, &str_id_query), entity_label(templ_ent, &str_id_query), entity_label(base_holder_ref.base, &str_id_query), light_occluder.offset);
            } else {
            let (occluder_mask, occluder_mesh) = occluder_handle_pair(
                templ_ent,
                light_occluder,
                &mut occluder_cache,
                &mut images,
                &mut meshes,
            );
            spawn_child_sprite_occluder_entity(
                &mut cmd,
                childsprite_ent,
                occluder_mask,
                occluder_mesh,
                light_occluder.shape_height(),
                light_occluder.offset,
            );
            trace!(target: OCCLUDER_INIT, "Spawned mesh occluder entity for child {} from template {} with height={:.3}", entity_label(childsprite_ent, &str_id_query), entity_label(templ_ent, &str_id_query), light_occluder.shape_height());
        }

        continue;
    };

        let Ok(light_occluder) = templ_light_occluder.get(base_holder_ref.base) else {
            warn!(target: OCCLUDER_INIT, "Skipping child sprite {}: base holder {} has no LightOccluderSeri", entity_label(childsprite_ent, &str_id_query), entity_label(base_holder_ref.base, &str_id_query));
            continue;
        };
        if !light_occluder.enabled {
            continue;
        }
        if light_occluder.use_sprite {
            let image = aserver.load(img_path_holder.path());
            pending_image_size_updates.register(image.id(), childsprite_ent);
            spawn_child_sprite_occluder_stub(
                &mut cmd,
                base_holder_ref.base,
                light_occluder.offset,
            );
            trace!(target: OCCLUDER_INIT, "Spawned sprite occluder stub for child {} at base {} offset={:?}", entity_label(childsprite_ent, &str_id_query), entity_label(base_holder_ref.base, &str_id_query), light_occluder.offset);
        } else {
            let (occluder_mask, occluder_mesh) = occluder_handle_pair(
                base_holder_ref.base,
                light_occluder,
                &mut occluder_cache,
                &mut images,
                &mut meshes,
            );
            spawn_child_sprite_occluder_entity(
                &mut cmd,
                base_holder_ref.base,
                occluder_mask,
                occluder_mesh,
                light_occluder.shape_height(),
                light_occluder.offset,
            );
            trace!(target: OCCLUDER_INIT, "Spawned mesh occluder entity for child {} at base {} with height={:.3}", entity_label(childsprite_ent, &str_id_query), entity_label(base_holder_ref.base, &str_id_query), light_occluder.shape_height());
        }
    }
}
fn occluder_handle_pair(
    template_entity: Entity,
    light_occluder: &LightOccluderSeri,
    occluder_cache: &mut EntityHashMap<(Handle<Image>, Handle<Mesh>)>,
    images: &mut Assets<Image>,
    meshes: &mut Assets<Mesh>,
) -> (Handle<Image>, Handle<Mesh>) {
    if let Some((image, mesh)) = occluder_cache.get(&template_entity) {
        return (image.clone(), mesh.clone());
    }

    let image = images.add(light_occluder.to_shape_mask_image());
    let mesh = meshes.add(Rectangle::new(light_occluder.shape_size.0.max(1.0), light_occluder.shape_size.1.max(1.0)));
    occluder_cache.insert(template_entity, (image.clone(), mesh.clone()));
    (image, mesh)
}


#[allow(unused_parens)]
pub fn fix_childrensprite_spritemask_occluders_img_size(
    mut cmd: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    aserver: Res<AssetServer>,
    image_size_map: Res<ImageSizeMap>,
    mut image_events: MessageReader<ImageSizeReady>,
    templ_light_occluder: Query<&LightOccluderSeri, With<Templ>>,
    child_query: Query<
        (
            &PathHolder,
            Option<&BaseHolderRef>,
            Option<&Children>,
        ),
        (With<TileChildSprite>, With<Templ>, AnyDisabling),
    >,
    occluder_query: Query<&TileChildSpriteOccluder>,
    str_id_query: Query<&StrId>,
) {
    for image_ready in image_events.read() {
        let Some(image_size) = image_size_map.0.get(&image_ready.image_id).copied() else {
            continue;
        };

        let Ok((path_holder, base_holder_ref, children)) = child_query.get(image_ready.entity) else {
            continue;
        };

        let template_entity = if let Some(base_holder_ref) = base_holder_ref {
            base_holder_ref.base
        } else {
            warn!(target: OCCLUDER_INIT, "ImageSizeReady event for {} has no BaseHolderRef, skipping occluder fix", entity_label(image_ready.entity, &str_id_query));
            continue;
        };

        let Ok(light_occluder_seri) = templ_light_occluder.get(template_entity) else {
            warn!(target: OCCLUDER_INIT, "Skipping occluder resize for {}: template entity {} has no LightOccluderSeri", entity_label(image_ready.entity, &str_id_query), entity_label(template_entity, &str_id_query));
            continue;
        };
        if !light_occluder_seri.enabled {
            continue;
        }
        if !light_occluder_seri.use_sprite {
            continue;
        }

        let occluder_entity = children
            .and_then(|children| {
                for child in children.iter() {
                    if occluder_query.get(child).is_ok() {
                        return Some(child);
                    }
                }
                None
            })
            .unwrap_or_else(|| {
                cmd.spawn((
                    ChildOf(template_entity),
                    BaseHolderRef { base: template_entity },
                    Transform::default(),
                    GlobalTransform::default(),
                    Visibility::Inherited,
                    TileChildSpriteOccluder,
                    Offset2D::from(light_occluder_seri.offset),
                    FlippedTransform { x: false, y: false },
                    AutoCorrectOffsetBasedOnParentSizeResults,
                ))
                .id()
            });

        let image = aserver.load(path_holder.path().clone());
        cmd.entity(occluder_entity).try_insert((
            Mesh2d(meshes.add(Rectangle::new(image_size.x as f32, image_size.y as f32))),
            LightOccluder2d::with_height(image, image_size.y as f32),
        ));
        trace!(target: OCCLUDER_INIT, "Updated child sprite occluder entity {} for parent {} to size={:?}", entity_label(occluder_entity, &str_id_query), entity_label(image_ready.entity, &str_id_query), image_size);
    }
}

