#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
use bevy_spritesheet_animation::prelude::*;

use crate::game::{being::sprite::{self, sprite_components::*, sprite_resources::*}, game_components::{FacingDirection, ImgPathHolder}};

#[allow(unused_parens)]
pub fn init_sprites(
    mut cmd: Commands, 
    mut seris_handles: ResMut<SpriteSerisHandles>,
    mut assets: ResMut<Assets<SpriteDataSeri>>,
    mut strid_ent_map: ResMut<SpriteDataIdEntityMap>,
) {
    let handles_vec = std::mem::take(&mut seris_handles.handles);
    
    for handle in handles_vec {
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
        &ChildOf, &mut Sprite, &mut Transform,
        Option<&FlipHorizIfDir>,
        Option<&Scale>, Option<&ScaleLookUpDown>, Option<&ScaleLookSideWays>,
    ),>, parent_query: Query<Option<&FacingDirection>>, ) {
    for (
        child_of, mut sprite, mut transform, flip_horiz_if_dir,
        scale, scale_look_up_down, scale_look_sideways,
    ) in query.iter_mut() {
        if let Ok(direction) = parent_query.get(child_of.parent()) {
            if let Some(dir) = direction {
                match dir {
                    FacingDirection::Left => {
                        if let Some(scale_look_sideways) = scale_look_sideways {
                            transform.scale = scale_look_sideways.0.extend(0.);
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
                            transform.scale = scale_look_sideways.0.extend(0.);
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
                            transform.scale = scale_look_up_down.0.extend(0.);
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
                            transform.scale = scale_look_up_down.0.extend(0.);
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
                transform.scale = scale.0.extend(0.);
            }
        }
    }
}

pub fn apply_offsets(
    mut query: Query<(
        &ChildOf, &mut Transform,
        Option<&Offset>,
        Option<&OffsetLookUpDown>, Option<&OffsetLookDown>, Option<&OffsetLookUp>,
        Option<&OffsetLookSideways>, Option<&OffsetLookLeft>, Option<&OffsetLookRight>,
    ),>,
    parent_query: Query<Option<&FacingDirection>>,
) {
    for (
        child_of,
        mut transform,
        offset,
        offset_look_up_down, offset_look_down, offset_look_up,
        offset_look_sideways, offset_look_left, offset_look_right,
    ) in query.iter_mut() {
        if let Ok(direction) = parent_query.get(child_of.parent()) {
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
            transform.translation = total_offset / 1.;
        }
    }
}

#[allow(unused_parens, )]
pub fn replace_string_ids_by_entities(
    mut cmd: Commands,
    mut query: Query<(Entity, &SpriteDatasChildrenStringIds), (Added<SpriteDatasChildrenStringIds>)>,
    map: Res<SpriteDataIdEntityMap>,
) {
    for (ent, string_ids) in query.iter_mut() {
        let mut entities: Vec<Entity> = Vec::new();
        for id in &string_ids.0 {
            if let Some(sprite_ent) = map.get_entity(id) {
                info!("Replacing string id '{}' with entity {:?}", id, sprite_ent);
                entities.push(sprite_ent);
            } else {
                warn!("SpriteDataIdEntityMap does not contain entity for id: {}", id);
            }
        }
        if !entities.is_empty() {
            cmd.entity(ent).insert(SpriteDatasChildrenRefs(entities));
        }
        cmd.entity(ent).remove::<SpriteDatasChildrenStringIds>();
    }
}


#[allow(unused_parens)]
pub fn add_spritechildren_and_comps(
    mut cmd: Commands,
    aserver: Res<AssetServer>,
    mut atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut target_ent_query: Query<(Entity, &SpriteDatasChildrenRefs), Without<SpriteDataId>>,
    spritedatas_query: Query<(
        &ImgPathHolder,
        &AtlasLayoutData,
        &OtherCompsToBuild,
        Option<&SpriteDatasChildrenRefs>,
        &SpriteDataId,
    ), With<SpriteDataId>>,
) {
    for (father_to_sprite, SpriteDatasChildrenRefs(to_build)) in target_ent_query.iter_mut() {
        for (sprite_to_insert_ent) in to_build.iter() {
            if let Ok((
                img_path_holder, atlas_layout_data, comps_to_build, children_refs, sprite_data_id
            )) 
            = spritedatas_query.get(*sprite_to_insert_ent) {
                info !("Building sprite with spritedata {}", sprite_data_id.id());
                let spritesheet_size= atlas_layout_data.spritesheet_size;
                let frame_size = atlas_layout_data.frame_size;

                let atlas = TextureAtlas {
                    layout: atlas_layouts.add(Spritesheet::new(spritesheet_size.x as usize, spritesheet_size.y as usize).atlas_layout(frame_size.x, frame_size.y)),
                    ..default()
                };

                let image = aserver.load(format!("texture/{}", &img_path_holder.0));

                let child_sprite = cmd.spawn((
                    ChildOf(father_to_sprite),
                    Sprite::from_atlas_image(image, atlas),
                )).id();

                if let Some(children_refs) = children_refs {
                    cmd.entity(child_sprite).insert(children_refs.clone());
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

