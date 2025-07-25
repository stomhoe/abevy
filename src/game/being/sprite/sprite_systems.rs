#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
use bevy_spritesheet_animation::prelude::*;

use crate::game::{being::sprite::{sprite_components::*, sprite_resources::*}, game_components::{FacingDirection, ImgPathHolder}};

#[allow(unused_parens)]
pub fn init_sprites(
    mut cmd: Commands, 
    mut seris_handles: ResMut<SpriteSerisHandles>,
    mut assets: ResMut<Assets<SpriteDataSeri>>,
    mut strid_ent_map: ResMut<SpriteDataIdEntityMap>,
) {
    for handle in std::mem::take(&mut seris_handles.handles) {
        info!("Loading SpriteDataSeri from handle: {:?}", handle);
        strid_ent_map.new_spritedata_from_seri(&mut cmd, handle, &mut assets);
    }

    info!("SpriteDataIdEntityMap contents:");
    for (id, ent) in strid_ent_map.map.iter() {
        info!("  - id: {}, entity: {:?}", id, ent);
    }
} 

#[allow(unused_parens)]
pub fn apply_scales(mut query: Query<(
        &SpriteHolderRef, &mut Sprite, &mut Transform,
        Option<&FlipHorizIfDir>,
        Option<&Scale>, Option<&ScaleLookUpDown>, Option<&ScaleLookSideWays>,
    ),>, parent_query: Query<Option<&FacingDirection>>, ) {
    for (
        spriteholder, mut sprite, mut transform, flip_horiz_if_dir,
        scale, scale_look_up_down, scale_look_sideways,
    ) in query.iter_mut() {
        if let Ok(direction) = parent_query.get(spriteholder.0) {
            let mut total_scale = Vec3::ONE;

            if let Some(dir) = direction {
                match dir {
                    FacingDirection::Left => {
                        if let Some(scale_look_sideways) = scale_look_sideways {
                            total_scale *= scale_look_sideways.0.extend(0.);
                        }
                        if let Some(flip_horiz) = flip_horiz_if_dir {
                            sprite.flip_x = match flip_horiz {
                                FlipHorizIfDir::Any => true,
                                FlipHorizIfDir::Left => true,
                                FlipHorizIfDir::Right => false,
                            };
                        }
                    },
                    FacingDirection::Right => {
                        if let Some(scale_look_sideways) = scale_look_sideways {
                            total_scale *= scale_look_sideways.0.extend(0.);
                        }
                        if let Some(flip_horiz) = flip_horiz_if_dir {
                            sprite.flip_x = match flip_horiz {
                                FlipHorizIfDir::Any => true,
                                FlipHorizIfDir::Left => false,
                                FlipHorizIfDir::Right => true,
                            };
                        }
                    },
                    FacingDirection::Up => {
                        if let Some(scale_look_up_down) = scale_look_up_down {
                            total_scale *= scale_look_up_down.0.extend(0.);
                        }
                        if let Some(flip_horiz) = flip_horiz_if_dir {
                            sprite.flip_x = match flip_horiz {
                                FlipHorizIfDir::Any => true,
                                _ => false,
                            };
                        }
                    },
                    FacingDirection::Down => {
                        if let Some(scale_look_up_down) = scale_look_up_down {
                            total_scale *= scale_look_up_down.0.extend(0.);
                        }
                        if let Some(flip_horiz) = flip_horiz_if_dir {
                            sprite.flip_x = match flip_horiz {
                                FlipHorizIfDir::Any => true,
                                _ => false,
                            };
                        }
                    },
                }
            }
            if let Some(scale) = scale {
                total_scale *= scale.0.extend(0.);
            }
            transform.scale = total_scale;
        }
    }
}

#[allow(unused_parens, )]
pub fn apply_offsets(
    mut query: Query<(
        &SpriteHolderRef, &ChildOf, &mut Transform, &Category,
        Option<&Offset>, 
        Option<&OffsetLookUpDown>, Option<&OffsetLookDown>, Option<&OffsetLookUp>,
        Option<&OffsetLookSideways>, Option<&OffsetLookLeft>, Option<&OffsetLookRight>,
    ),>,
    parent_sprite_query: Query<(Option<&OffsetForChildren>), (With<Sprite>)>,
    parent_query: Query<Option<&FacingDirection>>,
) {
    for (
        spriteholder, child_of, mut transform, cat,
        offset, 
        offset_look_up_down, offset_look_down, offset_look_up,
        offset_look_sideways, offset_look_left, offset_look_right,
    ) in query.iter_mut() {
        if let Ok(direction) = parent_query.get(spriteholder.0) {
            let mut total_offset = Vec3::ZERO;
            if let Some(dir) = direction {
                match dir {
                    FacingDirection::Left => {
                        if let Some(offset_look_sideways) = offset_look_sideways {
                            total_offset += offset_look_sideways.0.extend(0.);
                        }
                        if let Some(offset_look_left) = offset_look_left {
                            total_offset += offset_look_left.0.extend(0.);
                        }
                    },
                    FacingDirection::Right => {
                        if let Some(offset_look_sideways) = offset_look_sideways {
                            total_offset += offset_look_sideways.0.extend(0.);
                        }
                        if let Some(offset_look_right) = offset_look_right {
                            total_offset += offset_look_right.0.extend(0.);
                        }
                    },
                    FacingDirection::Up => {
                        if let Some(offset_look_up_down) = offset_look_up_down {
                            total_offset += offset_look_up_down.0.extend(0.);
                        }
                        if let Some(offset_look_up) = offset_look_up {
                            total_offset += offset_look_up.0.extend(0.);
                        }
                    },
                    FacingDirection::Down => {
                        if let Some(offset_look_up_down) = offset_look_up_down {
                            total_offset += offset_look_up_down.0.extend(0.);
                        }
                        if let Some(offset_look_down) = offset_look_down {
                            total_offset += offset_look_down.0.extend(0.);
                        }
                    },
                }
            }
            if let Some(offset) = offset {
                total_offset += offset.0;
            }

            if let Ok(Some(OffsetForChildren(map))) = parent_sprite_query.get(child_of.parent()) {
                if map.contains_key(cat) {
                    total_offset += map[cat].extend(0.);
                } 
            }
            total_offset.z *= 1e17;

            transform.translation = total_offset / 1.;
        }
    }
}

#[allow(unused_parens, )]
pub fn replace_string_ids_by_entities(
    mut cmd: Commands,
    mut query: Query<(Entity, &SpriteDatasChildrenStringIds, Option<&mut SpriteDatasChildrenRefs>), (Added<SpriteDatasChildrenStringIds>)>,
    map: Res<SpriteDataIdEntityMap>,
) {
    for (ent, string_ids, children_refs) in query.iter_mut() {
        let mut entities_vec = if let Some(children_refs) = children_refs {
            std::mem::take(&mut children_refs.into_inner().0)
        } else {
            Vec::new()
        };
        for id in &string_ids.0 {
            if let Some(sprite_ent) = map.get_entity(id) {
                info!("Replacing string id '{}' with entity {:?}", id, sprite_ent);
                entities_vec.push(sprite_ent);
            } else {
                warn!("SpriteDataIdEntityMap does not contain entity for id: {}", id);
            }
        }
        if ! entities_vec.is_empty() {
            cmd.entity(ent).insert(SpriteDatasChildrenRefs(entities_vec));
        }
        cmd.entity(ent).remove::<SpriteDatasChildrenStringIds>();
    }
}


#[allow(unused_parens)]
pub fn add_spritechildren_and_comps(
    mut cmd: Commands,
    aserver: Res<AssetServer>,
    mut atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut father_query: Query<(Entity, &SpriteDatasChildrenRefs, Option<&SpriteHolderRef>,), Without<SpriteDataId>>,
    spritedatas_query: Query<(
        &ImgPathHolder, &AtlasLayoutData, &Category, &OtherCompsToBuild,
        Option<&SpriteDatasChildrenRefs>,
        &SpriteDataId,
    ), With<SpriteDataId>>,
) {
    for (father_to_sprite, SpriteDatasChildrenRefs(to_build), spriteholder_ref,) in father_query.iter_mut() {
        for (sprite_to_insert_ent) in to_build.iter() {
            if let Ok((
                img_path_holder, atlas_layout_data, cat, comps_to_build, children_refs, sprite_data_id
            )) 
            = spritedatas_query.get(*sprite_to_insert_ent) {
                info !("Building sprite with spritedata {}", sprite_data_id.id());
                let spritesheet_size= atlas_layout_data.spritesheet_size;
                let frame_size = atlas_layout_data.frame_size;

                let atlas = TextureAtlas {
                    layout: atlas_layouts.add(Spritesheet::new(spritesheet_size.y as usize, spritesheet_size.x as usize).atlas_layout(frame_size.x, frame_size.y)),
                    ..default()
                };

                let image = aserver.load(format!("texture/{}", &img_path_holder.0));

                let child_sprite = cmd.spawn((
                    ChildOf(father_to_sprite),
                    Sprite::from_atlas_image(image, atlas),
                    cat.clone(),
                )).id();

                if let Some(spriteholder_ref) = spriteholder_ref {
                    cmd.entity(child_sprite).insert(spriteholder_ref.clone());
                } else {
                    cmd.entity(child_sprite).insert(SpriteHolderRef(father_to_sprite));
                }
                if let Some(refs) = children_refs {
                    cmd.entity(child_sprite).insert(refs.clone());
                }
                if let Some(excl) = &comps_to_build.exclusive {
                    cmd.entity(child_sprite).insert(excl.clone());
                }

                if let Some(offset_children) = &comps_to_build.offset_children {
                    cmd.entity(child_sprite).insert(offset_children.clone());
                }
                if let Some(to_become) = &comps_to_build.to_become_child_of_category {
                    cmd.entity(child_sprite).insert(to_become.clone());
                }
                if let Some(display_name) = &comps_to_build.display_name {
                    cmd.entity(child_sprite).insert(display_name.clone());
                }
                if let Some(prefix) = &comps_to_build.anim_prefix {
                    cmd.entity(child_sprite).insert((prefix.clone(), AnimationState::default()));
                }
                if let Some(directionable) = comps_to_build.directionable {
                    cmd.entity(child_sprite).insert(directionable);
                }
                if let Some(walk_anim) = &comps_to_build.walk_anim {
                    cmd.entity(child_sprite).insert(walk_anim.clone());
                }
                if let Some(swim_anim) = &comps_to_build.swim_anim {
                    cmd.entity(child_sprite).insert(swim_anim.clone());
                }
                if let Some(fly_anim) = &comps_to_build.fly_anim {
                    cmd.entity(child_sprite).insert(fly_anim.clone());
                }
                if let Some(offset) = &comps_to_build.offset {
                    cmd.entity(child_sprite).insert(offset.clone());
                }
                if let Some(offset_looking_up_down) = &comps_to_build.offset_looking_up_down {
                    cmd.entity(child_sprite).insert(offset_looking_up_down.clone());
                }
                if let Some(offset_looking_down) = &comps_to_build.offset_looking_down {
                    cmd.entity(child_sprite).insert(offset_looking_down.clone());
                }
                if let Some(offset_looking_up) = &comps_to_build.offset_looking_up {
                    cmd.entity(child_sprite).insert(offset_looking_up.clone());
                }
                if let Some(offset_looking_sideways) = &comps_to_build.offset_looking_sideways {
                    cmd.entity(child_sprite).insert(offset_looking_sideways.clone());
                }
                if let Some(offset_looking_right) = &comps_to_build.offset_looking_right {
                    cmd.entity(child_sprite).insert(offset_looking_right.clone());
                }
                if let Some(offset_looking_left) = &comps_to_build.offset_looking_left {
                    cmd.entity(child_sprite).insert(offset_looking_left.clone());
                }
                if let Some(scale) = &comps_to_build.scale {
                    cmd.entity(child_sprite).insert(scale.clone());
                }
                if let Some(scale_looking_up_down) = &comps_to_build.scale_looking_up_down {
                    cmd.entity(child_sprite).insert(scale_looking_up_down.clone());
                }
                if let Some(scale_looking_sideways) = &comps_to_build.scale_looking_sideways {
                    cmd.entity(child_sprite).insert(scale_looking_sideways.clone());
                }
                if let Some(flip_horiz_if_dir) = &comps_to_build.flip_horiz_if_dir {
                    cmd.entity(child_sprite).insert(flip_horiz_if_dir.clone());
                }
                if let Some(color) = &comps_to_build.color {
                    cmd.entity(child_sprite).insert(color.clone());
                }
            } else{
                warn!("query does not contain entity for: {}", sprite_to_insert_ent);
            }
        }
        cmd.entity(father_to_sprite).remove::<SpriteDatasChildrenRefs>();
    }
}

#[allow(unused_parens)]
pub fn become_child_of_sprite_with_category(mut cmd: Commands, mut to_become_child: Query<(Entity, &SpriteHolderRef, &Category, &ToBecomeChildOfCategory),(With<Sprite>, Without<SpriteDataId>)>, 
to_become_parent: Query<(Entity, &SpriteHolderRef, &Category, ), (With<Sprite>,)>,
) {
    for (tochild_ent, SpriteHolderRef(tochild_spriteholder), cat, child_of_cat) in to_become_child.iter_mut() {
        for (toparent_ent, SpriteHolderRef(toparent_spriteholder), parents_cat, ) in to_become_parent.iter() {
            if tochild_spriteholder == toparent_spriteholder && child_of_cat.0 == *parents_cat {
                info!("Adding ChildOfCategory to entity {:?} with id: {}", tochild_ent, cat);

                cmd.entity(tochild_ent).insert(ChildOf(toparent_ent));
                cmd.entity(tochild_ent).remove::<ToBecomeChildOfCategory>();

                break;
            }
        }
        
    }
}
