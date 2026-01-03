use std::mem::take;

use bevy::{ecs::entity_disabling::Disabled, platform::collections::{HashMap, HashSet}, };
#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
use common::common_components::*;
use debug_unwraps::DebugUnwrapExt;
use game_common::game_common_components::*;
use sprite_animation_shared::{AnimationLibrary, AcAnimationProgresses, sprite_animation_shared::AnimationState };
use ::sprite_shared::{sprite_scale_offset::*, *};

use crate::{sprite_components::*, sprite_resources::*, };

#[allow(unused_parens)]
pub fn init_sprite_cfgs(
    mut cmd: Commands, map: Option<Res<SpriteCfgEntityMap>>,

    mut seris_handles: ResMut<SpriteSerisHandles>,
    mut assets: ResMut<Assets<SpriteConfigSeri>>,
    library: Res<AnimationLibrary>,
    holder: Single<Entity, With<SpriteConfigsHolder>>,
) {
    if map.is_some(){ return; }

    let mut map = SpriteCfgEntityMap::default();

    cmd.spawn(EguiSpriteHolder::default());

    let mut comps_to_insert = Vec::new();

    for handle in take(&mut seris_handles.handles) {
        let Some(mut seri) = assets.remove(&handle) else {continue;};

        debug!(target: "sprite_init", "Loading SpriteDataSeri from handle: {:?}", handle);
        
        let str_id = match StrId::new_with_result(seri.id, 3) {
            Ok(id) => id,
            Err(e) => {
                let err = BevyError::from(format!("Failed to create StrId for SpriteConfig: {}", e));
                    error!(target: "sprite_init", "{}", err);
                    continue;
                }
            };

        if let Ok(_existing_ent) = map.0.get(&str_id) {
            error!(target: "sprite_init", "Duplicate SpriteConfig StrId found: '{}', skipping duplicate.", str_id);
            continue;
        }
        let spritecfg_ent = cmd.spawn_empty().id();
        map.0.overwrite(str_id.clone(), spritecfg_ent);
        
        let visib = match seri.visibility {
            Some(0) => Visibility::Inherited,
            Some(1) => Visibility::Visible,
            Some(2) => Visibility::Hidden,
            Some(v) => {
                warn!(target: "sprite_init", "Invalid visibility value: {} for SpriteConfig '{}', falling back to inherited", v, str_id);
                Visibility::default()
            },
            None => Visibility::Inherited,
        };
        
        
        let mut offset4children_cats = OffsetForChildren::default();
        if let Some(offset4children) = seri.offset4children.as_mut() {
            for (cat, (offset_x, offset_y, direction)) in take(offset4children) {
                offset4children_cats.0.insert(Tag::new_truncated(cat), 
                (Offset2D::from((offset_x, offset_y)), AppliesOnSpriteDirection::from(direction)));
            }
        }
        comps_to_insert.push((spritecfg_ent, ( 
            str_id.clone(), 
            SpriteConfig,
            visib,
            offset4children_cats,
            EntityZero,
            ChildOf(holder.entity()),
        )));
        if let Some(tags) = seri.tags.as_ref() {
            if !tags.is_empty() {
                cmd.entity(spritecfg_ent).insert(TagHashSet::new(tags));
            }
        }
        if let Some(scale_2d) = seri.scale {
            cmd.entity(spritecfg_ent).insert(Scale2D::from(scale_2d));
        }
        if let Some(offset_2d) = seri.offset {
            cmd.entity(spritecfg_ent).insert(Offset2D::from(offset_2d));
        }
        if let Some(scale_look_up_down) = seri.scale_up_down {
            cmd.entity(spritecfg_ent).insert(ScaleLookUpDown::from(scale_look_up_down));
        }
        if let Some(scale_sideways) = seri.scale_sideways {
            cmd.entity(spritecfg_ent).insert(ScaleSideways::from(scale_sideways));
        }
        if let Some(offset_up_down) = seri.offset_up_down {
            cmd.entity(spritecfg_ent).insert(OffsetUpDown::from(offset_up_down));
        }
        if let Some(offset_down) = seri.offset_down {
            cmd.entity(spritecfg_ent).insert(OffsetDown::from(offset_down));
        }
        if let Some(offset_up) = seri.offset_up {
            cmd.entity(spritecfg_ent).insert(OffsetUp::from(offset_up));
        }
        if let Some(offset_sideways) = seri.offset_sideways {
            cmd.entity(spritecfg_ent).insert(OffsetSideways::from(offset_sideways));
        }

        if seri.name.trim().is_empty() {
            warn!(target: "sprite_init", "SpriteConfig name is empty for SpriteConfig '{}', using StrId as name", str_id);
            cmd.entity(spritecfg_ent).insert(DisplayName::new_trimmed(str_id.as_str()));
        } else {
            let disp_name = DisplayName::new_trimmed(seri.name);
            cmd.entity(spritecfg_ent).insert(disp_name);
        }
        //if seri.exclusive { comps_to_build.exclusive = Some(Exclusive); }

        if seri.directionable == Some(true) 
        { cmd.entity(spritecfg_ent).insert(Directionable); }

        if seri.movement_based == Some(true) 
        { cmd.entity(spritecfg_ent).insert(MovementBased); }

        if seri.grounding_based == Some(true) { cmd.entity(spritecfg_ent).insert(GroundingBased); }
        if let Some(parent_cat) = seri.parent_cat.as_ref().filter(|s| !s.trim().is_empty()) {
            let to_become_child = BecomeChildOfSpriteWithTag(Tag::new_truncated(parent_cat.trim()));
            cmd.entity(spritecfg_ent).insert(to_become_child);
        }

        if ! seri.mapped_anims.is_empty() {
            let mut anims_map = MappedAnimations::default();
            for (anim_type, anim_id) in seri.mapped_anims {
                let anim_type = AnimType::from_tuple(anim_type);
                let anim_id = StrId::new_truncated(anim_id);
                let Some(&anim_ent) = library.0.get(&anim_id) else {
                    error!(target: "sprite_init", "SpriteConfig {}: AnimationLibrary does not contain: {} ", str_id, anim_id);
                    continue;
                };
                anims_map.0.insert(anim_type, anim_ent);
                
            }
            if anims_map.0.is_empty() {
                error!(target: "sprite_init", "SpriteConfig '{}' animations map has no valid entries", str_id);
            }
            else {
               cmd.entity(spritecfg_ent).insert(anims_map);
            }
        }
        else {
            error!(target: "sprite_init", "SpriteConfig '{}' was given an empty animations map", str_id);
        }


        if let Some(children_sprites) = seri.children_sprites.as_ref() {
            if !children_sprites.is_empty() {
                let ids = SpriteConfigStrIds::new(children_sprites.clone());
                cmd.entity(spritecfg_ent).insert(ids);
            
            }
        }

        /*
        match seri.flip_horiz {
            1 => { cmd.entity(spritecfg_ent).insert(FlipHorizIfDir::Any); },
            2 => { cmd.entity(spritecfg_ent).insert(FlipHorizIfDir::Left); },
            3 => { cmd.entity(spritecfg_ent).insert(FlipHorizIfDir::Right); },
            _ => {},
        };
        */
        
    }
    cmd.insert_resource(map);
    cmd.insert_batch(comps_to_insert);  
} 


#[allow(unused_parens, )]
pub fn replace_string_ids_by_entities(
    mut cmd: Commands,
    query: Query<(Entity, &SpriteConfigStrIds, ), (Changed<SpriteConfigStrIds>,)>,
    map: Option<Res<SpriteCfgEntityMap>>,
) {
    let Some(map) = map else {
        if ! query.is_empty() {
            error!(target: "sprite_building", "SpriteCfgEntityMap not found, cannot replace string ids");
        }
        return;
    };

    for (ent, str_ids, ) in query.iter() {
        info!(target: "sprite_building", "Replacing string ids for entity {:?}", ent);
        let mut entities_to_build = HashSet::new();
        for id in str_ids.ids() {
            if let Ok(sprite_ent) = map.0.get(id) {
                info!(target: "sprite_building", "Replacing string id '{}' with entity {:?}", id, sprite_ent);
                entities_to_build.insert(sprite_ent);
            } else {
                error!(target: "sprite_building", "ekf SpriteConfigEntityMap does not contain entity for id: {}", id);
            }
        }
        if ! entities_to_build.is_empty() {
          
            cmd.entity(ent).try_insert(SpriteCfgsToBuild(entities_to_build));
        }
    }
}

#[allow(unused_parens)]
pub fn add_spritechildren_and_comps(//SOLO SERVER PA SYNQUEAR
    mut cmd: Commands,
    father_query: Query<(Entity, &SpriteCfgsToBuild, Option<&BaseHolderRef>,), 
    (Without<SpriteConfig>, Changed<SpriteCfgsToBuild>,)>,
    spritecfgs_query: Query<(&StrId, Option<&SpriteCfgsToBuild>), 
    (With<SpriteConfig>, Or<(With<Disabled>, Without<Disabled>)>)>,
    held_sprites_query: Query<&HeldSprites, Or<(With<Disabled>, Without<Disabled>)>>,
    sprite_config_ref_query: Query<&EntityZeroRef, Or<(With<Disabled>, Without<Disabled>)>>,
) {
    for (father_to_sprite, to_build, baseholder_ref,) in father_query.iter() {

        'sprite_cfg_for :for &spritecfg_ent in to_build.0.iter() {
            if let Ok((str_id, extra_to_build)) = spritecfgs_query.get(spritecfg_ent) {

                info!(target: "sprite_building", "Building sprite {}", str_id);

                let baseholder_ref = if let Some(baseholder_ref) = baseholder_ref {
                    baseholder_ref.clone()
                } else {
                    BaseHolderRef{ base: father_to_sprite }
                };
                if let Ok(held_sprites) = held_sprites_query.get(baseholder_ref.base) {
                    for &sprite_ent in held_sprites.entities() {
                        if let Ok(sprite_cfg_ref) = sprite_config_ref_query.get(sprite_ent) {
                            if sprite_cfg_ref.0 == spritecfg_ent {
                                warn!(target: "sprite_building", "SpriteConfig '{}' already present in HeldSprites of base holder {:?}, skipping.", str_id, baseholder_ref.base);
                                continue 'sprite_cfg_for;
                            }
                        }
                    }
                } 
                let sprite = cmd.spawn((
                    str_id.clone(),
                    EntityZeroRef(spritecfg_ent),
                    Visibility::default(),
                    Transform::default(),
                    AcAnimationProgresses::default(),

                    ChildOf(father_to_sprite),
                    Replicated,
                )).id();

                cmd.entity(sprite).try_insert(baseholder_ref);

                if let Some(extra_to_build) = extra_to_build {
                    cmd.entity(sprite).try_insert(extra_to_build.clone());
                    // NO HACE FALTA PONER UN SpriteCfgsBuiltSoFar EN ESTO PORQ LOS CHILDREN FALTANTES SE VAN A AUTOCONSTRUIR CON LA PRESENCIA DE ESTE
                }
 
                // if let Some(excl) = &comps_to_build.exclusive {
                //     cmd.entity(child_sprite).insert(excl.clone());
                // }
            } else{
                warn!(target: "sprite_building", "query does not contain entity for: {}", spritecfg_ent);
            }
        }
        //cmd.entity(father_to_sprite).remove::<SpriteCfgsToBuild>();
        //NO HACER ESO PORQ HACE FALTA PARA LA REPLICACIÓN ^^
    }
}

#[allow(unused_parens)]
pub fn become_child_of_sprite_with_tag(
    mut cmd: Commands,
    new_sprites: Query<(Entity, &BaseHolderRef, &EntityZeroRef), (Without<SpriteConfig>, Changed<EntityZeroRef>,)>,
    sprite_holder: Query<&HeldSprites>,
    other_sprites: Query<(Entity, &EntityZeroRef), (Without<SpriteConfig>, )>,
    becomes: Query<(&BecomeChildOfSpriteWithTag), (Or<(With<Disabled>, Without<Disabled>)>)>,
    other_cats: Query<&TagHashSet, (Or<(With<Disabled>, Without<Disabled>)>)>,
) {
    for (new_ent, &sprite_holder_ref, &new_sprite_cfg_ref) in new_sprites.iter(){
        if let Ok(becomes_child_of_sprite_with_cat) = becomes.get(new_sprite_cfg_ref.0) {unsafe {
            let held_sprites = sprite_holder.get(sprite_holder_ref.base).debug_expect_unchecked("SpriteHolderRef should have a HeldSprites component");

            for (other_ent, o_spritecfg_ref) in other_sprites.iter_many(held_sprites.entities()) {
                if new_ent == other_ent { continue; }

                let other_cats = match other_cats.get(o_spritecfg_ref.0) {
                    Ok(cats) => cats,
                    Err(e) => {
                        break;
                    },
                };
                if other_cats.0.contains(&becomes_child_of_sprite_with_cat.0) {
                    debug!(target: "sprite_building", "Adding ChildOfTag to entity {:?} with id: {}", new_ent, becomes_child_of_sprite_with_cat.0);
                    cmd.entity(new_ent).try_insert(ChildOf(other_ent));
                    break;
                }
            }
        }}
    }
}

