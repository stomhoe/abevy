use std::mem::take;

#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
use common::{common_components::*, common_tag_components::TagSet};
use game_common::game_common_components::*;
use sprite_animation_shared::AnimationLibrary ;
use ::sprite_shared::{sprite_scale_offset::*, *};

use crate::{sprite_components::*, sprite_resources::*, };

#[allow(unused_parens)]
pub fn init_sprite_cfgs(
    mut cmd: Commands, 
    mut scs_map: ResMut<SpriteCfgEntityMap>,
    mut seris_handles: ResMut<SpriteSerisHandles>,
    mut assets: ResMut<Assets<SpriteConfigSeri>>,
    library: Res<AnimationLibrary>,
    scs_holder: Query<Entity, With<SpriteConfigsHolder>>,
    world_sprites_holder: Query<Entity, With<EguiWorldSprites>>,
) {
    info!(target: "sprite_init", "Initializing Sprite Configs...");
    if !scs_map.0.is_empty(){ return; }
    
    if world_sprites_holder.is_empty() {
        cmd.spawn(EguiWorldSprites::default());
    }

    let scs_holder = scs_holder.single().unwrap();
    
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
        
        if let Ok(_existing_ent) = scs_map.0.get_cloned(&str_id) {
            error!(target: "sprite_init", "Duplicate SpriteConfig StrId found: '{}', skipping duplicate.", str_id);
            continue;
        }
        let spritecfg_ent = cmd.spawn_empty().id();
        scs_map.0.overwrite(str_id.clone(), spritecfg_ent);
        
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
                offset4children_cats.0.insert(Tag::trunc(cat), 
                (Offset2D::from((offset_x, offset_y)), AppliesOnSpriteDirection::from(direction)));
            }
        }
        comps_to_insert.push((spritecfg_ent, ( 
            str_id.clone(), 
            SpriteConfig,
            visib,
            offset4children_cats,
            EntityZero,
            ChildOf(scs_holder),
        )));
        if let Some(tags) = seri.tags.as_ref() {
            if !tags.is_empty() {
                cmd.entity(spritecfg_ent).insert(TagSet::new(tags));
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
            let to_become_child = BecomeChildOfSpriteWithTag(Tag::trunc(parent_cat.trim()));
            cmd.entity(spritecfg_ent).insert(to_become_child);
        }
        
        if ! seri.mapped_anims.is_empty() {
            let mut anims_map = MappedAnimations::default();
            for (anim_type, anim_id) in seri.mapped_anims {
                let anim_type = AnimType::from_tuple(anim_type);
                let anim_id = StrId::trunc(anim_id);
                let Ok(&anim_ent) = library.0.get(&anim_id) else {
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
    cmd.try_insert_batch(comps_to_insert);  
} 


#[allow(unused_parens)]
pub fn remove_spriteconfig_from_entimap_on_despawn(
    trigger: On<Despawn, SpriteConfig>,
    query: Query<(&StrId),(AnyDisabling)>,
    mut map: ResMut<SpriteCfgEntityMap>,

) {
    if let Ok(str_id) = query.get(trigger.entity) {
        if let Ok(found_entity) = map.0.get_cloned(str_id) {
            if found_entity == trigger.entity {
                map.0.remove(str_id.as_str());
            }
        }
    }
}