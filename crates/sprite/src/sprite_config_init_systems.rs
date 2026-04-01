use ::sprite_shared::*;
#[allow(unused_imports)]
use bevy::prelude::*;
#[allow(unused_imports)]
use bevy_replicon::prelude::*;
use common::{SPRITE_INIT, common_components::*, common_tag_components::TagSet};
use game_common::game_common_components::*;
use sprite_animation_shared::AcAnimationEntityMap;

use crate::{ sprite_resources::*};

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
                error!(target: SPRITE_INIT, "{}", err);
                continue;
            }
        };

        let spritecfg_ent = cmd.spawn_empty().id();

        let visib = match seri.visibility {
            Some(0) => Visibility::Inherited,
            Some(1) => Visibility::Visible,
            Some(2) => Visibility::Hidden,
            Some(v) => {
                warn!(target: SPRITE_INIT, "Invalid visibility value: {} for SpriteConfig '{}', falling back to inherited", v, str_id);
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
                Templ,
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
            warn!(target: SPRITE_INIT, "SpriteConfig name is empty for SpriteConfig '{}', using StrId as name", str_id);
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
        cmd.entity(spritecfg_ent).insert(BaseMovementSpeed(seri.baseline_move_speed));
        if !seri.parent_cat.trim().is_empty() {
            let to_become_child = BecomeChildOfSpriteWithTag(Tag::trunc(seri.parent_cat.trim()));
            cmd.entity(spritecfg_ent).insert(to_become_child);
        }

        let fallback_img_path = seri.fallback_img_path.trim();
        let has_fallback = !fallback_img_path.is_empty();
        if has_fallback {
            let Ok(img_path_holder) = PathHolder::new(fallback_img_path.to_string()) else {
                error!(target: SPRITE_INIT, "SpriteConfig '{}' fallback_img_path '{}' is invalid", str_id, fallback_img_path);
                continue;
            };
            cmd.entity(spritecfg_ent).insert((UseFallbackSprite, img_path_holder));
            if seri.z.is_finite() {
                cmd.entity(spritecfg_ent).insert(AcZ(seri.z));
            }
            if seri.y_sort.is_finite() {
                cmd.entity(spritecfg_ent).insert(YSortOrigin(seri.y_sort));
            }
        }

        if !seri.mapped_anims.is_empty() {
            let mut anims_map = MappedAnimations::default();
            for (anim_type, anim_id) in seri.mapped_anims {
                let anim_type = AnimType::from_tuple(anim_type);
                let anim_id = StrId::trunc(anim_id);
                let Ok(&anim_ent) = library.0.get(&anim_id) else {
                    error!(target: SPRITE_INIT, "SpriteConfig {}: AcAnimationEntityMap does not contain: {} ", str_id, anim_id);
                    continue;
                };
                anims_map.0.insert(anim_type, anim_ent);
            }
            if anims_map.0.is_empty() {
                error!(target: SPRITE_INIT, "SpriteConfig '{}' animations map has no valid entries", str_id);
            } else {
                cmd.entity(spritecfg_ent).insert(anims_map);
            }
        } else if !has_fallback {
            error!(target: SPRITE_INIT, "SpriteConfig '{}' needs either mapped_anims or fallback_img_path", str_id);
        }

        if !seri.children_sprites.is_empty() {
            let ids = SampleSpritesFromStrIds::new(seri.children_sprites.clone());
            cmd.entity(spritecfg_ent).insert(ids);
        }
        if !seri.sfx_every_n_frames.paths.is_empty() {
            cmd.entity(spritecfg_ent).insert(SpriteAnimSfx {
                sound_paths: seri.sfx_every_n_frames.paths.clone(),
                every_n_frame_changes: seri.sfx_every_n_frames.n.max(0.001),
            });
        }
        if !seri.loop_sfx.paths.is_empty() {
            cmd.entity(spritecfg_ent).insert(SpriteLoopSfx {
                sound_paths: seri.loop_sfx.paths.clone(),
                condition: SfxPlayCondition::from(seri.loop_sfx.condition.as_str()),
            });
        }
        if !seri.interval_sfx.paths.is_empty() {
            cmd.entity(spritecfg_ent).insert(SpriteTimedSfx {
                sound_paths: seri.interval_sfx.paths.clone(),
                condition: SfxPlayCondition::from(seri.interval_sfx.condition.as_str()),
                time_interval_secs: seri.interval_sfx.secs.max(0.001),
                scale_interval_with_animation_speed: seri.interval_sfx.shorten_with_anim_playing_speed,
            });
        }
    }
    cmd.try_insert_batch(comps_to_insert);
}
