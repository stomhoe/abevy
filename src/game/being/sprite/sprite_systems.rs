#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
use bevy_spritesheet_animation::prelude::*;
use serde::{Deserialize, Serialize};

use crate::game::{being::{being_components::{SpriteDatasIdsToBuild, SpriteDatasToBuild}, sprite::{sprite_resources::*, sprite_components::*}}, game_components::{Direction, ImgPathHolder}};

#[allow(unused_parens)]
pub fn init_sprites(
    mut cmd: Commands, 
    mut seris_handles: ResMut<SpriteSerisHandles>,
    mut assets: ResMut<Assets<SpriteDataSeri>>,
    mut strid_ent_map: ResMut<SpriteDataIdEntityMap>,
) {
    let handles_vec = std::mem::take(&mut seris_handles.handles);
    
    for handle in handles_vec {
        strid_ent_map.new_spritedata_from_seri(&mut cmd, handle, &mut assets);
    }
        
    // let human_body0 = cmd.spawn((
    //     SpriteDataId::new("human_body0"),
    //     DefaultBodyBundle::new("being/body/human_male.png"),
    // )).id();

    // let human_head0 = cmd.spawn((
    //     SpriteDataId::new("human_head0"),
    //     DefaultHeadBundle::new("being/head/human/0.png"),
    // )).id();
} 

pub fn apply_offsets_and_scales(
    mut query: Query<(
        &ChildOf,
        &mut Sprite,
        &mut Transform,
        Option<&FlipHorizIfDir>,
        Option<&Offset>,
        Option<&OffsetLookUpDown>,
        Option<&OffsetLookDown>,
        Option<&OffsetLookUp>,
        Option<&OffsetLookSideways>,
        Option<&OffsetLookRight>,
        Option<&OffsetLookLeft>,
        Option<&Scale>,
        Option<&ScaleLookUpDown>,
        Option<&ScaleLookSideWays>,
    ), 
    >,
    parent_query: Query<Option<&Direction>>,
) {
    for (
        child_of,
        mut sprite,
        mut transform,
        flip_horiz_if_dir,
        offset,
        offset_look_up_down,
        offset_look_down,
        offset_look_up,
        offset_look_sideways,
        offset_look_right,
        offset_look_left,
        scale,
        scale_look_up_down,
        scale_look_sideways,
    ) in query.iter_mut() {
        if let Ok(direction) = parent_query.get(child_of.parent()) {
            let mut total_offset: Vec3 = Vec3::ZERO;
            if let Some(dir) = direction {
                match dir {
                    Direction::Left => {
                        if let Some(scale_look_sideways) = scale_look_sideways {
                            transform.scale = scale_look_sideways.0.extend(0.0);
                        }
                        if let Some(offset_look_sideways) = offset_look_sideways {
                            total_offset += offset_look_sideways.0.extend(0.0);
                        }
                        if let Some(offset_look_left) = offset_look_left {
                            total_offset += offset_look_left.0.extend(0.0);
                        }
                        if let Some(flip_horiz) = flip_horiz_if_dir {
                            sprite.flip_x = match flip_horiz {
                                FlipHorizIfDir::Any => true,
                                FlipHorizIfDir::Left => true,
                                FlipHorizIfDir::Right => false,
                            };
                        }
                    },
                    Direction::Right => {
                        if let Some(scale_look_sideways) = scale_look_sideways {
                            transform.scale = scale_look_sideways.0.extend(0.0);
                        }
                        if let Some(offset_look_sideways) = offset_look_sideways {
                            total_offset += offset_look_sideways.0.extend(0.0);
                        }
                        if let Some(offset_look_right) = offset_look_right {
                            total_offset += offset_look_right.0.extend(0.0);
                        }
                        if let Some(flip_horiz) = flip_horiz_if_dir {
                            sprite.flip_x = match flip_horiz {
                                FlipHorizIfDir::Any => true,
                                FlipHorizIfDir::Left => false,
                                FlipHorizIfDir::Right => true,
                            };
                        }
                       
                    },
                    Direction::Up => {
                        if let Some(scale_look_up_down) = scale_look_up_down {
                            transform.scale = scale_look_up_down.0.extend(0.0);
                        }
                        if let Some(offset_look_up_down) = offset_look_up_down {
                            total_offset += offset_look_up_down.0.extend(0.0);
                        }
                        if let Some(offset_look_up) = offset_look_up {
                            total_offset += offset_look_up.0.extend(0.0);
                        }
                        if let Some(flip_horiz) = flip_horiz_if_dir {
                            sprite.flip_x = match flip_horiz {
                                FlipHorizIfDir::Any => true,
                                _ => false,
                            };
                        }

                    },
                    Direction::Down => {
                        if let Some(scale_look_up_down) = scale_look_up_down {
                            transform.scale = scale_look_up_down.0.extend(0.0);
                        }
                        if let Some(offset_look_up_down) = offset_look_up_down {
                            total_offset += offset_look_up_down.0.extend(0.0);
                        }
                        if let Some(offset_look_down) = offset_look_down {
                            total_offset += offset_look_down.0.extend(0.0);
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
                transform.scale = scale.0.extend(0.0);
            }
            if let Some(offset) = offset {
                total_offset += offset.0;
            }
            
            transform.translation = total_offset;
        }
    }
}

#[allow(unused_parens)]
pub fn turn_spritedatasids_into_entities(mut cmd: Commands, query: Query<(Entity, &SpriteDatasIdsToBuild,),()>, map: Res<SpriteDataIdEntityMap>) {
    for (ent, spritedatas_to_build) in query.iter() {
        let mut spritedata_ents : Vec<Entity> = Vec::new();
        for id in spritedatas_to_build.ids() {
            if let Some(sprite_ent) = map.get_entity(id) {
                spritedata_ents.push(sprite_ent);
            } else {
                warn!("SpriteDataIdEntityMap does not contain entity for id: {}", id);
            }
        }
        if ! spritedata_ents.is_empty() {
            cmd.entity(ent).insert(SpriteDatasToBuild(spritedata_ents));
        }
        cmd.entity(ent).remove::<SpriteDatasIdsToBuild>();
    }
}


#[allow(unused_parens)]
pub fn add_spritechildren_and_comps(
    mut cmd: Commands,
    mut query: Query<(Entity, &SpriteDatasToBuild)>,
    aserver: Res<AssetServer>,
    mut atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    spritedatas_query: Query<(
        &AtlasLayoutData,
        &ImgPathHolder,
        Option<&ColorHolder>,
        Option<&Offset>,
        Option<&OffsetLookUpDown>,
        Option<&OffsetLookDown>,
        Option<&OffsetLookUp>,
        Option<&OffsetLookSideways>,
        Option<&OffsetLookRight>,
        Option<&OffsetLookLeft>,
        Option<&Scale>,
        Option<&ScaleLookUpDown>,
        Option<&ScaleLookSideWays>,
        Option<&SpriteDatasChildren>,
    ), With<SpriteDataId>>,
) {
    for (father_to_sprite, sprites_to_insert) in query.iter_mut() {
        for (sprite_to_insert_ent) in sprites_to_insert.0.iter() {
            if let Ok((
                atlas_layout_data,
                img_path_holder,
                color_holder,
                offset,
                offset_looking_up_down,
                offset_looking_down,
                offset_looking_up,
                offset_looking_sideways,
                offset_looking_right,
                offset_looking_left,
                scale,
                scale_looking_up_down,
                scale_looking_sideways,
                children_sprite_datas_ids,
            )) = spritedatas_query.get(*sprite_to_insert_ent) {
                let spritesheet_size= atlas_layout_data.spritesheet_size;
                let frame_size = atlas_layout_data.frame_size;

                let atlas = TextureAtlas {
                    layout: atlas_layouts.add(Spritesheet::new(spritesheet_size.x as usize, spritesheet_size.y as usize).atlas_layout(frame_size.x, frame_size.y)),
                    ..default()
                };

                let image = aserver.load(format!("textures/{}", &img_path_holder.0));

                let child_sprite = cmd.spawn((
                    ChildOf(father_to_sprite),
                    Sprite::from_atlas_image(image, atlas),
                )).id();

                cmd.entity(child_sprite).insert(color_holder.cloned().unwrap_or_default());
                
                if let Some(offset) = offset {
                    cmd.entity(child_sprite).insert(offset.clone());
                }
                if let Some(offset_looking_up_down) = offset_looking_up_down {
                    cmd.entity(child_sprite).insert(offset_looking_up_down.clone());
                }
                if let Some(offset_looking_down) = offset_looking_down {
                    cmd.entity(child_sprite).insert(offset_looking_down.clone());
                }
                if let Some(offset_looking_up) = offset_looking_up {
                    cmd.entity(child_sprite).insert(offset_looking_up.clone());
                }
                if let Some(offset_looking_sideways) = offset_looking_sideways {
                    cmd.entity(child_sprite).insert(offset_looking_sideways.clone());
                }
                if let Some(offset_looking_right) = offset_looking_right {
                    cmd.entity(child_sprite).insert(offset_looking_right.clone());
                }
                if let Some(offset_looking_left) = offset_looking_left {
                    cmd.entity(child_sprite).insert(offset_looking_left.clone());
                }
                if let Some(scale) = scale {
                    cmd.entity(child_sprite).insert(scale.clone());
                }
                if let Some(scale_looking_up_down) = scale_looking_up_down {
                    cmd.entity(child_sprite).insert(scale_looking_up_down.clone());
                }
                if let Some(scale_looking_sideways) = scale_looking_sideways {
                    cmd.entity(child_sprite).insert(scale_looking_sideways.clone());
                }
                if let Some(SpriteDatasChildren(children_sprite_datas_ids)) = children_sprite_datas_ids {

                    cmd.entity(child_sprite).insert(
                        SpriteDatasToBuild(children_sprite_datas_ids.to_vec())
                    );
                }
            }
        }
        cmd.entity(father_to_sprite).remove::<SpriteDatasToBuild>();
    }
}

pub fn replace_string_ids_by_entities(
    mut cmd: Commands,
    mut query: Query<(Entity, &SpriteDatasChildrenStringIds), ()>,
    map: Res<SpriteDataIdEntityMap>,
) {
    for (ent, string_ids) in query.iter_mut() {
        let mut entities: Vec<Entity> = Vec::new();
        for id in &string_ids.0 {
            if let Some(sprite_ent) = map.get_entity(id) {
                entities.push(sprite_ent);
            } else {
                warn!("SpriteDataIdEntityMap does not contain entity for id: {}", id);
            }
        }
        if ! entities.is_empty() {
            cmd.entity(ent).insert(SpriteDatasChildren(entities));
        }
        cmd.entity(ent).remove::<SpriteDatasChildrenStringIds>();
    }
}