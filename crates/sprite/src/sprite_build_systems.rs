use ::sprite_shared::*;
use being_shared::BeingInstTemplate;
#[allow(unused_imports)]
use bevy::{
    ecs::{entity::EntityHashSet, entity_disabling::Disabled},
    prelude::*,
};
#[allow(unused_imports)]
use bevy_replicon::prelude::*;
use common::{common_components::*, common_tag_components::TagSet};
use game_common::game_common_components::*;
use sprite_animation_shared::AcAnimationProgresses;

use crate::{sprite_components::*, sprite_resources::*};

#[allow(unused_parens)]
pub fn add_spritechildren_and_comps(
    mut cmd: Commands,
    changed_fathers: Query<
        Entity,
        (
            Without<SpriteConfig>,
            Without<BeingInstTemplate>,
            Changed<ScsToBuild>,
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
        (Entity, &StrId, Option<&ScsToBuild>),
        (With<SpriteConfig>, common::AnyDisabling),
    >,
    held_sprites_query: Query<&HeldSprites, common::AnyDisabling>,
    sprite_config_ref_query: Query<&EntityZeroRef, common::AnyDisabling>,
    mut removed_disabled: RemovedComponents<Disabled>,
) {
    let reenabled_fathers = collect_reenabled_entities(&mut removed_disabled);
    let mut fathers_to_build = reenabled_fathers.clone();
    fathers_to_build.extend(changed_fathers.iter());

    fathers_to_build.into_iter().for_each(|parent_of_sprite| {
        let Ok((to_build, baseholder_ref)) = father_query.get(parent_of_sprite) else {
            return;
        };
        let baseholder_ref = baseholder_ref.cloned().unwrap_or(BaseHolderRef { base: parent_of_sprite });
        let is_reenabled_only = reenabled_fathers.contains(&parent_of_sprite) && changed_fathers.get(parent_of_sprite).is_err();
        if is_reenabled_only && !needs_sprite_build(to_build, &baseholder_ref, &held_sprites_query, &sprite_config_ref_query) {
            return;
        }
        spritecfgs_query.iter_many(to_build.0.iter()).for_each(|(spritecfg_ent, str_id, extra_to_build)| {
            info!(target: "sprite_building", "Building sprite {}", str_id);

            if let Ok(held_sprites) = held_sprites_query.get(baseholder_ref.base) {
                for &sprite_ent in held_sprites.entities() {
                    if let Ok(sprite_cfg_ref) = sprite_config_ref_query.get(sprite_ent) {
                        if sprite_cfg_ref.0 == spritecfg_ent {
                            warn!(target: "sprite_building", "SpriteConfig '{}' already present in HeldSprites of base holder {:?}, skipping.", str_id, baseholder_ref.base);
                            //cmd.entity(parent_of_sprite).insert(DoNotRetryBuildScRef);
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

                ChildOf(parent_of_sprite),
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

        //cmd.entity(parent_of_sprite).remove::<SpriteCfgsToBuild>();
        //NO HACER ESO PORQ HACE FALTA PARA LA REPLICACIÓN ^^
    });
}

#[allow(unused_parens)]
pub fn remap_broken_sprite_config_refs_after_hotreload(
    mut sprites_query: Query<
        (&StrId, &mut EntityZeroRef),
        (Without<SpriteConfig>, With<BaseHolderRef>),
    >,
    sprite_cfg_query: Query<(), With<SpriteConfig>>,
    sprite_map: Res<SpriteConfigEntityMap>,
) {
    for (sprite_id, mut ezero_ref) in sprites_query.iter_mut() {
        if sprite_cfg_query.get(ezero_ref.0).is_ok() {
            continue;
        }
        let Ok(new_cfg_ent) = sprite_map.0.get_cloned(sprite_id) else {
            continue;
        };
        if new_cfg_ent != ezero_ref.0 {
            ezero_ref.0 = new_cfg_ent;
        }
    }
}

#[allow(unused_parens)]
pub fn become_child_of_sprite_with_tag(
    mut cmd: Commands,
    changed_new_sprites: Query<Entity, (Without<SpriteConfig>, Changed<EntityZeroRef>)>,
    new_sprites: Query<
        (&BaseHolderRef, &EntityZeroRef),
        (Without<SpriteConfig>, common::AnyDisabling),
    >,
    sprite_holder: Query<&HeldSprites>,
    other_sprites: Query<(Entity, &EntityZeroRef), (Without<SpriteConfig>,)>,
    becomes_query: Query<(&BecomeChildOfSpriteWithTag), (common::AnyDisabling)>,
    other_cats: Query<&TagSet, (common::AnyDisabling)>,
    mut removed_disabled: RemovedComponents<Disabled>,
) {
    let reenabled_sprites = collect_reenabled_entities(&mut removed_disabled);
    let mut sprites_to_process = reenabled_sprites;
    sprites_to_process.extend(changed_new_sprites.iter());
    let mut childofs_to_add = Vec::new();

    for new_ent in sprites_to_process {
        let Ok((&sprite_holder_ref, &new_sprite_cfg_ref)) = new_sprites.get(new_ent) else {
            continue;
        };
        let Ok(becomes_child_of_sprite_with_cat) = becomes_query.get(new_sprite_cfg_ref.0) else {
            continue;
        };

        let Ok(held_sprites) = sprite_holder.get(sprite_holder_ref.base) else {
            warn!(target: "sprite_building", "Cannot get HeldSprites for base holder {:?} when processing become_child_of_sprite_with_tag for entity {:?}", sprite_holder_ref.base, new_ent);
            continue;
        };

        for (other_ent, o_spritecfg_ref) in other_sprites.iter_many(held_sprites.entities()) {
            if new_ent == other_ent {
                continue;
            }

            let other_cats = match other_cats.get(o_spritecfg_ref.0) {
                Ok(cats) => cats,
                Err(e) => {
                    break;
                }
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

fn needs_sprite_build(
    to_build: &ScsToBuild,
    baseholder_ref: &BaseHolderRef,
    held_sprites_query: &Query<&HeldSprites, common::AnyDisabling>,
    sprite_config_ref_query: &Query<&EntityZeroRef, common::AnyDisabling>,
) -> bool {
    let Ok(held_sprites) = held_sprites_query.get(baseholder_ref.base) else {
        return true;
    };

    let mut built_cfgs = EntityHashSet::with_capacity(held_sprites.entities().len());
    for &sprite_ent in held_sprites.entities() {
        let Ok(sprite_cfg_ref) = sprite_config_ref_query.get(sprite_ent) else {
            continue;
        };
        built_cfgs.insert(sprite_cfg_ref.0);
    }

    to_build.0.iter().any(|cfg_ent| !built_cfgs.contains(cfg_ent))
}

fn collect_reenabled_entities(removed_disabled: &mut RemovedComponents<Disabled>) -> EntityHashSet {
    let mut entities = EntityHashSet::default();
    entities.extend(removed_disabled.read());
    entities
}
