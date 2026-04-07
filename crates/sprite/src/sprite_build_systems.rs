use being_shared::BeingInstTemplate;
#[allow(unused_imports)]
use bevy::{
    ecs::{entity::EntityHashSet, entity_disabling::Disabled},
    platform::collections::HashMap,
    platform::collections::HashSet,
    prelude::*,
};
#[allow(unused_imports)]
use bevy_replicon::prelude::*;
use common::{common_components::*, common_tag_components::TagSet};
use game_common::game_common_components::*;
use sprite_animation_shared::AcAnimationProgresses;
use ::sprite_shared::*;

use crate::{ sprite_resources::*};

#[allow(unused_parens)]
pub fn add_spritechildren_and_comps(
    mut cmd: Commands,
    changed_fathers: Query<
        Entity,
        (
            Or<(Changed<ScsToBuild>, Changed<Visibility>, (Without<HeldSprites>, With<ScsToBuild>,))>,
            Without<SpriteConfig>,
            Without<BeingInstTemplate>,
        ),
    >,
    father_query: Query<
        (&ScsToBuild, Option<&BaseHolderRef>),
        (
            Without<SpriteConfig>,
            Without<BeingInstTemplate>,
            common::AnyDisabling,
        ),
    >,
    spritecfgs_query: Query<
        (Entity, &StrId, &HashId, Option<&ScsToBuild>),
        (With<SpriteConfig>, common::AnyDisabling),
    >,
    sprite_map: Res<SpriteConfigEntityMap>,
    held_sprites_query: Query<&HeldSprites, common::AnyDisabling>,
    sprite_config_hash_query: Query<&HashId, (With<SpriteConfig>, common::AnyDisabling)>,
    mut removed_disabled: RemovedComponents<Disabled>,
    mut removed_unloaded: RemovedComponents<being_shared::Unloaded>,
    mut fathers_to_build: Local<Vec<Entity>>,
) {
    fathers_to_build.clear();
    fathers_to_build.extend(removed_disabled.read());
    fathers_to_build.extend(removed_unloaded.read());
    fathers_to_build.extend(changed_fathers.iter());

    fathers_to_build.iter().for_each(|&parent_of_sprite| {
        let Ok((to_build, baseholder_ref)) = father_query.get(parent_of_sprite) else {
            return;
        };
        let baseholder_ref = baseholder_ref.cloned().unwrap_or(BaseHolderRef { base: parent_of_sprite });
        let is_reenabled_only = changed_fathers.get(parent_of_sprite).is_err();
        if is_reenabled_only && !needs_sprite_build(to_build, &baseholder_ref, &held_sprites_query, &sprite_config_hash_query) {
            return;
        }
        for cfg_hash_id in to_build.0.iter() {
            let Ok(&spritecfg_ent) = sprite_map.0.get(*cfg_hash_id) else {
                warn!(target: "sprite_building", "SpriteConfig hash {} missing from SpriteConfigEntityMap while building {:?}", cfg_hash_id, parent_of_sprite);
                continue;
            };
            let Ok((_, str_id, _, extra_to_build)) = spritecfgs_query.get(spritecfg_ent) else {
                warn!(target: "sprite_building", "SpriteConfig entity {:?} for hash {} is missing during build", spritecfg_ent, cfg_hash_id);
                continue;
            };
            info!(target: "sprite_building", "Building sprite {}", str_id);

            let mut already_built = false;
            if let Ok(held_sprites) = held_sprites_query.get(baseholder_ref.base) {
                for sprite_ent in held_sprites.iter() {
                    let Ok(sprite_cfg_ref) = sprite_config_hash_query.get(sprite_ent) else {
                        continue;
                    };
                    if *sprite_cfg_ref == *cfg_hash_id {
                        debug!(target: "sprite_building", "SpriteConfig '{}' already present in HeldSprites of base holder {:?}, skipping.", str_id, baseholder_ref.base);
                        //cmd.entity(parent_of_sprite).insert(DoNotRetryBuildScRef);
                        already_built = true;
                        break;
                    }
                }
            }
            if already_built {
                continue;
            }
            let sprite = cmd.spawn((
                TemplEntiRef(spritecfg_ent),
                Visibility::default(),
                Transform::default(),
                AcAnimationProgresses::default(),

                ChildOf(parent_of_sprite),
                //Replicated,
            )).id();

            cmd.entity(sprite).try_insert(baseholder_ref);

            if let Some(extra_to_build) = extra_to_build {
                cmd.entity(sprite).try_insert(extra_to_build.clone());
                // NO HACE FALTA PONER UN SpriteCfgsBuiltSoFar EN ESTO PORQ LOS CHILDREN FALTANTES SE VAN A AUTOCONSTRUIR CON LA PRESENCIA DE ESTE
            }

            // if let Some(excl) = &comps_to_build.exclusive {
            //     cmd.entity(child_sprite).insert(excl.clone());
            // }
        }

        //cmd.entity(parent_of_sprite).remove::<SpriteCfgsToBuild>();
        //NO HACER ESO PORQ HACE FALTA PARA LA REPLICACIÓN ^^
    });
}

#[allow(unused_parens)]
pub fn remap_broken_sprite_config_refs_after_hotreload(
    mut cmd: Commands,
    sprites_query: Query<(Entity, &TemplEntiRef), (Without<SpriteConfig>, With<BaseHolderRef>)>,
    str_id_query: Query<&StrId>,
    sprite_map: Res<SpriteConfigEntityMap>,
) {
    for (sprite_ent, templ_ref) in sprites_query.iter() {
        let Ok(sprite_id) = str_id_query.get(templ_ref.0)
        else {
            continue;
        };
        let Ok(new_cfg_ent) = sprite_map.0.get_cloned(&sprite_id) else {
            continue;
        };
        if new_cfg_ent != templ_ref.0 {
            cmd.entity(sprite_ent).insert(TemplEntiRef(new_cfg_ent));
        }
    }
}

#[allow(unused_parens)]
pub fn become_child_of_sprite_with_tag(
    mut cmd: Commands,
    changed_new_sprites: Query<Entity, (Without<SpriteConfig>, Changed<TemplEntiRef>,  common::AnyDisabling)>,
    new_sprite_baseholder_query: Query<&BaseHolderRef, (Without<SpriteConfig>, common::AnyDisabling)>,
    sprite_config_ref_query: Query<&TemplEntiRef, (Without<SpriteConfig>, common::AnyDisabling)>,
    sprite_holder: Query<&HeldSprites>,
    becomes_query: Query<(&BecomeChildOfSpriteWithTag), (common::AnyDisabling)>,
    other_cats: Query<&TagSet, (common::AnyDisabling)>,
    mut removed_disabled: RemovedComponents<Disabled>,
    mut sprites_to_process: Local<EntityHashSet>,
    mut parent_by_tag: Local<HashMap<Tag, Entity>>,
) {
    let mut childofs_to_add = Vec::new();
    let changed_iter = changed_new_sprites.iter();
    sprites_to_process.reserve(removed_disabled.len() + changed_iter.size_hint().1.unwrap_or(changed_iter.size_hint().0));
    sprites_to_process.extend(changed_iter);
    sprites_to_process.extend(removed_disabled.read());


    for new_ent in sprites_to_process.drain() {
        let Ok(sprite_holder_ref) = new_sprite_baseholder_query.get(new_ent) else {
            continue;
        };
        let Ok(held_sprites) = sprite_holder.get(sprite_holder_ref.base) else {
            warn!(target: "sprite_building", "Cannot get HeldSprites for base holder {:?} when processing become_child_of_sprite_with_tag for entity {:?}", sprite_holder_ref.base, new_ent);
            continue;
        };

        parent_by_tag.clear();
        parent_by_tag.reserve(held_sprites.len());
        for other_ent in held_sprites.iter() {
            let Ok(o_spritecfg_ref) = sprite_config_ref_query.get(other_ent) else {
                continue;
            };
            let Ok(other_cats) = other_cats.get(o_spritecfg_ref.0) else {
                continue;
            };
            for tag in other_cats.iter() {
                parent_by_tag.entry(tag.clone()).or_insert(other_ent);
            }
        }

        let Ok(new_sprite_cfg_ref) = sprite_config_ref_query.get(new_ent) else {
            continue;
        };
        let Ok(becomes_child_of_sprite_with_cat) = becomes_query.get(new_sprite_cfg_ref.0) else {
            continue;
        };

        let Some(&parent_ent) = parent_by_tag.get(&becomes_child_of_sprite_with_cat.0) else {
            continue;
        };
        if parent_ent == new_ent {
            continue;
        }
        debug!(target: "sprite_building", "Adding ChildOfTag to entity {:?} with id: {}", new_ent, becomes_child_of_sprite_with_cat.0);
        childofs_to_add.push((new_ent, ChildOf(parent_ent)));
    }
    cmd.try_insert_batch(std::mem::take(&mut childofs_to_add));
}

fn needs_sprite_build(
    to_build: &ScsToBuild,
    baseholder_ref: &BaseHolderRef,
    held_sprites_query: &Query<&HeldSprites, common::AnyDisabling>,
    sprite_config_hash_query: &Query<&HashId, (With<SpriteConfig>, common::AnyDisabling)>,
) -> bool {
    let Ok(held_sprites) = held_sprites_query.get(baseholder_ref.base) else {
        return true;
    };

    let mut built_cfgs = HashSet::with_capacity(held_sprites.iter().len());
    for sprite_ent in held_sprites.iter() {
        let Ok(sprite_cfg_ref) = sprite_config_hash_query.get(sprite_ent) else {
            continue;
        };
        built_cfgs.insert(*sprite_cfg_ref);
    }

    to_build.0.iter().any(|cfg_ent| !built_cfgs.contains(cfg_ent))
}
