#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
use bevy::ecs::entity_disabling::Disabled;
use game_common::game_common_components::{Categories, EntityZero, FacingDirection};
use player::player_components::*;
use ::sprite_shared::{sprite_scale_offset::*, *};

use crate::{sprite_components::*, sprite_resources::SpriteCfgEntityMap, };

#[allow(unused_parens)]
pub fn apply_scales(
    mut sprite_que: Query<(&BaseHolderRef, &mut Sprite, &SpriteConfigRef, &mut Transform,
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
                    FacingDirection::West => {
                        total_scale *= ref_scale_sideways * scale_look_sideways.copied().unwrap_or_default();
                        
                        if let Some(&flip_horiz) = ref_flip_horiz_if_dir {
                            sprite.flip_x = match flip_horiz {
                                FlipHorizIfDir::Left => true, _ => true,
                            };
                        }
                    },
                    FacingDirection::East => {
                        total_scale *= ref_scale_sideways * scale_look_sideways.copied().unwrap_or_default();

                        if let Some(flip_horiz) = ref_flip_horiz_if_dir {
                            sprite.flip_x = match flip_horiz {
                                FlipHorizIfDir::Left => false, _ => true,
                            };
                        }
                    },
                    FacingDirection::North => {
                        total_scale *= ref_scale_updown * scale_look_up_down.copied().unwrap_or_default();
                        if let Some(flip_horiz) = ref_flip_horiz_if_dir {
                            sprite.flip_x = match flip_horiz {
                                FlipHorizIfDir::Any => true, _ => false,
                            };
                        }
                    },
                    FacingDirection::South => {
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
            warn!("total_scale is zero for sprite entity");
        }
        transform.scale.x = total_scale_vec2.x;
        transform.scale.y = total_scale_vec2.y;
    }
}

#[allow(unused_parens, )]
pub fn apply_offsets(
    mut sprite_query: Query<(
        &BaseHolderRef, 
        &ChildOf,
        Option<&SpriteConfigRef>,
        &mut Transform,
        Option<&Offset2D>, 
    ), (Without<EntityZero>, )>,
    sprite_config_query: Query<(
        &Categories,
        Option<&Offset2D>,
        Option<&OffsetSideways>,
        Option<&OffsetUpDown>, Option<&OffsetUp>, Option<&OffsetDown>, 
        Option<&OffsetForChildren>,
    ),(With<SpriteConfig>, Or<(With<Disabled>, Without<Disabled>)>)>, 
    parent_sprite_query: Query<&SpriteConfigRef>,
    base_query: Query<&FacingDirection>,
) {
    for (
        baseholder, child_of, sprite_config_ref, mut transform, 
        offset, 
    ) in sprite_query.iter_mut() {

        let mut total_offset = offset.cloned().unwrap_or_default();

        if let Some(SpriteConfigRef(sprite_config)) = sprite_config_ref.cloned() {
            let Ok((my_cats, offset, offset_sideways, offset_updown, offset_up, offset_down, _offset4children)) = sprite_config_query.get(sprite_config) 
            else {
                error!("Failed to get sprite config for entity {:?}", sprite_config);
                transform.translation.x = total_offset.0.x; transform.translation.y = total_offset.0.y;
                continue;
            };


            total_offset += offset.cloned().unwrap_or_default();

            if let Ok(direction) = base_query.get(baseholder.base) {
                match direction {
                    FacingDirection::West => {
                        total_offset += offset_sideways.cloned().unwrap_or_default();
                    },
                    FacingDirection::East => {
                        total_offset += offset_sideways.cloned().unwrap_or_default();
                    },
                    FacingDirection::North => {
                        total_offset += offset_updown.cloned().unwrap_or_default();
                        total_offset += offset_up.cloned().unwrap_or_default();
                    },
                    FacingDirection::South => {
                        total_offset += offset_updown.cloned().unwrap_or_default();
                        total_offset += offset_down.cloned().unwrap_or_default();
                    }
                }
                if let Ok(SpriteConfigRef(ent)) = parent_sprite_query.get(child_of.parent()) {
                    if let Ok((//TA BIEN
                        _, _, _, _, _, _, offset_for_children
                    )) = sprite_config_query.get(*ent) {
                        if let Some(offset_for_children) = offset_for_children {
                            for (offset_cat, &(offset, dir)) in offset_for_children.0.iter() {
                                if my_cats.0.contains(offset_cat) {
                                    total_offset += offset;
                                }
                            }
                        }
                    }
                }
            }
        }
        transform.translation.x = total_offset.0.x; transform.translation.y = total_offset.0.y;
    }
}


#[allow(unused_parens)]
pub fn disable_children_sprites_of_disabled(mut cmd: Commands, 
    ezero_bases: Query<(&HeldSprites),(With<EntityZero>, Added<Disabled>)>,
    non_ezero_bases: Query<(&HeldSprites),(Without<EntityZero>,)>,
    mut removed: RemovedComponents<Disabled>,
) {
    for (held_sprites) in ezero_bases.iter() {
        for &sprite_ent in held_sprites.entities() {
            cmd.entity(sprite_ent).try_insert(Disabled);
            trace!(target:"sprite_systems", "Disabled sprite entity {:?} as its base entity was disabled", sprite_ent);
        }
    }
    for ent in removed.read() {
        if let Ok((held_sprites)) = non_ezero_bases.get(ent) {
            for &sprite_ent in held_sprites.entities() {
                cmd.entity(sprite_ent).try_remove::<Disabled>();
                trace!(target:"sprite_systems","Re-enabled sprite entity {:?} as its base entity {:?} was re-enabled", sprite_ent, ent);
            }
        }
    }
}

