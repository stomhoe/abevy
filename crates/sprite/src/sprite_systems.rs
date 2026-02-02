#[allow(unused_imports)] use bevy::prelude::*;
use bevy_ecs_tilemap::{DrawTilemap, anchor::TilemapAnchor};
#[allow(unused_imports)] use bevy_replicon::prelude::*;
use bevy::ecs::entity_disabling::Disabled;
use common::{common_components::AnyDisabling, common_tag_components::TagSet};
use game_common::game_common_components::{EntityZero, EntityZeroRef, Direction};
use ::sprite_shared::{sprite_scale_offset::*, *};

use crate::sprite_components::*;


#[allow(unused_parens)]
pub fn apply_scales(
    mut sprite_que: Query<(&BaseHolderRef, &mut Sprite, &EntityZeroRef, &mut Transform,
        Option<&Scale2D>, Option<&ScaleLookUpDown>, Option<&ScaleSideways>,
    ),>, 
    sprite_config_query: Query<(Option<&FlipHorizIfDir>, &Scale2D, &ScaleLookUpDown, &ScaleSideways,),
    (AnyDisabling)>, 
    baseholder_query: Query<&Direction>, 
) {
    for (
        spriteholder, mut sprite, &EntityZeroRef(spritecfg_ent), 
        mut transform, scale, scale_look_up_down, scale_look_sideways,
    ) in sprite_que.iter_mut() {
        let mut total_scale = scale.copied().unwrap_or_default();

        if let Ok((ref_flip_horiz_if_dir, &ref_scale, &ref_scale_updown, &ref_scale_sideways)) = sprite_config_query.get(spritecfg_ent) {
            total_scale *= ref_scale;
        
            if let Ok(base_direction) = baseholder_query.get(spriteholder.base) {
    
                match base_direction {
                    Direction::West => {
                        total_scale *= ref_scale_sideways * scale_look_sideways.copied().unwrap_or_default();
                        
                        if let Some(&flip_horiz) = ref_flip_horiz_if_dir {
                            sprite.flip_x = match flip_horiz {
                                FlipHorizIfDir::Left => true, _ => true,
                            };
                        }
                    },
                    Direction::East => {
                        total_scale *= ref_scale_sideways * scale_look_sideways.copied().unwrap_or_default();

                        if let Some(flip_horiz) = ref_flip_horiz_if_dir {
                            sprite.flip_x = match flip_horiz {
                                FlipHorizIfDir::Left => false, _ => true,
                            };
                        }
                    },
                    Direction::North => {
                        total_scale *= ref_scale_updown * scale_look_up_down.copied().unwrap_or_default();
                        if let Some(flip_horiz) = ref_flip_horiz_if_dir {
                            sprite.flip_x = match flip_horiz {
                                FlipHorizIfDir::Any => true, _ => false,
                            };
                        }
                    },
                    Direction::South => {
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
    mut cmd: Commands,
    mut sprite_query: Query<(
        &mut Transform,
        Entity,
        &BaseHolderRef, 
        &ChildOf,
        Option<&EntityZeroRef>,
        Option<&Offset2D>, 
        Has<SpriteConfigNotFound>,
    ), (AnyDisabling, Without<EntityZero>, )>,
    sprite_config_query: Query<(
        Option<&TagSet>,
        Option<&Offset2D>,
        Option<&OffsetSideways>,
        Option<&OffsetUpDown>, Option<&OffsetUp>, Option<&OffsetDown>, 
        Option<&OffsetForChildren>,
    ),(AnyDisabling)>, 
    parent_sprite_query: Query<&EntityZeroRef>,
    base_query: Query<&Direction>,
) {
    for (
        mut transform, sprite_entity, baseholder, child_of, sprite_config_ref, 
        offset, has_sprite_config_not_found
    ) in sprite_query.iter_mut() {

        let mut total_offset = Offset2D::default();

        if let Some(EntityZeroRef(sprite_config)) = sprite_config_ref.cloned() {
            let Ok((my_cats, offset, offset_sideways, offset_updown, offset_up, offset_down, _offset4children)) = sprite_config_query.get(sprite_config) 
            else {
                if !has_sprite_config_not_found {
                    error!("Failed to get sprite config for entity {:?}", sprite_config);
                    cmd.entity(sprite_entity).try_insert(SpriteConfigNotFound);
                }
                transform.translation.x = total_offset.0.x; transform.translation.y = total_offset.0.y;
                continue;
            };
            if has_sprite_config_not_found {
                cmd.entity(sprite_entity).try_remove::<SpriteConfigNotFound>();
            }

            total_offset += offset.cloned().unwrap_or_default();

            if let Ok(direction) = base_query.get(baseholder.base) {
                match direction {
                    Direction::West => {
                        total_offset += offset_sideways.cloned().unwrap_or_default();
                    },
                    Direction::East => {
                        total_offset += offset_sideways.cloned().unwrap_or_default();
                    },
                    Direction::North => {
                        total_offset += offset_updown.cloned().unwrap_or_default();
                        total_offset += offset_up.cloned().unwrap_or_default();
                    },
                    Direction::South => {
                        total_offset += offset_updown.cloned().unwrap_or_default();
                        total_offset += offset_down.cloned().unwrap_or_default();
                    }
                }

                if let Some(my_cats) = my_cats {
                    if let Ok(EntityZeroRef(ent)) = parent_sprite_query.get(child_of.parent()) {
                        if let Ok((//TA BIEN DE ESTA FORMA REBUSCADA, OffsetAsChild NO SIRVE POR EL ORDEN DE APLICACION INDETERMINISTA. ES MUCHO MAS BUG PRONE CON CHANGE DETECTION
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
        } else{
            total_offset += offset.cloned().unwrap_or_default();
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
    let mut disableds = Vec::new();
    for (held_sprites) in ezero_bases.iter() {
        for &sprite_ent in held_sprites.entities() {
            disableds.push((sprite_ent, Disabled));
            //trace!(target: "sprite_systems", "Disabled sprite entity {:?} as its base entity was disabled", sprite_ent);
        }
    }
    for ent in removed.read() {
        if let Ok((held_sprites)) = non_ezero_bases.get(ent) {
            for &sprite_ent in held_sprites.entities() {
                cmd.entity(sprite_ent).try_remove::<Disabled>();
                //trace!(target: "sprite_systems","Re-enabled sprite entity {:?} as its base entity {:?} was re-enabled", sprite_ent, ent);
            }
        }
    }
    cmd.try_insert_batch(disableds);
}
#[allow(unused_parens)]
pub fn add_sprites_to_holder(mut cmd: Commands, 
    holder: Single<(Entity, ), (With<EguiWorldSprites>)>, 

    query: Query<(Entity, ),(With<Sprite>, Without<EguiSpriteHolderReference>, Without<Disabled>)>,
    added_disabled: Query<(Entity, ),(With<Sprite>, With<EguiSpriteHolderReference>, Added<Disabled>)>,
) {
    for (ent, ) in query.iter() {
        cmd.entity(ent).try_insert(EguiSpriteHolderReference(holder.0));
    }
    for (ent, ) in added_disabled.iter() {
        cmd.entity(ent).try_remove::<EguiSpriteHolderReference>();
    }
}

#[allow(unused_parens, )]
pub fn z_sort_system(
    
    mut query: Query<(Entity, &mut Transform, &GlobalTransform, Option<&YSortOrigin>, 
        AnyOf<(&AcZ, &EntityZeroRef)>, Has<TilemapAnchor>, &ChildOf, ), 
        (Or<(Changed<EntityZeroRef>, Changed<GlobalTransform>, Changed<YSortOrigin>, Changed<AcZ>, Changed<ChildOf>,)>, 
        Or<(With<Sprite>, With<TilemapAnchor>, )>)>,
        
    parent_sprite_query: Query<&Sprite, (AnyDisabling,)>,
    
    ezero_query: Query<(&AcZ, Option<&YSortOrigin>), (AnyDisabling,)>,

    mut mw_draw_tmap: MessageWriter<DrawTilemap>,

) {//TODO MEJORAR
    let mut to_draw = Vec::new();

    for (ent, mut transform, global_transform, ysort_origin, (maybe_z_index, ezero_ref), is_tilemap, child_of) in query.iter_mut() {

        let (maybe_z_index, maybe_ysort_origin) = if let Some(ezero_ref) = ezero_ref
            && let Ok((ezero_z_index, ezero_ysort_origin)) = ezero_query.get(ezero_ref.0)
        {
            (Some(ezero_z_index.clone()), ezero_ysort_origin.cloned())
        } else if let Some(z_index) = maybe_z_index.cloned() {
            (Some(z_index), ysort_origin.cloned())
        } else {
            (None, None)
        };

        let y_pos = global_transform.translation().y - maybe_ysort_origin.unwrap_or_default().0;

        let use_y_sort = (maybe_ysort_origin.is_some() && parent_sprite_query.get(child_of.0).is_err()) as i32 as f32;

        let target_z = maybe_z_index.unwrap_or_default().used_float() - use_y_sort * y_pos * YSortOrigin::Y_SORT_DIV;

        if (transform.translation.z - target_z).abs() > f32::EPSILON {//NO TOCAR
            transform.translation.z = target_z;
            trace!(target: "zlevel", "Set entity {:?} to z {}", ent, target_z);
            if is_tilemap {
                to_draw.push(DrawTilemap(ent));
            }
        }
    }

    mw_draw_tmap.write_batch(to_draw);
}
