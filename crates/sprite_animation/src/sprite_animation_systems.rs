

use bevy::{ecs::{entity::EntityHashSet, system::SystemParam}, };
use bevy_replicon::prelude::*;
use ac_audio::ac_audio_components::{AnimationFrameSfxState, AnimationSeriSfxConfig, AnimationSeriSfxState};
use being_shared::{Grounding, ComputedBy, ComputedLocally};
#[allow(unused_imports)] use bevy::prelude::*;
use bevy_spritesheet_animation::{prelude::*, };
use common::{SPRITE_ANIMATION_SYSTEM, common_components::*, file_logging::file_log};
use game_common::{game_common_components::{Directionable, TemplEntiRef, }, Templ};
use being_shared::movement_shared_components::SpeedMagnitude;
use player_shared::player_components::*;
use ::sprite_animation_shared::*;
use ::sprite_shared::*;
use ::tilemap_shared::directions::*;

#[allow(unused_parens, )]
#[derive(SystemParam)]
pub struct SpriteAnimationQueries<'w, 's> {
    pub sprite_entities_query: Query<'w, 's, (Entity, &'static BaseHolderRef, &'static TemplEntiRef)>,
    pub sprite_query: Query<'w, 's, &'static mut Sprite>,
    pub sprite_animation_query: Query<'w, 's, &'static mut SpritesheetAnimation>,
    pub sprite_progress_query: Query<'w, 's, &'static mut AcAnimationProgresses>,
    pub ac_z_query: Query<'w, 's, &'static mut AcZ>,
    pub y_sort_query: Query<'w, 's, &'static mut YSortOrigin>,
    pub transform_query: Query<'w, 's, &'static mut Transform>,
    pub frame_sfx_query: Query<'w, 's, &'static mut AnimationFrameSfxState>,
    pub seri_sfx_config_query: Query<'w, 's, &'static mut AnimationSeriSfxConfig>,
    pub seri_sfx_state_query: Query<'w, 's, &'static mut AnimationSeriSfxState>,
    pub anim_extra_state_query: Query<'w, 's, &'static AnimExtraState>,
    pub playing_speed_query: Query<'w, 's, &'static PlayingSpeed>,
    pub anim_handle_sheet_query: Query<'w, 's, (&'static AnimationHandle, &'static AnimationSheet)>,
    pub clip_start_frames_query: Query<'w, 's, &'static ClipStartFrames>,
    pub save_anim_progress_query: Query<'w, 's, &'static SaveAnimationProgress>,
    pub alternating_config_query: Query<'w, 's, &'static AlternatingStartFramesConfig>,
    pub alternating_state_query: Query<'w, 's, &'static mut AlternatingStartFramesState>,
    pub animation_seri_query: Query<'w, 's, &'static AnimationSeri>,
}



pub type HoldersChangeFilter = (Without<Templ>, Or<(Changed<HeldSprites>, Changed<Grounding>, Changed<MoveAnimActive>, Changed<CardinalDirection>, Changed<SpeedMagnitude>)>);

#[allow(unused_parens, )]
pub fn switch_or_readjust_sprite_animation(
    mut cmd: Commands, asset_server: Res<AssetServer>,
    changed_entities: Query<Entity, (HoldersChangeFilter)>,
    changed_sprite_cfg_refs: Query<&BaseHolderRef, (Changed<TemplEntiRef>, Without<SpriteConfig>, Without<Templ>)>,
    animation_map: Res<AcAnimationEntityMap>,
    queries: SpriteAnimationQueries,


    base: Query<(&HeldSprites, Option<&CardinalDirection>, Option<&MoveAnimActive>, Option<&Grounding>, ), (Without<Templ>)>,
    spriteconfig: Query<(
        Option<&MappedAnimations>,
        Has<UseFallbackSprite>,
        Option<&PathHolder>,
        Has<Directionable>,
        Has<MovementBased>,
        Has<GroundingBased>,
        Option<&BaseMovementSpeed>,
        Option<&SpriteAnimSfx>
    ), (With<SpriteConfig>)>,
    baseline_speed_query: Query<&SpeedMagnitude>,
    strid_query: Query<&StrId>,

    mut atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    images: Res<Assets<Image>>,
    mut sprite_entis_to_iter: Local<EntityHashSet>,
) {
    let SpriteAnimationQueries {
        sprite_entities_query,
        mut sprite_query,
        mut sprite_animation_query,
        mut sprite_progress_query,
        mut ac_z_query,
        mut y_sort_query,
        mut transform_query,
        mut frame_sfx_query,
        mut seri_sfx_config_query,
        mut seri_sfx_state_query,
        anim_extra_state_query,
        playing_speed_query,
        anim_handle_sheet_query,
        clip_start_frames_query,
        save_anim_progress_query,
        alternating_config_query,
        mut alternating_state_query,
        animation_seri_query,
    } = queries;

    let changed_entities_iter = changed_entities.iter();
    let cfg_refs_iter = changed_sprite_cfg_refs.iter();
    sprite_entis_to_iter.reserve(
        changed_entities_iter.size_hint().1.unwrap_or(changed_entities_iter.size_hint().0)
            + cfg_refs_iter.size_hint().1.unwrap_or(cfg_refs_iter.size_hint().0)
    );
    sprite_entis_to_iter.extend(changed_entities_iter);
    sprite_entis_to_iter.extend(cfg_refs_iter.map(|base_holder| base_holder.base));

    for (held_sprites, direction, moving, grounding) in base.iter_many(sprite_entis_to_iter.drain()) {
        for held_sprite in held_sprites.iter() {
            let held_sprite_strid = strid_query.get(held_sprite).ok().cloned().unwrap_or_default();
            let Ok((ent, base_holder, sprite_cfg_ref)) = sprite_entities_query.get(held_sprite)
            else { error_once!(target: SPRITE_ANIMATION_SYSTEM, "Failed to get sprite entity {:?} {}", held_sprite, held_sprite_strid); continue };
            let state_id = anim_extra_state_query.get(held_sprite).ok();
            let playing_speed = playing_speed_query.get(held_sprite).ok().copied();
            let animation_progresses = sprite_progress_query.get_mut(held_sprite).ok();
            let sprite_comp = sprite_query.get_mut(held_sprite).ok();
            let mut prev_anim = sprite_animation_query.get_mut(held_sprite).ok();
            let transform = transform_query.get_mut(held_sprite).ok();


            let Ok((sprite_cfg_animations_map, has_fallback, fallback_img_path, directionable, movement_based, grounding_based, baseline_move_speed, sprite_cfg_sfx)) = spriteconfig.get(sprite_cfg_ref.0)
            else {
                /*
                let sprite_cfg_strid = strid_query.get(sprite_cfg_ref.0).ok().cloned().unwrap_or_default();
                warn_once!(target: SPRITE_ANIMATION_SYSTEM, "Failed to get SpriteConfigRef entity {:?} {}", sprite_cfg_ref.0, sprite_cfg_strid);
                */
                continue
            };
            let Some(sprite_cfg_animations_map) = sprite_cfg_animations_map else {
                if let Some(fallback_img_path) = fallback_img_path {
                    if sprite_comp.is_none() {
                        cmd.entity(ent).insert(Sprite {
                            image: asset_server.load(fallback_img_path.path()),
                            ..Default::default()
                        });
                    }
                    continue;
                }
                let sprite_cfg_strid = strid_query.get(sprite_cfg_ref.0).ok().cloned().unwrap_or_default();
                error_once!(target: SPRITE_ANIMATION_SYSTEM, "SpriteConfig {:?} {} has no MappedAnimations and no image path", sprite_cfg_ref.0, sprite_cfg_strid);
                continue;
            };

            let anim_type = AnimType {//is OK
                direction: if directionable { direction.copied().unwrap_or_default() } else { CardinalDirection::default() },
                moving: if movement_based { moving.copied().unwrap_or_default() } else { MoveAnimActive::default() },
                grounding: if grounding_based { grounding.copied().unwrap_or_default() } else { Grounding::default() },
                state_id: state_id.cloned(),
            };

            let Some(anim_hash) = sprite_cfg_animations_map.0.get(&anim_type) else {
                file_log(
                    "move",
                    "sprite",
                    &format!(
                        "no_anim base={:?} sprite={ent:?} dir={:?} moving={:?} grounding={:?} state={:?}",
                        base_holder.base,
                        anim_type.direction,
                        anim_type.moving,
                        anim_type.grounding,
                        anim_type.state_id,
                    ),
                );
                if !has_fallback {
                    warn_once!(target: SPRITE_ANIMATION_SYSTEM, "No animation found for AnimType {:?} in SpriteCfgAnimationsMap for entity {:?} {}", anim_type, ent, held_sprite_strid);
                }
                continue;
            };
            let Ok(&anim_ent) = animation_map.0.get(*anim_hash) else {
                error_once!(target: SPRITE_ANIMATION_SYSTEM, "Failed to resolve animation hash {} for sprite config {:?} {}", anim_hash, sprite_cfg_ref.0, held_sprite_strid);
                continue;
            };
            file_log(
                "move",
                "sprite",
                &format!(
                    "select_anim base={:?} sprite={ent:?} anim={anim_ent:?} dir={:?} moving={:?} grounding={:?} state={:?}",
                    base_holder.base,
                    anim_type.direction,
                    anim_type.moving,
                    anim_type.grounding,
                    anim_type.state_id,
                ),
            );
            let Ok((anim_handle, anim_sheet)) = anim_handle_sheet_query.get(anim_ent) else {
                let anim_strid = strid_query.get(anim_ent).ok().cloned().unwrap_or_default();
                error_once!(target: SPRITE_ANIMATION_SYSTEM, "Failed to get animation data for animation entity {:?} {}", anim_ent, anim_strid);
                continue;
            };
            let ac_z = ac_z_query.get(anim_ent).ok().copied();
            let y_sort = y_sort_query.get(anim_ent).ok().copied();
            let clip_start_frames = clip_start_frames_query.get(anim_ent).ok();
            let should_save_anim_progress = save_anim_progress_query.get(anim_ent).is_ok();
            let alternating_config = alternating_config_query.get(anim_ent).ok();
            let mut alternating_state = alternating_state_query.get_mut(anim_ent).ok();
            let anim_playing_speed = playing_speed_query.get(anim_ent).ok().copied();
            let anim_seri = animation_seri_query.get(anim_ent).ok();

            let Some(sprite) = anim_sheet.0.with_loaded_image(&images) else {
                let anim_strid = strid_query.get(anim_ent).ok().cloned().unwrap_or_default();
                error_once!(target: SPRITE_ANIMATION_SYSTEM, "Failed to create sprite for animation entity {:?} {} because image is not loaded yet.", anim_ent, anim_strid);
                continue;
            };
            let mut new_sprite = sprite.sprite(&mut atlas_layouts);
            if let Some(anim_seri) = anim_seri {
                new_sprite.flip_x = anim_seri.flip_x;
                new_sprite.flip_y = anim_seri.flip_y;
            }

            let (start_frame, should_update_alternating_state) = {
                let base_frame = clip_start_frames
                    .and_then(|csf| csf.0.first().copied())
                    .unwrap_or(0);

                // Global alternating start frames (animation-level)
                if let Some(alt_config) = alternating_config {
                    if let Some((frame1, frame2)) = alt_config.0.first().copied().flatten() {
                        if let Some(ref alt_state) = alternating_state {
                            let current_index = alt_state.0.get(0).copied().unwrap_or(0);
                            let frame = if current_index == 0 { frame1 } else { frame2 };
                            (frame, true)
                        } else {
                            // Fallback if no alternating state (shouldn't happen)
                            (frame1, false)
                        }
                    } else {
                        (base_frame, false)
                    }
                } else {
                    (base_frame, false)
                }
            };
            let sprite_playing_speed = playing_speed.map(|speed| speed.0).unwrap_or(1.0);
            let anim_playing_speed = anim_playing_speed.map(|speed| speed.0).unwrap_or(PlayingSpeed::default().0);

            let speed_factor = sprite_playing_speed * anim_playing_speed;
            let speed_factor = if movement_based && anim_type.moving.get() {
                if let Some(baseline_move_speed) = baseline_move_speed {
                    if baseline_move_speed.0 <= 0.01 {
                        speed_factor
                    } else {
                        let current_speed = baseline_speed_query
                            .get(base_holder.base)
                            .map_or(baseline_move_speed.0, |speed| speed.0.max(0.0));
                        speed_factor * (current_speed / baseline_move_speed.0)
                    }
                } else {
                    speed_factor
                }
            } else {
                speed_factor
            };
            let Sprite {
                image,
                texture_atlas,
                color,
                flip_x,
                flip_y,
                custom_size,
                rect,
                image_mode,
            } = new_sprite;
            let animation = anim_handle.0.clone();
            let mut progress = AnimationProgress {
                frame: start_frame,
                repetition: 0,
            };
            let playing = !anim_seri.map(|seri| seri.paused).unwrap_or(false);

            let mut insert_needed = false;

            if let Some(prev_animation) = prev_anim.as_mut() {
                if prev_animation.animation != anim_handle.0 {
                    if let Some(mut anim_progresses) = animation_progresses {
                        if should_save_anim_progress {
                            anim_progresses.0.insert(prev_animation.animation.clone(), prev_animation.progress);

                            if let Some(stored_progress) = anim_progresses.0.get(&anim_handle.0) {
                                progress = *stored_progress;
                            }
                        }
                    }
                    insert_needed = true;
                } else {
                    if (prev_animation.speed_factor - speed_factor).abs() > f32::EPSILON {
                        prev_animation.speed_factor = speed_factor;
                    }
                    if prev_animation.playing != playing {
                        prev_animation.playing = playing;
                    }
                }
            } else {
                if should_save_anim_progress {
                    if let Some(anim_progresses) = animation_progresses {
                        if let Some(stored_progress) = anim_progresses.0.get(&anim_handle.0) {
                            progress = *stored_progress;
                        }
                    }
                }
                insert_needed = true;
            }
            let cardinal_rotation = anim_seri
                .map(|seri| seri.cardinal_rotation)
                .unwrap_or_default();

            if let Some(mut transform) = transform {
                transform.rotation = cardinal_rotation
                    .angle()
                    .map(Quat::from_rotation_z)
                    .unwrap_or(Quat::IDENTITY);
            }


            if insert_needed {
                let initial_frame = progress.frame;

                if let Some(mut sprite_comp) = sprite_comp {
                    sprite_comp.image = image.clone();
                    sprite_comp.texture_atlas = texture_atlas.clone();
                    sprite_comp.color = color;
                    sprite_comp.flip_x = flip_x;
                    sprite_comp.flip_y = flip_y;
                    sprite_comp.custom_size = custom_size;
                    sprite_comp.rect = rect;
                    sprite_comp.image_mode = image_mode;
                } else {
                    cmd.entity(ent).try_insert(Sprite {
                        image,
                        texture_atlas,
                        color,
                        flip_x,
                        flip_y,
                        custom_size,
                        rect,
                        image_mode,
                    });
                }

                if let Some(prev_animation) = prev_anim.as_mut() {
                    prev_animation.animation = animation.clone();
                    prev_animation.progress = progress;
                    prev_animation.playing = playing;
                    prev_animation.speed_factor = speed_factor;
                } else {
                    cmd.entity(ent).try_insert(SpritesheetAnimation {
                        animation,
                        progress,
                        playing,
                        speed_factor,
                    });
                }

                if let Some(ac_z) = ac_z {
                    if let Ok(mut prev_z) = ac_z_query.get_mut(ent) {
                        prev_z.0 = ac_z.0;
                    } else {
                        cmd.entity(ent).insert(ac_z);
                    }
                } else {
                    cmd.entity(ent).try_remove::<AcZ>();
                }

                if let Some(y_sort) = y_sort {
                    if let Ok(mut prev_y_sort) = y_sort_query.get_mut(ent) {
                        prev_y_sort.0 = y_sort.0;
                    } else {
                        cmd.entity(ent).insert(y_sort);
                    }
                } else {
                    cmd.entity(ent).try_remove::<YSortOrigin>();
                }

                if sprite_cfg_sfx.is_some() {
                    if let Ok(mut frame_sfx_state) = frame_sfx_query.get_mut(ent) {
                        frame_sfx_state.last_frame = initial_frame;
                        frame_sfx_state.frame_changes_acc = 0.0;
                    } else {
                        cmd.entity(ent).insert(AnimationFrameSfxState {
                            last_frame: initial_frame,
                            frame_changes_acc: 0.0,
                        });
                    }
                }
                if let Some(anim_seri) = anim_seri {
                    if anim_seri.sound_effects.is_empty() {
                        cmd.entity(ent).remove::<(AnimationSeriSfxConfig, AnimationSeriSfxState)>();
                    } else {
                        if let Ok(mut seri_sfx_config) = seri_sfx_config_query.get_mut(ent) {
                            seri_sfx_config.sound_paths = anim_seri.sound_effects.clone();
                            seri_sfx_config.every_n_frame_changes = anim_seri.sound_effects_every_n_frames.max(0.001);
                        } else {
                            cmd.entity(ent).insert(AnimationSeriSfxConfig {
                                sound_paths: anim_seri.sound_effects.clone(),
                                every_n_frame_changes: anim_seri.sound_effects_every_n_frames.max(0.001),
                            });
                        }
                        if let Ok(mut seri_sfx_state) = seri_sfx_state_query.get_mut(ent) {
                            seri_sfx_state.last_frame = initial_frame;
                            seri_sfx_state.frame_changes_acc = 0.0;
                        } else {
                            cmd.entity(ent).insert(AnimationSeriSfxState {
                                last_frame: initial_frame,
                                frame_changes_acc: 0.0,
                            });
                        }
                    }
                }
                if should_update_alternating_state {
                    if let Some(alt_state) = alternating_state.as_mut() {
                        if !alt_state.0.is_empty() {
                            alt_state.0[0] = (alt_state.0[0] + 1) % 2;
                        }
                    }
                }
            }
        }
    }
}
#[allow(unused_parens)]
pub fn msg_movestate_update_to_clients_for_sprite_animation(
    connected: Query<&Player, Without<Mine>>,
    changed_entities: Query<Entity, (HoldersChangeFilter, )>,
    bases_query: Query<(Entity, &MoveAnimActive, Option<&Grounding>, Option<&CardinalDirection>, Option<&StrId>), (HoldersChangeFilter)>,
    changed_sprite_cfg_refs: Query<&BaseHolderRef, (Changed<TemplEntiRef>, Without<SpriteConfig>, Without<Templ>)>,

    controller: Query<&ComputedBy>,
    mut mwriter: MessageWriter<ToClients<SyncMoveState>>,
    mut messages_to_send: Local<Vec<ToClients<SyncMoveState>>>,
    mut entis_to_iter: Local<EntityHashSet>,
){
    if connected.is_empty() { return; }
    entis_to_iter.clear();
    let changed_entities_iter = changed_entities.iter();
    let cfg_refs_iter = changed_sprite_cfg_refs.iter();
    entis_to_iter.reserve(changed_entities_iter.size_hint().1.unwrap_or(changed_entities_iter.size_hint().0) + cfg_refs_iter.size_hint().1.unwrap_or(cfg_refs_iter.size_hint().0));
    entis_to_iter.extend(changed_entities_iter);
    entis_to_iter.extend(cfg_refs_iter.map(|base_holder| base_holder.base));

    for (being_ent, &moving, grounding, direction, id) in bases_query.iter_many(entis_to_iter.iter()) {
        let moving = moving.get();
        let event_data = SyncMoveState {being_ent, moving, grounding: grounding.cloned(), direction: direction.cloned()};
        if let Ok(controller) = controller.get(being_ent) {
            messages_to_send.push(ToClients {
                targets: SendTargets::AllExcept(ClientId::Client(controller.client_ent)),
                message: event_data,
            });
            trace!(target: SPRITE_ANIMATION_SYSTEM, "Sending moving {} for entity {:?} {} to all clients except {:?}", moving, being_ent, id.cloned().unwrap_or_default(), controller.client_ent);
        }
        else {
            messages_to_send.push(ToClients { targets: SendTargets::All, message: event_data, });
            trace!(target: SPRITE_ANIMATION_SYSTEM, "Sending moving {} for entity {:?} to all clients", moving, being_ent);
        }
    }
    mwriter.write_batch(messages_to_send.drain(..));
}
#[allow(unused_parens, )]
pub fn client_receive_moving_anim(
    mut mreader: MessageReader<SyncMoveState>,
    mut query: Query<(&mut MoveAnimActive, &mut Grounding, &mut CardinalDirection), Without<ComputedLocally>>,
) {
    for message in mreader.par_read() {
        let SyncMoveState { being_ent, moving, grounding, direction } = message.0;
        trace!(target: SPRITE_ANIMATION_SYSTEM, "Received moving {} for entity {:?}", moving, being_ent);

        if let Ok((mut move_anim, mut grounding_comp, mut direction_comp)) = query.get_mut(*being_ent) {
            move_anim.set(*moving);
            if let Some(grounding) = grounding {
                *grounding_comp = *grounding;
            }
            if let Some(direction) = direction {
                *direction_comp = *direction;
            }
        } else {
            warn_once!("Received moving state for entity {:?} that does not exist in this client.", being_ent);
        }
    }
}
