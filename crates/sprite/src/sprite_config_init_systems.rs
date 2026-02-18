use ::sprite_shared::{sprite_scale_offset::*, *};
#[allow(unused_imports)]
use bevy::prelude::*;
#[allow(unused_imports)]
use bevy_replicon::prelude::*;
use common::{common_components::*, common_tag_components::TagSet};
use game_common::game_common_components::*;
use sprite_animation_shared::AcAnimationEntityMap;

use crate::{sprite_components::*, sprite_resources::*};

#[allow(unused_parens)]
pub fn init_sprite_configs(
    mut cmd: Commands,
    scs_map: Res<SpriteConfigEntityMap>,
    library: Res<AcAnimationEntityMap>,
    scs_holder: Query<Entity, With<EguiScsHolder>>,
) {
    if !scs_map.0.is_empty() {
        return;
    }

    let scs_holder = scs_holder.single().unwrap();

    let mut comps_to_insert = Vec::new();

    for mut seri in load_sprite_config_seri_defs() {

        let str_id = match StrId::new_with_result(seri.id, 3) {
            Ok(id) => id,
            Err(e) => {
                let err =
                    BevyError::from(format!("Failed to create StrId for SpriteConfig: {}", e));
                error!(target: "sprite_init", "{}", err);
                continue;
            }
        };

        let spritecfg_ent = cmd.spawn_empty().id();

        let visib = match seri.visibility {
            Some(0) => Visibility::Inherited,
            Some(1) => Visibility::Visible,
            Some(2) => Visibility::Hidden,
            Some(v) => {
                warn!(target: "sprite_init", "Invalid visibility value: {} for SpriteConfig '{}', falling back to inherited", v, str_id);
                Visibility::default()
            }
            None => Visibility::Inherited,
        };

        let mut offset4children_cats = OffsetForChildren::default();
        for (cat, (offset_x, offset_y, direction)) in std::mem::take(&mut seri.offset4children) {
            offset4children_cats.0.insert(
                Tag::trunc(cat),
                (
                    Offset2D::from((offset_x, offset_y)),
                    AppliesOnSpriteDirection::from(direction),
                ),
            );
        }
        comps_to_insert.push((
            spritecfg_ent,
            (
                str_id.clone(),
                SpriteConfig,
                visib,
                offset4children_cats,
                EntityZero,
                ChildOf(scs_holder),
                Transform::default(),
            ),
        ));
        if !seri.tags.is_empty() {
            cmd.entity(spritecfg_ent).insert(TagSet::new(&seri.tags));
        }
        if seri.scale != (1.0, 1.0) {
            let scale_2d = seri.scale;
            cmd.entity(spritecfg_ent).insert(Scale2D::from(scale_2d));
        }
        if seri.offset != (0.0, 0.0) {
            let offset_2d = seri.offset;
            cmd.entity(spritecfg_ent).insert(Offset2D::from(offset_2d));
        }
        if seri.scale_up_down != (1.0, 1.0) {
            let scale_look_up_down = seri.scale_up_down;
            cmd.entity(spritecfg_ent)
                .insert(ScaleLookUpDown::from(scale_look_up_down));
        }
        if seri.scale_sideways != (1.0, 1.0) {
            let scale_sideways = seri.scale_sideways;
            cmd.entity(spritecfg_ent)
                .insert(ScaleSideways::from(scale_sideways));
        }
        if seri.offset_up_down != (0.0, 0.0) {
            let offset_up_down = seri.offset_up_down;
            cmd.entity(spritecfg_ent)
                .insert(OffsetUpDown::from(offset_up_down));
        }
        if seri.offset_down != (0.0, 0.0) {
            let offset_down = seri.offset_down;
            cmd.entity(spritecfg_ent)
                .insert(OffsetDown::from(offset_down));
        }
        if seri.offset_up != (0.0, 0.0) {
            let offset_up = seri.offset_up;
            cmd.entity(spritecfg_ent).insert(OffsetUp::from(offset_up));
        }
        if seri.offset_sideways != (0.0, 0.0) {
            let offset_sideways = seri.offset_sideways;
            cmd.entity(spritecfg_ent)
                .insert(OffsetSideways::from(offset_sideways));
        }

        if seri.name.trim().is_empty() {
            warn!(target: "sprite_init", "SpriteConfig name is empty for SpriteConfig '{}', using StrId as name", str_id);
            cmd.entity(spritecfg_ent)
                .insert(DisplayName::trunc(str_id.as_str()));
        } else {
            let disp_name = DisplayName::trunc(seri.name);
            cmd.entity(spritecfg_ent).insert(disp_name);
        }
        //if seri.exclusive { comps_to_build.exclusive = Some(Exclusive); }

        if seri.directionable {
            cmd.entity(spritecfg_ent).insert(Directionable);
        }

        if seri.movement_based {
            cmd.entity(spritecfg_ent).insert(MovementBased);
        }

        if seri.grounding_based {
            cmd.entity(spritecfg_ent).insert(GroundingBased);
        }
        if !seri.parent_cat.trim().is_empty() {
            let to_become_child = BecomeChildOfSpriteWithTag(Tag::trunc(seri.parent_cat.trim()));
            cmd.entity(spritecfg_ent).insert(to_become_child);
        }

        if !seri.mapped_anims.is_empty() {
            let mut anims_map = MappedAnimations::default();
            for (anim_type, anim_id) in seri.mapped_anims {
                let anim_type = AnimType::from_tuple(anim_type);
                let anim_id = StrId::trunc(anim_id);
                let Ok(&anim_ent) = library.0.get(&anim_id) else {
                    error!(target: "sprite_init", "SpriteConfig {}: AcAnimationEntityMap does not contain: {} ", str_id, anim_id);
                    continue;
                };
                anims_map.0.insert(anim_type, anim_ent);
            }
            if anims_map.0.is_empty() {
                error!(target: "sprite_init", "SpriteConfig '{}' animations map has no valid entries", str_id);
            } else {
                cmd.entity(spritecfg_ent).insert(anims_map);
            }
        } else {
            error!(target: "sprite_init", "SpriteConfig '{}' was given an empty animations map", str_id);
        }

        if !seri.children_sprites.is_empty() {
            let ids = SampleSpritesFromStrIds::new(seri.children_sprites.clone());
            cmd.entity(spritecfg_ent).insert(ids);
        }
    }
    cmd.try_insert_batch(comps_to_insert);
}
