use std::mem::take;

use bevy::{ecs::entity_disabling::Disabled, platform::collections::HashSet, render::{sync_world::SyncToRenderWorld, view::VisibilityClass}};
#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
use bevy_replicon::shared::server_entity_map::ServerEntityMap;
use bevy_replicon_renet::renet::RenetClient;
use common::common_components::{DisplayName, EntityPrefix, ImageHolder, StrId};
use debug_unwraps::DebugUnwrapExt;
use game_common::game_common_components::{Directionable, MyZ};
use sprite_animation_shared::sprite_animation_shared::{AnimationIdPrefix, AnimationState};

use crate::{sprite_components::*, sprite_resources::*};



#[allow(unused_parens)]
pub fn init_sprite_cfgs(
    mut cmd: Commands, map: Option<Res<SpriteCfgEntityMap>>,
    aserver: Res<AssetServer>,
    mut atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut seris_handles: ResMut<SpriteSerisHandles>,
    mut assets: ResMut<Assets<SpriteConfigSeri>>,
) -> Result {
    if map.is_some(){ return Ok(());}
    cmd.init_resource::<SpriteCfgEntityMap>();

    let mut result: Result = Ok(());

    for handle in take(&mut seris_handles.handles) {
        let Some(mut seri) = assets.remove(&handle) else {continue;};

        debug!(target: "sprite_loading", "Loading SpriteDataSeri from handle: {:?}", handle);
        
        let str_id = match StrId::new(seri.id, 3) {
            Ok(id) => id,
            Err(e) => {
                let err = BevyError::from(format!("Failed to create StrId for SpriteConfig: {}", e));
                    error!(target: "sprite_loading", "{}", err);
                    result = Err(err);
                    continue;
                }
            };
        let img_holder = match ImageHolder::new(&aserver, seri.img_path) {
            Ok(holder) => holder,
            Err(e) => {
                let err = BevyError::from(format!("Failed to load image for SpriteConfig {}: {}", str_id, e));
                error!(target: "sprite_loading", "{}", err);
                result = Err(err);
                continue;
            }
        };
        
        let atlas = AtlasLayoutData::new(seri.rows_cols, seri.frame_size);
        let atlas: TextureAtlas = atlas.into_texture_atlas(&mut atlas_layouts);

        let visib = match seri.visibility {
            0 => Visibility::Inherited, 1 => Visibility::Visible, 2 => Visibility::Hidden,    
            _ => {
                warn!(target: "sprite_loading", "Invalid visibility value: {} for SpriteConfig '{}', falling back to inherited", seri.visibility, str_id);
                Visibility::default()
            },
        };

        let mut offset4children_cats = OffsetForChildren::default();
        for (cat, offset_arr) in take(&mut seri.offset4children) {
            offset4children_cats.0.insert(Category::new(cat), Offset2D::from(offset_arr));
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

            Sprite::from_atlas_image(img_holder.0, atlas),
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

        if ! seri.parent_cat.is_empty() {
            let to_become_child = BecomeChildOfSpriteWithCategory::new(seri.parent_cat);
            cmd.entity(spritecfg_ent).insert(to_become_child);
        }

        if ! seri.anim_prefix.is_empty() {
            cmd.entity(spritecfg_ent).insert(AnimationIdPrefix::from(seri.anim_prefix));
        }
        
        if ! seri.children_sprites.is_empty(){
            if let Ok(ids) = SpriteConfigStrIds::new(seri.children_sprites){
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

        if seri.walk_anim {cmd.entity(spritecfg_ent).insert(WalkAnim);}
        if seri.swim_anim {cmd.entity(spritecfg_ent).insert(SwimAnim{use_still: seri.swim_anim_still});}
        if seri.fly_anim {cmd.entity(spritecfg_ent).insert(FlyAnim{use_still: seri.fly_anim_still});}

        match seri.flip_horiz {
            1 => { cmd.entity(spritecfg_ent).insert(FlipHorizIfDir::Any); },
            2 => { cmd.entity(spritecfg_ent).insert(FlipHorizIfDir::Left); },
            3 => { cmd.entity(spritecfg_ent).insert(FlipHorizIfDir::Right); },
            _ => {},
        };
        
    }
    result
} 



pub fn add_sprites_to_local_map(
    mut cmd: Commands,
    map: Option<ResMut<SpriteCfgEntityMap>>,
    query: Query<(Entity, &EntityPrefix, &StrId), (Added<SpriteConfig>, Or<(With<Disabled>, Without<Disabled>)>)>,
) -> Result {
    let mut result: Result = Ok(());
    if let Some(mut terrgen_map) = map {
        for (ent, prefix, str_id) in query.iter() {
            if let Err(err) = terrgen_map.0.insert(str_id, ent, ) {
                error!(target: "sprite_loading", "{} {} already in SpriteCfgEntityMap : {}", prefix, str_id, err);
                cmd.entity(ent).despawn();
                result = Err(err);
            } else {
                debug!(target: "sprite_loading", "Inserted sprite '{}' into SpriteCfgEntityMap with entity {:?}", str_id, ent);
            }
        }
    }
    result
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
    spritecfgs_query: Query<(&StrId, Has<AnimationIdPrefix>, Option<&SpriteCfgsToBuild>), 
    (With<SpriteConfig>, Or<(With<Disabled>, Without<Disabled>)>)>,
) {
    for (father_to_sprite, mut to_build, spriteholder_ref,) in father_query.iter_mut() {

        for spritecfg_ent in to_build.0.drain() {
            if let Ok((str_id, has_anim, sprite_cfgs_to_build)) = spritecfgs_query.get(spritecfg_ent) {

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
                if has_anim {
                    cmd.entity(child_sprite).insert(AnimationState::default());
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


#[allow(unused_parens, )]

pub fn client_map_server_sprite_cfgs(
    trigger: Trigger<SpriteCfgEntityMap>,
    client: Option<Res<RenetClient>>,
    mut entis_map: ResMut<ServerEntityMap>,
    own_map: Res<SpriteCfgEntityMap>,
) {
    if client.is_none() { return; }


    let SpriteCfgEntityMap(received_map) = trigger.event().clone();
    for (hash_id, &server_entity) in received_map.0.iter() {
        if let Ok(client_entity) = own_map.0.get_with_hash(hash_id) {
            debug!(target: "sprite_loading", "Mapping server entity {:?} to local entity {:?}", server_entity, client_entity);
            entis_map.insert(server_entity, client_entity);
        } else {
            error!(target: "sprite_loading", "Received entity {:?} with hash id {:?} not found in own map", server_entity, hash_id);
        }
    }
}

