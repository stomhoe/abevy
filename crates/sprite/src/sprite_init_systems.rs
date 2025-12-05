use std::mem::take;

use bevy::{ecs::entity_disabling::Disabled, platform::collections::{HashMap, HashSet}, render::sync_world::SyncToRenderWorld};
#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
use bevy_replicon::shared::server_entity_map::ServerEntityMap;
use bevy_replicon_renet::renet::RenetClient;
use bevy_spritesheet_animation::prelude::Animation;
use common::common_components::{AssetScoped, Category, DisplayName, EntityPrefix, ImageHolder, ImagePathHolder, StrId};
use debug_unwraps::DebugUnwrapExt;
use game_common::game_common_components::{Categories, Directionable, MyZ};
use sprite_animation_shared::{AnimationLibrary, sprite_animation_shared::AnimationState};

use crate::{sprite_components::*, sprite_resources::*, sprite_scale_offset_components::*};



#[derive(Component, Debug, Default, )]
#[require(AssetScoped, EntityPrefix::new_truncated("SpriteConfigs"), )]
struct SpriteConfigsHolder;

#[allow(unused_parens)]
pub fn init_sprite_cfgs(
    mut cmd: Commands, map: Option<Res<SpriteCfgEntityMap>>,

    mut seris_handles: ResMut<SpriteSerisHandles>,
    mut assets: ResMut<Assets<SpriteConfigSeri>>,
    library: Res<AnimationLibrary>,
) {
    if map.is_some(){ return; }


    cmd.init_resource::<SpriteCfgEntityMap>();
    let holder = cmd.spawn((SpriteConfigsHolder, )).id();


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
                offset4children_cats.0.insert(Category::new_truncated(cat), 
                (Offset2D::from((offset_x, offset_y)), AppliesOnSpriteDirection::from(direction)));
            }
        }
        
        let spritecfg_ent = cmd.spawn((
            str_id.clone(), 
            SpriteConfig,
            Categories::new(seri.categories.unwrap_or_default()),
            visib,
            offset4children_cats,
            //poner esto de vuelta por si se quieren reescalar las animations globalmente
            /*
            Scale2D::from(seri.scale.unwrap_or([1.0, 1.0])),
            ScaleLookUpDown::from(seri.scale_up_down.unwrap_or([1.0, 1.0])),
            ScaleSideways::from(seri.scale_sideways.unwrap_or([1.0, 1.0])),
            Offset2D::from(seri.offset),
            OffsetUpDown::from(seri.offset_up_down.unwrap_or_default()),
            OffsetDown::from(seri.offset_down.unwrap_or_default()),
            OffsetUp::from(seri.offset_up.unwrap_or_default()),
            OffsetSideways::from(seri.offset_sideways.unwrap_or_default()),
            */
        )).insert((
            ChildOf(holder),
        )).id();

        if seri.name.trim().is_empty() {
            warn!(target: "sprite_init", "SpriteConfig name is empty for SpriteConfig '{}', using StrId as name", str_id);
            cmd.entity(spritecfg_ent).insert(DisplayName::new_trimmed(str_id.as_str()));
        } else {
            let disp_name = DisplayName::new_trimmed(seri.name);
            cmd.entity(spritecfg_ent).insert(disp_name);
        }
        //if seri.exclusive { comps_to_build.exclusive = Some(Exclusive); }

        if seri.directionable == Some(true) { cmd.entity(spritecfg_ent).insert(Directionable); }

        if seri.movement_based == Some(true) { cmd.entity(spritecfg_ent).insert(MovementBased); }

        if seri.grounding_based == Some(true) { cmd.entity(spritecfg_ent).insert(GroundingBased); }
        if let Some(parent_cat) = seri.parent_cat.as_ref().filter(|s| !s.trim().is_empty()) {
            let to_become_child = BecomeChildOfSpriteWithCategory(Category::new_truncated(parent_cat.trim()));
            cmd.entity(spritecfg_ent).insert(to_become_child);
        }

        if ! seri.anims.is_empty() {
            let mut anims_map = SpriteCfgAnimationsMap::default();
            for (anim_type, anim_id) in seri.anims {
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
} 

pub fn add_sprites_to_local_map(
    mut cmd: Commands,
    map: Option<ResMut<SpriteCfgEntityMap>>,
    query: Query<(Entity, &EntityPrefix, &StrId), (Added<SpriteConfig>, Or<(With<Disabled>, Without<Disabled>)>)>,
) {
    let Some(mut terrgen_map) = map else { return; };
    for (ent, prefix, str_id) in query.iter() {
        if let Err(err) = terrgen_map.0.insert(str_id, ent, ) {
            error!(target: "sprite_init", "{} {} already in SpriteCfgEntityMap : {}", prefix, str_id, err);
            cmd.entity(ent).despawn();
        } else {
            debug!(target: "sprite_init", "Inserted sprite '{}' into SpriteCfgEntityMap with entity {:?}", str_id, ent);
        }
    }
}

#[allow(unused_parens, )]
pub fn replace_string_ids_by_entities(
    mut cmd: Commands,
    mut query: Query<(Entity, &SpriteConfigStrIds, ), (/*Added<SpriteConfigStrIds>,*/)>,
    map: Option<Res<SpriteCfgEntityMap>>,
) {
    let Some(map) = map else {
        //error!(target: "sprite_building", "SpriteCfgEntityMap not found, cannot replace string ids");
        return;
    };

    for (ent, str_ids, ) in query.iter_mut() {
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
          
            cmd.entity(ent).insert(SpriteCfgsToBuild(entities_to_build));
        }
        cmd.entity(ent).remove::<SpriteConfigStrIds>();
    }
}

#[allow(unused_parens)]
pub fn add_spritechildren_and_comps(//SOLO SERVER PA SYNQUEAR
    mut cmd: Commands,
    mut father_query: Query<(Entity, &mut SpriteCfgsToBuild, Option<&SpriteBaseHolderRef>,), 
    (Without<SpriteConfig>, Changed<SpriteCfgsToBuild>,)>,
    spritecfgs_query: Query<(&StrId, Option<&SpriteCfgsToBuild>), 
    (With<SpriteConfig>, Or<(With<Disabled>, Without<Disabled>)>)>,
) {
    for (father_to_sprite, mut to_build, spriteholder_ref,) in father_query.iter_mut() {

        for spritecfg_ent in to_build.0.drain() {
            if let Ok((str_id, sprite_cfgs_to_build)) = spritecfgs_query.get(spritecfg_ent) {

                info!(target: "sprite_building", "Building sprite {}", str_id);

                let child_sprite = cmd.spawn((
                    str_id.clone(),
                    SpriteConfigRef(spritecfg_ent),
                    ChildOf(father_to_sprite),
                )).id();

                if let Some(spriteholder_ref) = spriteholder_ref {
                    cmd.entity(child_sprite).insert(spriteholder_ref.clone());
                } else {
                    cmd.entity(child_sprite).insert(SpriteBaseHolderRef{ base: father_to_sprite });
                }

                if let Some(sprite_cfgs_to_build) = sprite_cfgs_to_build {
                    cmd.entity(child_sprite).insert(sprite_cfgs_to_build.clone());
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
pub fn become_child_of_sprite_with_category(
    mut cmd: Commands,
    new_sprites: Query<(Entity, &SpriteBaseHolderRef, &SpriteConfigRef), (Without<SpriteConfig>, Changed<SpriteConfigRef>,)>,
    sprite_holder: Query<&HeldSprites>,
    other_sprites: Query<(Entity, &SpriteConfigRef), (Without<SpriteConfig>, )>,
    becomes: Query<(&BecomeChildOfSpriteWithCategory), (With<SpriteConfig>, Or<(With<Disabled>, Without<Disabled>)>)>,
    other_cats: Query<&Categories, (With<SpriteConfig>, Or<(With<Disabled>, Without<Disabled>)>)>,
) -> Result {
    let mut result: Result = Ok(());
    for (new_ent, &sprite_holder_ref, &new_sprite_cfg_ref) in new_sprites.iter(){
        if let Ok(becomes_child_of_sprite_with_cat) = becomes.get(new_sprite_cfg_ref.0) {unsafe {
            let held_sprites = sprite_holder.get(sprite_holder_ref.base).debug_expect_unchecked("SpriteHolderRef should have a HeldSprites component");

            for (other_ent, o_spritecfg_ref) in other_sprites.iter_many(held_sprites.sprite_ents()) {
                if new_ent == other_ent { continue; }

                let other_cats = match other_cats.get(o_spritecfg_ref.0) {
                    Ok(cats) => cats,
                    Err(e) => {
                        error!(target: "sprite_building", "Entity {:?} does not have Categories: {}", o_spritecfg_ref.0, e);
                        result = Err(e.into());
                        break;
                    },
                };
                if other_cats.0.contains(&becomes_child_of_sprite_with_cat.0) {
                    debug!(target: "sprite_building", "Adding ChildOfCategory to entity {:?} with id: {}", new_ent, becomes_child_of_sprite_with_cat.0);
                    cmd.entity(new_ent).insert(ChildOf(other_ent));
                    break;
                }
            }
        }}
    }
    result
}
