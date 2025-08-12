#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
use bevy::ecs::entity_disabling::Disabled;
use game_common::game_common_components::FacingDirection;
use sprite_animation_shared::sprite_animation_shared::*;

use crate::sprite_components::*;



#[allow(unused_parens)]
pub fn apply_scales(
    mut sprite_que: Query<(&SpriteHolderRef, &mut Sprite, &SpriteConfigRef, &mut Transform,
        Option<&Scale2D>, Option<&ScaleLookUpDown>, Option<&ScaleSideways>,
    ),>, 
    sprite_config_query: Query<(Option<&FlipHorizIfDir>, &Scale2D, &ScaleLookUpDown, &ScaleSideways,),
    (With<SpriteConfig>, Or<(With<Disabled>, Without<Disabled>)>)>, 
    baseholder_query: Query<&FacingDirection>, 
) {
    for (
        spriteholder, mut sprite, &SpriteConfigRef(spritecfg_ent), 
        mut transform, scale, scale_look_up_down, scale_look_sideways,
    ) in sprite_que.iter_mut() {
        let mut total_scale = scale.copied().unwrap_or_default();

        if let Ok((ref_flip_horiz_if_dir, &ref_scale, &ref_scale_updown, &ref_scale_sideways)) = sprite_config_query.get(spritecfg_ent) {
            total_scale *= ref_scale;
        
            if let Ok(base_direction) = baseholder_query.get(spriteholder.base) {
    
                match base_direction {
                    FacingDirection::Left => {
                        total_scale *= ref_scale_sideways * scale_look_sideways.copied().unwrap_or_default();
                        
                        if let Some(&flip_horiz) = ref_flip_horiz_if_dir {
                            sprite.flip_x = match flip_horiz {
                                FlipHorizIfDir::Left => true, _ => true,
                            };
                        }
                    },
                    FacingDirection::Right => {
                        total_scale *= ref_scale_sideways * scale_look_sideways.copied().unwrap_or_default();

                        if let Some(flip_horiz) = ref_flip_horiz_if_dir {
                            sprite.flip_x = match flip_horiz {
                                FlipHorizIfDir::Left => false, _ => true,
                            };
                        }
                    },
                    FacingDirection::Up => {
                        total_scale *= ref_scale_updown * scale_look_up_down.copied().unwrap_or_default();
                        if let Some(flip_horiz) = ref_flip_horiz_if_dir {
                            sprite.flip_x = match flip_horiz {
                                FlipHorizIfDir::Any => true, _ => false,
                            };
                        }
                    },
                    FacingDirection::Down => {
                        total_scale *= ref_scale_updown * scale_look_up_down.copied().unwrap_or_default();
                        if let Some(flip_horiz) = ref_flip_horiz_if_dir {
                            sprite.flip_x = match flip_horiz {
                                FlipHorizIfDir::Any => true, _ => false,
                            };
                        }
                    },
                }
            }
        }
        let total_scale_vec2 = total_scale.as_vec2();
        if total_scale_vec2.x == 0.0 || total_scale_vec2.y == 0.0 {
            warn!(target: "sprite_animation", "total_scale is zero for sprite entity");
        }
        transform.scale.x = total_scale_vec2.x;
        transform.scale.y = total_scale_vec2.y;
    }
}

#[allow(unused_parens, )]
pub fn apply_offsets(
    mut sprite_que: Query<(
        &SpriteHolderRef, &ChildOf,
        &SpriteConfigRef,
        &mut Transform,
        Option<&Offset2D>, 
    ),>,
    sprite_config_query: Query<(
        &Categories,
        &Offset2D,
        &OffsetSideways,
        &OffsetUpDown, &OffsetUp, &OffsetDown, 
        &OffsetForChildren,
    ),(With<SpriteConfig>, Or<(With<Disabled>, Without<Disabled>)>)>, 
    parent_sprite_query: Query<&SpriteConfigRef>,
    base_query: Query<&FacingDirection>,
) {
    for (
        &baseholder, child_of, &SpriteConfigRef(sprite_config), mut transform, 
        offset, 
    ) in sprite_que.iter_mut() {

        let mut total_offset = offset.cloned().unwrap_or_default();

        let Ok((my_cats, &offset, &offset_sideways, &offset_updown, &offset_up, &offset_down, &_)) = sprite_config_query.get(sprite_config) 
        else {
            error!(target: "sprite_animation", "Failed to get sprite config for entity {:?}", sprite_config);
            continue;
        };

        total_offset += offset;

        if let Ok(direction) = base_query.get(baseholder.base) {
            match direction {
                FacingDirection::Left => {
                    total_offset += offset_sideways;
                },
                FacingDirection::Right => {
                    total_offset += offset_sideways;
                },
                FacingDirection::Up => {
                    total_offset += offset_updown;
                    total_offset += offset_up;
                },
                FacingDirection::Down => {
                    total_offset += offset_updown;
                    total_offset += offset_down;
                }
            }
            if let Ok(SpriteConfigRef(ent)) = parent_sprite_query.get(child_of.parent()) {
                if let Ok((
                    _, _, _, _, _, _, offset_for_children
                )) = sprite_config_query.get(*ent) {
                    for (cat, &offset) in offset_for_children.0.iter() {
                        if my_cats.0.contains(cat) {
                            total_offset += offset;
                        }
                    }
                }
                
            }
            transform.translation.x = total_offset.0.x;
            transform.translation.y = total_offset.0.y;
        }
    }
}




