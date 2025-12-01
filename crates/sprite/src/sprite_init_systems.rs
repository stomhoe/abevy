use std::mem::take;

use bevy::{ecs::entity_disabling::Disabled, platform::collections::{HashMap, HashSet}, render::sync_world::SyncToRenderWorld};
#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
use bevy_replicon::shared::server_entity_map::ServerEntityMap;
use bevy_replicon_renet::renet::RenetClient;
use bevy_spritesheet_animation::prelude::Animation;
use common::common_components::{AssetScoped, DisplayName, EntityPrefix, ImageHolder, ImagePathHolder, StrId};
use debug_unwraps::DebugUnwrapExt;
use game_common::game_common_components::{Categories, Category, Directionable, MyZ};
use sprite_animation_shared::{AnimationLibrary, sprite_animation_shared::{AnimationState}};

use crate::{sprite_components::*, sprite_resources::*, sprite_scale_offset_components::*};



#[derive(Component, Debug, Default, )]
#[require(AssetScoped, EntityPrefix::new_truncated("SpriteConfigs"), )]
struct SpriteConfigsHolder;

#[allow(unused_parens)]
pub fn init_sprite_cfgs(
    mut cmd: Commands, map: Option<Res<SpriteCfgEntityMap>>,

    mut seris_handles: ResMut<SpriteSerisHandles>,
    mut assets: ResMut<Assets<SpriteConfigSeri>>,
) {
    if map.is_some(){ return; }


    cmd.init_resource::<SpriteCfgEntityMap>();
    let holder = cmd.spawn((SpriteConfigsHolder, )).id();


    for handle in take(&mut seris_handles.handles) {
        let Some(mut seri) = assets.remove(&handle) else {continue;};

        debug!(target: "sprite_loading", "Loading SpriteDataSeri from handle: {:?}", handle);
        
        let str_id = match StrId::new_with_result(seri.id, 3) {
            Ok(id) => id,
            Err(e) => {
                let err = BevyError::from(format!("Failed to create StrId for SpriteConfig: {}", e));
                    error!(target: "sprite_loading", "{}", err);
                    continue;
                }
            };

        
        //let atlas = AtlasLayoutData::new(seri.rows_cols, seri.frame_size);
        //let atlas: TextureAtlas = atlas.into_texture_atlas(&mut atlas_layouts);

        let visib = match seri.visibility {
            0 => Visibility::Inherited, 1 => Visibility::Visible, 2 => Visibility::Hidden,    
            _ => {
                warn!(target: "sprite_loading", "Invalid visibility value: {} for SpriteConfig '{}', falling back to inherited", seri.visibility, str_id);
                Visibility::default()
            },
        };

        let mut offset4children_cats = OffsetForChildren::default();
        for (cat, offset_arr) in take(&mut seri.offset4children) {
            offset4children_cats.0.insert(Category::new(cat), (Offset2D::from(offset_arr.0), AppliesOnSpriteDirection::from(offset_arr.1)));
        }
        
        let spritecfg_ent = cmd.spawn((
            str_id.clone(), 
            SpriteConfig,
            Categories::new(seri.categories),
            visib,
            offset4children_cats,
            MyZ(seri.z),
            Scale2D::from(seri.scale.unwrap_or([1.0, 1.0])),
            ScaleLookUpDown::from(seri.scale_up_down.unwrap_or([1.0, 1.0])),
            ScaleSideways::from(seri.scale_sideways.unwrap_or([1.0, 1.0])),
            Offset2D::from(seri.offset),
            OffsetUpDown::from(seri.offset_up_down.unwrap_or_default()),
            OffsetDown::from(seri.offset_down.unwrap_or_default()),
            OffsetUp::from(seri.offset_up.unwrap_or_default()),
            OffsetSideways::from(seri.offset_sideways.unwrap_or_default()),
        )).insert((
            ChildOf(holder),

        )).id();


        if seri.name.is_empty() {
            warn!(target: "sprite_loading", "SpriteConfig name is empty for SpriteConfig '{}', using StrId as name", str_id);
            cmd.entity(spritecfg_ent).insert(DisplayName(str_id.to_string()));
        } else {
            let disp_name = DisplayName::new(seri.name);
            cmd.entity(spritecfg_ent).insert(disp_name);
        }
        //if seri.exclusive { comps_to_build.exclusive = Some(Exclusive); }

        if seri.directionable { cmd.entity(spritecfg_ent).insert(Directionable); }

        if seri.movement_based { cmd.entity(spritecfg_ent).insert(MovementBased); }

        if seri.grounding_based { cmd.entity(spritecfg_ent).insert(GroundingBased); }

        if ! seri.parent_cat.is_empty() {
            let to_become_child = BecomeChildOfSpriteWithCategory(Category::new(seri.parent_cat));
            cmd.entity(spritecfg_ent).insert(to_become_child);
        }

        if ! seri.anims.is_empty() {
            let mut new_map = HashMap::default();
            for (anim_type, anim_id) in seri.anims {
                let anim_type = AnimType::from_tuple(anim_type);

                let anim_id = match StrId::new_with_result(&anim_id, 3) {
                    Ok(anim_id) => { anim_id }
                    Err(e) => {
                        error!("SpriteConfig '{}' has invalid img_anim_pair_id string in ReplicatedAnimationsMap: {} ({})", str_id, anim_id, e);
                        continue;
                    }
                };
                new_map.insert(anim_type, anim_id);
            }
            if new_map.is_empty() {
                error!(target: "sprite_loading", "SpriteConfig '{}' animations map has no valid entries", str_id);
            }
            else {
                cmd.entity(spritecfg_ent).insert(ReplicatedAnimationsMap(new_map));
            }
        }
        else {
            error!(target: "sprite_loading", "SpriteConfig '{}' was given an empty animations map", str_id);
        }

        if ! seri.children_sprites.is_empty(){
            #[allow(irrefutable_let_patterns, )]
            if let ids = SpriteConfigStrIds::new(seri.children_sprites){
                cmd.entity(spritecfg_ent).insert(ids);
            }
            else {
                error!(target: "sprite_loading", "Failed to create SpriteConfigStrIds for SpriteConfig '{}'", str_id);
            }
        }
        
        if let Some(color) = seri.color {
            let (red, green, blue, alpha) = color.into();
            cmd.entity(spritecfg_ent).insert(ColorHolder(Color::srgba_u8(red, green, blue, alpha)));
        }



        match seri.flip_horiz {
            1 => { cmd.entity(spritecfg_ent).insert(FlipHorizIfDir::Any); },
            2 => { cmd.entity(spritecfg_ent).insert(FlipHorizIfDir::Left); },
            3 => { cmd.entity(spritecfg_ent).insert(FlipHorizIfDir::Right); },
            _ => {},
        };
        
    }
} 

// ----------------------> NO OLVIDARSE DE AGREGARLO AL Plugin DEL MÓDULO <-----------------------------
//                                                       ^^^^
#[allow(unused_parens)]
pub fn instantiate_anim_map(mut cmd: Commands, 
    query: Query<(Entity, &StrId, &ReplicatedAnimationsMap),(Changed<ReplicatedAnimationsMap>, With<SpriteConfig>,)>,
    library: Res<AnimationLibrary>,
) {
    for (ent, str_id, repli_map) in query.iter() {
        
        let mut anims_map = SpriteCfgAnimationsMap::default();
        for (anim_type, anim_id) in &repli_map.0 {
            let anim_handle = match library.0.get(anim_id) {
                Some(handle) => handle,
                None => {
                    error!(target: "sprite_loading", "SpriteConfig {}: AnimationLibrary does not contain animation id: {} for ", str_id, anim_id);
                    continue;
                }
            };
            anims_map.0.insert(anim_type.clone(), anim_handle.clone());
        }
        cmd.entity(ent).insert(anims_map);
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
            error!(target: "sprite_loading", "{} {} already in SpriteCfgEntityMap : {}", prefix, str_id, err);
            cmd.entity(ent).despawn();
        } else {
            debug!(target: "sprite_loading", "Inserted sprite '{}' into SpriteCfgEntityMap with entity {:?}", str_id, ent);
        }
    }
}

#[allow(unused_parens, )]
pub fn replace_string_ids_by_entities(
    mut cmd: Commands,
    mut query: Query<(Entity, &SpriteConfigStrIds, ), (Added<SpriteConfigStrIds>,)>,
    map: Option<Res<SpriteCfgEntityMap>>,
) {
    let Some(map) = map else {
        error!(target: "sprite_building", "SpriteCfgEntityMap not found, cannot replace string ids");
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
                error!(target: "sprite_building", "SpriteConfigEntityMap does not contain entity for id: {}", id);
            }
        }
        if ! entities_to_build.is_empty() {
          
            cmd.entity(ent).insert(SpriteCfgsToBuild(entities_to_build));
        }
        cmd.entity(ent).remove::<SpriteConfigStrIds>();
    }
}

//LO HACEN TODOS
#[allow(unused_parens)]
pub fn insert_sprite_to_instance(mut cmd: Commands, 
    instance_query: Query<(Entity, &SpriteConfigRef, /*&BecomeChildOf*/),( Changed<SpriteHolderRef>, Without<SpriteConfig>, )>,
    spritecfgs_query: Query<(&Sprite, &Visibility), (With<SpriteConfig>, Or<(With<Disabled>, Without<Disabled>)>)>,
    
) {
    for (ent, sprite_config_ref, /*become_child_of*/) in instance_query.iter() {
        if let Ok((sprite, visibility)) = spritecfgs_query.get(sprite_config_ref.0) {
            cmd.entity(ent).insert((SyncToRenderWorld, sprite.clone(), visibility.clone(), /*ChildOf(become_child_of.0)*/));
        } else {
            warn!(target: "sprite_building", "SpriteConfigRef {:?} does not have a Sprite component", sprite_config_ref.0);
        }
    }
}


#[allow(unused_parens)]
pub fn add_spritechildren_and_comps(//SOLO SERVER PA SYNQUEAR
    mut cmd: Commands,
    mut father_query: Query<(Entity, &mut SpriteCfgsToBuild, Option<&SpriteHolderRef>,), 
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
                    Transform::default(),
                    ChildOf(father_to_sprite),
                    Visibility::Inherited,
                )).id();

                if let Some(spriteholder_ref) = spriteholder_ref {
                    cmd.entity(child_sprite).insert(spriteholder_ref.clone());
                } else {
                    cmd.entity(child_sprite).insert(SpriteHolderRef{ base: father_to_sprite });
                }
                // if has_anim {
                //     cmd.entity(child_sprite).insert(AnimationState::default());
                // }

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
    }
}

#[allow(unused_parens)]
pub fn become_child_of_sprite_with_category(
    mut cmd: Commands,
    new_sprites: Query<(Entity, &SpriteHolderRef, &SpriteConfigRef), (Without<SpriteConfig>, Changed<SpriteConfigRef>,)>,
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


// TODO replicar los spritecfgs normalmente en vez de hacer esto
// #[allow(unused_parens, )]

// pub fn client_map_server_sprite_cfgs(
//     trigger: On<SpriteCfgEntityMap>,
//     client: Option<Res<RenetClient>>,
//     mut entis_map: ResMut<ServerEntityMap>,
//     own_map: Res<SpriteCfgEntityMap>,
// ) {
//     if client.is_none() { return; }


//     let SpriteCfgEntityMap(received_map) = trigger.event().clone();
//     for (hash_id, &server_entity) in received_map.0.iter() {
//         if let Ok(client_entity) = own_map.0.get_with_hash(hash_id) {
//             debug!(target: "sprite_loading", "Mapping server entity {:?} to local entity {:?}", server_entity, client_entity);
//             entis_map.insert(server_entity, client_entity);
//         } else {
//             error!(target: "sprite_loading", "Received entity {:?} with hash id {:?} not found in own map", server_entity, hash_id);
//         }
//     }
// }

