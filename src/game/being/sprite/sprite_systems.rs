#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
use bevy_spritesheet_animation::prelude::*;
use serde::{Deserialize, Serialize};

use crate::game::{being::{being_components::SpritesToInsert, sprite::{animation_resources::*, sprite_components::*}}, game_components::{Direction, ImgPathHolder}};

#[allow(unused_parens)]
pub fn init_sprites(
    mut cmd: Commands, 
    aserver: Res<AssetServer>,
    race_seris: ResMut<Assets<SpriteDataSeri>>,
    mut map: ResMut<IdSpriteDataEntityMap>,

) {
        
    let human_body0 = cmd.spawn((
        SpriteDataId::new("human_body0"),
        DefaultBodyBundle::new("being/body/human_male.png"),
    )).id();

    let human_head0 = cmd.spawn((
        SpriteDataId::new("human_head0"),
        DefaultHeadBundle::new("being/head/human/0.png"),
    )).id();
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
pub fn add_spritechildren_and_comps(
    mut cmd: Commands,
    mut query: Query<(Entity, &SpritesToInsert)>,
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

    ), With<SpriteDataId>>,
) {
    for (ent, sprites_to_insert) in query.iter_mut() {
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
            )) = spritedatas_query.get(*sprite_to_insert_ent) {
                let spritesheet_size= atlas_layout_data.spritesheet_size;
                let frame_size = atlas_layout_data.frame_size;

                let atlas = TextureAtlas {
                    layout: atlas_layouts.add(Spritesheet::new(spritesheet_size.x as usize, spritesheet_size.y as usize).atlas_layout(frame_size.x, frame_size.y)),
                    ..default()
                };

                let image = aserver.load(&img_path_holder.0);

                let spritechild = cmd.spawn((
                    ChildOf(ent),
                    Sprite::from_atlas_image(image, atlas),
                )).id();

                cmd.entity(spritechild).insert(color_holder.cloned().unwrap_or_default());
                
                if let Some(offset) = offset {
                    cmd.entity(spritechild).insert(offset.clone());
                }
                if let Some(offset_looking_up_down) = offset_looking_up_down {
                    cmd.entity(spritechild).insert(offset_looking_up_down.clone());
                }
                if let Some(offset_looking_down) = offset_looking_down {
                    cmd.entity(spritechild).insert(offset_looking_down.clone());
                }
                if let Some(offset_looking_up) = offset_looking_up {
                    cmd.entity(spritechild).insert(offset_looking_up.clone());
                }
                if let Some(offset_looking_sideways) = offset_looking_sideways {
                    cmd.entity(spritechild).insert(offset_looking_sideways.clone());
                }
                if let Some(offset_looking_right) = offset_looking_right {
                    cmd.entity(spritechild).insert(offset_looking_right.clone());
                }
                if let Some(offset_looking_left) = offset_looking_left {
                    cmd.entity(spritechild).insert(offset_looking_left.clone());
                }
                if let Some(scale) = scale {
                    cmd.entity(spritechild).insert(scale.clone());
                }
                if let Some(scale_looking_up_down) = scale_looking_up_down {
                    cmd.entity(spritechild).insert(scale_looking_up_down.clone());
                }
                if let Some(scale_looking_sideways) = scale_looking_sideways {
                    cmd.entity(spritechild).insert(scale_looking_sideways.clone());
                }


            }
        }
        cmd.entity(ent).remove::<SpritesToInsert>();
    }
}

