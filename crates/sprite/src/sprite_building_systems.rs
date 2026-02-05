use being_shared::BeingInstTemplate;
use bevy::{ecs::entity::EntityHashSet, platform::collections::HashSet} ;
#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
use common::{common_components::*, common_tag_components::TagSet};
use game_common::game_common_components::*;
use sprite_animation_shared::AcAnimationProgresses ;
use ::sprite_shared::*;

use crate::{sprite_components::*, sprite_resources::* };

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
    let mut sprite_cfgs_to_build = Vec::new();
    
    query.iter().for_each(|(ent, str_ids)| {
        
        debug!(target: "sprite_building", "Replacing string ids for entity {:?}", ent);
        let mut entities_to_build = EntityHashSet::new();
        for id in str_ids.ids() {
            if let Ok(sprite_ent) = map.0.get_cloned(id) {
                debug!(target: "sprite_building", "Replacing string id '{}' with entity {:?}", id, sprite_ent);
                entities_to_build.insert(sprite_ent);
            } else {
                error!(target: "sprite_building", "ekf SpriteConfigEntityMap does not contain entity for id: {}", id);
            }
        }
        if ! entities_to_build.is_empty() {
            
            sprite_cfgs_to_build.push((ent, SpriteCfgsToBuild(entities_to_build)));
        }
    });
    cmd.try_insert_batch(sprite_cfgs_to_build);
}

#[allow(unused_parens)]
pub fn add_spritechildren_and_comps(//SOLO SERVER PA SYNQUEAR
    mut cmd: Commands,
    father_query: Query<(Entity, &SpriteCfgsToBuild, Option<&BaseHolderRef>,), 
    (Without<SpriteConfig>, Without<BeingInstTemplate>, Changed<SpriteCfgsToBuild>,)>,
    spritecfgs_query: Query<(Entity, &StrId, Option<&SpriteCfgsToBuild>), 
    (With<SpriteConfig>, common::AnyDisabling)>,
    held_sprites_query: Query<&HeldSprites, common::AnyDisabling>,
    sprite_config_ref_query: Query<&EntityZeroRef, common::AnyDisabling>,
) {
    father_query.iter().for_each(|(father_to_sprite, to_build, baseholder_ref)| {
        

        spritecfgs_query.iter_many(to_build.0.iter()).for_each(|(spritecfg_ent, str_id, extra_to_build)| {
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
                            return;
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
        });

        //cmd.entity(father_to_sprite).remove::<SpriteCfgsToBuild>();
        //NO HACER ESO PORQ HACE FALTA PARA LA REPLICACIÓN ^^
    });

}

#[allow(unused_parens)]
pub fn become_child_of_sprite_with_tag(
    mut cmd: Commands,
    new_sprites: Query<(Entity, &BaseHolderRef, &EntityZeroRef), (Without<SpriteConfig>, Changed<EntityZeroRef>,)>,
    sprite_holder: Query<&HeldSprites>,
    other_sprites: Query<(Entity, &EntityZeroRef), (Without<SpriteConfig>, )>,
    becomes_query: Query<(&BecomeChildOfSpriteWithTag), (common::AnyDisabling)>,
    other_cats: Query<&TagSet, (common::AnyDisabling)>,
) {
    let mut childofs_to_add = Vec::new();

    for (new_ent, &sprite_holder_ref, &new_sprite_cfg_ref) in new_sprites.iter(){
        let Ok(becomes_child_of_sprite_with_cat) = becomes_query.get(new_sprite_cfg_ref.0)
        else {continue;};

        
        let Ok(held_sprites) = sprite_holder.get(sprite_holder_ref.base)
        else {
            warn!(target: "sprite_building", "Cannot get HeldSprites for base holder {:?} when processing become_child_of_sprite_with_tag for entity {:?}", sprite_holder_ref.base, new_ent);
            continue;
        };
        
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
                childofs_to_add.push((new_ent, ChildOf(other_ent)));
                break;
            }
        }
    }
    cmd.try_insert_batch(childofs_to_add);
}

