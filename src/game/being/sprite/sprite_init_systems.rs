use bevy::{ecs::entity_disabling::Disabled, platform::collections::HashSet};
#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;

use crate::{common::common_components::{DisplayName, EntityPrefix, MyZ, StrId}, game::{being::sprite::{sprite_components::*, sprite_resources::*}, game_components::ImageHolder}};

#[allow(unused_parens)]
pub fn init_sprite_cfgs(
    mut cmd: Commands, 
    aserver: Res<AssetServer>,
    mut atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut seris_handles: ResMut<SpriteSerisHandles>,
    mut assets: ResMut<Assets<SpriteConfigSeri>>,
) -> Result {
    let mut result: Result = Ok(());
    use std::mem::take;

    for handle in take(&mut seris_handles.handles) {
        info!("Loading SpriteDataSeri from handle: {:?}", handle);
        if let Some(mut seri) = assets.remove(&handle) {
            
            let str_id = match StrId::new(seri.id) {
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

            let categories: Categories = Categories::new(take(&mut seri.categories));
            
            let atlas = AtlasLayoutData::new(seri.rows_cols, seri.frame_size);
            let atlas: TextureAtlas = atlas.into_texture_atlas(&mut atlas_layouts);

            let visib = match seri.visibility {
                0 => Visibility::Inherited, 
                1 => Visibility::Visible,   
                2 => Visibility::Hidden,    
                _ => {
                    warn!(target: "sprite_loading", "Invalid visibility value: {} for SpriteConfig '{}', falling back to inherited", seri.visibility, str_id);
                    Visibility::default()
                },
            };

            let mut offset4children_cats = OffsetForChildren::default();
            if ! seri.offset4children.is_empty(){
                for (cat, offset_arr) in take(&mut seri.offset4children) {
                    offset4children_cats.0.insert(Category::new(cat), Offset2D::from(offset_arr));
                }
            }
            
            let spritecfg_ent = cmd.spawn((
                str_id.clone(), 
                categories,
                visib,
                offset4children_cats,
                MyZ::new(seri.z),
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
                let disp_name = DisplayName::new(seri.name.clone());
                cmd.entity(spritecfg_ent).insert(disp_name);
            }
            //if seri.exclusive { comps_to_build.exclusive = Some(Exclusive); }

            if seri.directionable {cmd.entity(spritecfg_ent).insert(Directionable);}

            if ! seri.parent_cat.is_empty() {
                let to_become_child = BecomeChildOfSpriteWithCategory::new(take(&mut seri.parent_cat));
                cmd.entity(spritecfg_ent).insert(to_become_child);
            }

            if ! seri.anim_prefix.is_empty() {
                cmd.entity(spritecfg_ent).insert(AnimationIdPrefix::from(take(&mut seri.anim_prefix)));
            }
            
            if ! seri.children_sprites.is_empty(){
                cmd.entity(spritecfg_ent).insert(SpriteConfigStringIds(take(&mut seri.children_sprites)));
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
        else {
            warn!(target: "sprite_loading", "SpriteDataSeri with handle {:?} not found in assets", handle);
        }
    }
    result
} 



pub fn add_sprites_to_local_map(
    map: Option<ResMut<SpriteCfgEntityMap>>,
    query: Query<(Entity, &EntityPrefix, &StrId), (With<SpriteConfig>, Or<(With<Disabled>, Without<Disabled>)>)>,
) -> Result {
    let mut result: Result = Ok(());
    if let Some(mut terrgen_map) = map {
        for (ent, prefix, str_id) in query.iter() {
            if let Err(err) = terrgen_map.0.insert(str_id, ent, prefix) {
                error!(target: "sprite_loading", "{}", err);
                result = Err(err);
            } else {
                info!(target: "sprite_loading", "Inserted sprite '{}' into SpriteConfigEntityMap with entity {:?}", str_id, ent);
            }
        }
    }
    result
}

#[allow(unused_parens, )]
pub fn replace_string_ids_by_entities(
    mut cmd: Commands,
    mut query: Query<(Entity, &SpriteConfigStringIds, Option<&mut SpriteCfgsToBuild>, &mut SpriteCfgsBuiltSoFar), (Added<SpriteConfigStringIds>,)>,
    map: Res<SpriteCfgEntityMap>,
) {
    for (ent, string_ids, sprite_cfgs_to_build, mut built_so_far) in query.iter_mut() {
        info!(target: "sprite_building", "Replacing string ids for entity {:?}", ent);
        let mut entities_to_build = if let Some(children_refs) = sprite_cfgs_to_build {
            std::mem::take(&mut children_refs.into_inner().0)
        } else {
            HashSet::new()
        };
        for id in &string_ids.0 {
            if let Ok(sprite_ent) = map.0.get(id) {
                info!(target: "sprite_building", "Replacing string id '{}' with entity {:?}", id, sprite_ent);
                entities_to_build.insert(sprite_ent);
            } else {
                error!(target: "sprite_building", "SpriteConfigEntityMap does not contain entity for id: {}", id);
            }
        }
        if ! entities_to_build.is_empty() {
            built_so_far.0.extend(entities_to_build.clone());
            cmd.entity(ent).insert(SpriteCfgsToBuild(entities_to_build));
        }
        cmd.entity(ent).remove::<SpriteConfigStringIds>();
    }
}


#[allow(unused_parens)]
pub fn add_spritechildren_and_comps(
    mut cmd: Commands,
    mut father_query: Query<(Entity, &mut SpriteCfgsToBuild, Option<&SpriteHolderRef>,), (Without<SpriteConfig>, Added<SpriteCfgsToBuild>,)>,
    spritecfgs_query: Query<(
        &StrId, &Sprite
    ), (With<SpriteConfig>, Or<(With<Disabled>, Without<Disabled>)>)>,
) {
    for (father_to_sprite, mut to_build, spriteholder_ref,) in father_query.iter_mut() {

        for spritecfg_ent in to_build.0.drain() {
            if let Ok((str_id, sprite,)) = spritecfgs_query.get(spritecfg_ent) {
                
                info!(target: "sprite_building", "Building sprite {}", str_id);

                let child_sprite = cmd.spawn((
                    str_id.clone(),
                    sprite.clone(),
                    SpriteConfigRef(spritecfg_ent),
                    ChildOf(father_to_sprite),
                    EntityPrefix::new("Sprite"),
                )).id();

                if let Some(spriteholder_ref) = spriteholder_ref {
                    cmd.entity(child_sprite).insert(spriteholder_ref.clone());
                } else {
                    cmd.entity(child_sprite).insert(SpriteHolderRef{ base: father_to_sprite });
                }
 
                // if let Some(excl) = &comps_to_build.exclusive {
                //     cmd.entity(child_sprite).insert(excl.clone());
                // }
            } else{
                warn!(target: "sprite_building", "query does not contain entity for: {}", spritecfg_ent);
            }
        }
        cmd.entity(father_to_sprite).remove::<SpriteCfgsToBuild>();
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
        let becomes_child_of_sprite_with_cat = match becomes.get(new_sprite_cfg_ref.0) {
            Ok(become_cats) => {become_cats},
            Err(e) => {
                info!(target: "sprite_building", "Entity {:?} does not have BecomeChildOfSpriteWithCategory: {}", new_ent, e);
                continue;
            },
        };
        match sprite_holder.get(sprite_holder_ref.base) {
            Ok(held_sprites) => {

                for (other_ent, o_spritecfg_ref) in other_sprites.iter_many(held_sprites.entities()) {
                    if new_ent == other_ent { continue; }
    
                    let other_cats = match other_cats.get(o_spritecfg_ref.0) {
                        Ok(cats) => cats,
                        Err(e) => {
                            error!(target: "sprite_building", "Entity {:?} does not have Categories: {}", o_spritecfg_ref.0, e);
                            break;
                        },
                    };
                    if other_cats.0.contains(&becomes_child_of_sprite_with_cat.0) {
                        info!(target: "sprite_building", "Adding ChildOfCategory to entity {:?} with id: {}", new_ent, becomes_child_of_sprite_with_cat.0);
                        cmd.entity(new_ent).insert(ChildOf(other_ent));
                        break;
                    }
                }
           
            },
            Err(e) => {
                let err = BevyError::from(format!("Entity {:?} does not have HeldSprites: {}", new_ent, e));
                result = Err(err);
            }
        }
    }
    result
}


