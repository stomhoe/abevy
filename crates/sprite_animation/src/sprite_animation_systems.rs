

use bevy::{ecs::entity::EntityHashSet, platform::collections::HashSet};
use bevy_replicon::prelude::*;
use ac_audio::ac_audio_components::{AnimationFrameSfxState, AnimationSeriSfxConfig, AnimationSeriSfxState};
use being_shared::{Grounding, ControlledBy};
#[allow(unused_imports)] use bevy::prelude::*;
use bevy_spritesheet_animation::{prelude::*, };
use common::{SPRITE_ANIMATION_SYSTEM, common_components::*};
use game_common::game_common_components::{Directionable, EntityZeroRef, };
use movement::movement_components::MoveVecMag;
use player::player_components::*;
use sprite::sprite_components::*;
use ::sprite_animation_shared::*;
use ::sprite_shared::*;
use ::tilemap_shared::directions::*;

//TODO hacer animation speed para walking proporcional a la velocidad real del being

#[allow(unused_parens, )]
pub fn animate_sprite(
    mut cmd: Commands,
    asset_server: Res<AssetServer>,

    mut move_anims_changed: MessageReader<BeingChangedMoveState>,
    changers: Query<Entity, Or<(Changed<HeldSprites>, Changed<Grounding>, )>>,
    changed_sprite_cfg_refs: Query<&BaseHolderRef, (Changed<EntityZeroRef>, Without<SpriteConfig>)>,

    base: Query<(&HeldSprites, Option<&CardinalDirection>, Option<&MoveAnimActive>, Option<&Grounding>, ), ()>,

    mut sprites_query: Query<(
        Entity,
        Has<Sprite>,
        Option<&mut SpritesheetAnimation>,
        &BaseHolderRef,
        &EntityZeroRef,
        Option<&AnimExtraState>,
        Option<&PlayingSpeed>,
        Option<&mut AcAnimationProgresses>,
        Option<&mut Transform>,
    ), ()>,

    spriteconfig: Query<(
        Option<&MappedAnimations>,
        Has<UseFallbackSprite>,
        Option<&ImagePathHolder>,
        Has<Directionable>,
        Has<MovementBased>,
        Has<GroundingBased>,
        Option<&BaseMovementSpeed>,
        Option<&SpriteAnimationSfx>
    ), ()>,
    base_movevec_query: Query<&MoveVecMag>,
    strid_query: Query<&StrId>,

    mut animation_query: Query<(&StrId, &AnimationHandle, &AnimationSheet, &AcZ, Option<&YSortOrigin>, Option<&ClipStartFrames>, Has<SaveAnimationProgress>, Option<&AlternatingStartFramesConfig>, Option<&mut AlternatingStartFramesState>, Option<&PlayingSpeed>, Option<&AnimationSeri>),()>,

    mut atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    images: Res<Assets<Image>>,
) {

    let mut entis_to_iter = EntityHashSet::with_capacity(
        changers.iter().size_hint().0 + move_anims_changed.len() + changed_sprite_cfg_refs.iter().size_hint().0
    );
    entis_to_iter.extend(changers.iter());
    entis_to_iter.extend(move_anims_changed.read().map(|f| f.0));
    entis_to_iter.extend(changed_sprite_cfg_refs.iter().map(|base_holder| base_holder.base));

    for (held_sprites, direction, moving, grounding) in base.iter_many(entis_to_iter) {
        for held_sprite in held_sprites.entities() {
            let held_sprite_strid = strid_query.get(*held_sprite).ok().cloned().unwrap_or_default();
            let Ok((ent, has_sprite, prev_anim, base_holder, sprite_cfg_ref, state_id, playing_speed, animation_progresses, transform, )) = sprites_query.get_mut(*held_sprite)
            else { error_once!(target: SPRITE_ANIMATION_SYSTEM, "Failed to get sprite entity {:?} {}", held_sprite, held_sprite_strid); continue };


            let Ok((sprite_cfg_animations_map, has_fallback, fallback_img_path, directionable, movement_based, grounding_based, base_movement_speed, sprite_cfg_sfx)) = spriteconfig.get(sprite_cfg_ref.0)
            else {
                let sprite_cfg_strid = strid_query.get(sprite_cfg_ref.0).ok().cloned().unwrap_or_default();
                error_once!(target: SPRITE_ANIMATION_SYSTEM, "Failed to get SpriteConfigRef entity {:?} {}", sprite_cfg_ref.0, sprite_cfg_strid);
                continue
            };
            let Some(sprite_cfg_animations_map) = sprite_cfg_animations_map else {
                if has_fallback {
                    if !has_sprite {
                        let Some(fallback_img_path) = fallback_img_path else {
                            continue;
                        };
                        cmd.entity(ent).insert(Sprite {
                            image: asset_server.load(fallback_img_path.path()),
                            ..Default::default()
                        });
                    }
                    continue;
                }
                let sprite_cfg_strid = strid_query.get(sprite_cfg_ref.0).ok().cloned().unwrap_or_default();
                error_once!(target: SPRITE_ANIMATION_SYSTEM, "SpriteConfig {:?} {} has no MappedAnimations and no fallback image", sprite_cfg_ref.0, sprite_cfg_strid);
                continue;
            };

            let anim_type = AnimType {
                direction: if directionable { direction.copied().unwrap_or_default() } else { CardinalDirection::default() },
                moving: if movement_based { moving.copied().unwrap_or_default() } else { MoveAnimActive::default() },
                grounding: if grounding_based { grounding.copied().unwrap_or_default() } else { Grounding::default() },
                state_id: state_id.cloned(),
            };
            trace!(target: SPRITE_ANIMATION_SYSTEM, "Determined AnimType {:?} for sprite entity {:?} {}", anim_type, ent, held_sprite_strid);

            let Some(anim_ent) = sprite_cfg_animations_map.0.get(&anim_type) else {
                if !has_fallback {
                    warn_once!(target: SPRITE_ANIMATION_SYSTEM, "No animation found for AnimType {:?} in SpriteCfgAnimationsMap for entity {:?} {}", anim_type, ent, held_sprite_strid);
                }
                continue;
            };

            let Ok((_, anim_handle, anim_sheet, z, y_sort, clip_start_frames, should_save_anim_progress, alternating_config, mut alternating_state, anim_playing_speed, anim_seri )) = animation_query.get_mut(*anim_ent) else {
                let anim_strid = strid_query.get(*anim_ent).ok().cloned().unwrap_or_default();
                error_once!(target: SPRITE_ANIMATION_SYSTEM, "Failed to get animation data for animation entity {:?} {}", anim_ent, anim_strid);
                continue;
            };

            let Some(sprite) = anim_sheet.0.with_loaded_image(&images) else {
                let anim_strid = strid_query.get(*anim_ent).ok().cloned().unwrap_or_default();
                error_once!(target: SPRITE_ANIMATION_SYSTEM, "Failed to create sprite for animation entity {:?} {} because image is not loaded yet.", anim_ent, anim_strid);
                continue;
            };
            let mut sprite = sprite.sprite(&mut atlas_layouts);
            if let Some(anim_seri) = anim_seri {
                sprite.flip_x = anim_seri.flip_x;
                sprite.flip_y = anim_seri.flip_y;
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

            let base_anim_speed = anim_seri.map(|seri| seri.speed).unwrap_or(PlayingSpeed::default().0);
            let speed_factor = playing_speed
                .map(|speed| speed.0)
                .or_else(|| anim_playing_speed.map(|speed| speed.0))
                .unwrap_or(base_anim_speed);
            let speed_factor = if movement_based && anim_type.moving.get() {
                if let Some(base_speed) = base_movement_speed {
                    if base_speed.0 <= 0.01 {
                        speed_factor
                    } else {
                        let current_speed = base_movevec_query
                            .get(base_holder.base)
                            .map_or(base_speed.0, |m| m.speed_magnitude.max(0.0));
                        speed_factor * (current_speed / base_speed.0)
                    }
                } else {
                    speed_factor
                }
            } else {
                speed_factor
            };

            let playing = !anim_seri.map(|seri| seri.paused).unwrap_or(false);

            let mut spritesheet_animation =
            SpritesheetAnimation{
                animation: anim_handle.0.clone(),
                progress: AnimationProgress {
                    frame: start_frame,
                    repetition: 0,
                },
                playing,
                speed_factor,
            };

            let mut insert_needed = false;


            if let Some(mut prev_animation) = prev_anim {
                if prev_animation.animation != anim_handle.0 {
                    if let Some(mut anim_progresses) = animation_progresses {
                        if should_save_anim_progress {
                            anim_progresses.0.insert(prev_animation.animation.clone(), prev_animation.progress);

                            if let Some(stored_progress) = anim_progresses.0.get(&anim_handle.0) {
                                spritesheet_animation.progress = *stored_progress;
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
                            spritesheet_animation.progress = *stored_progress;
                        }
                    }
                }
                insert_needed = true;
            }
            let target_direction = anim_seri
                .map(|seri| seri.cardinal_rotation)
                .unwrap_or(CardinalDirection::South);

            if let Some(mut transform) = transform {
                if target_direction != CardinalDirection::South {
                    transform.rotation = Quat::from_rotation_z(cardinal_rotation_angle(target_direction));
                } else {
                    transform.rotation = Quat::IDENTITY;
                }
            }

            if target_direction != CardinalDirection::South {
                cmd.entity(ent).insert(target_direction);
            } else {
                cmd.entity(ent).remove::<CardinalDirection>();
            }

            if insert_needed {
                let initial_frame = spritesheet_animation.progress.frame;
                cmd.entity(ent).try_insert((sprite, spritesheet_animation, z.clone(), y_sort.cloned().unwrap_or_default()));
                if sprite_cfg_sfx.is_some() {
                    cmd.entity(ent).insert(AnimationFrameSfxState {
                        last_frame: initial_frame,
                        frame_changes_acc: 0.0,
                    });
                }
                if let Some(anim_seri) = anim_seri {
                    if anim_seri.sound_effects.is_empty() {
                        cmd.entity(ent).remove::<AnimationSeriSfxConfig>();
                        cmd.entity(ent).remove::<AnimationSeriSfxState>();
                    } else {
                        cmd.entity(ent).insert(AnimationSeriSfxConfig {
                            sound_paths: anim_seri.sound_effects.clone(),
                            every_n_frame_changes: anim_seri.sound_effects_every_n_frames.max(0.001),
                        });
                        cmd.entity(ent).insert(AnimationSeriSfxState {
                            last_frame: initial_frame,
                            frame_changes_acc: 0.0,
                        });
                    }
                }

                // Update alternating state after using it
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

fn cardinal_rotation_angle(direction: CardinalDirection) -> f32 {
    match direction {
        CardinalDirection::South => 0.0,
        CardinalDirection::West => std::f32::consts::FRAC_PI_2,
        CardinalDirection::North => std::f32::consts::PI,
        CardinalDirection::East => -std::f32::consts::FRAC_PI_2,
    }
}
#[allow(unused_parens)]
pub fn update_animstate_for_clients(
    connected: Query<&Player, Without<Mine>>,
    mut move_anims_changed: MessageReader<BeingChangedMoveState>,
    changers: Query<Entity, Or<(Changed<HeldSprites>, Changed<Grounding>, )>>,

    bases_query: Query<(Entity, &MoveAnimActive, Option<&Grounding>, Option<&CardinalDirection>, Option<&StrId>)>,
    controller: Query<&ControlledBy>,
    mut mwriter: MessageWriter<ToClients<SyncMoveState>>,
){
    if connected.is_empty() { return; }

    let mut messages_to_send = Vec::new();
    let mut entis_to_iter = EntityHashSet::with_capacity(changers.iter().size_hint().0 + move_anims_changed.len());
    entis_to_iter.extend(changers.iter());
    entis_to_iter.extend(move_anims_changed.read().map(|f| f.0));

    for (being_ent, &moving, grounding, direction, id) in bases_query.iter_many(entis_to_iter) {
        let moving = moving.get();
        let event_data = SyncMoveState {being_ent, moving, grounding: grounding.cloned(), direction: direction.cloned()};
        if let Ok(controller) = controller.get(being_ent) {
            messages_to_send.push(ToClients {
                mode: SendMode::BroadcastExcept(ClientId::Client(controller.client_ent)),
                message: event_data,
            });
            trace!(target: SPRITE_ANIMATION_SYSTEM, "Sending moving {} for entity {:?} {} to all clients except {:?}", moving, being_ent, id.cloned().unwrap_or_default(), controller.client_ent);
        }
        else {
            messages_to_send.push(ToClients { mode: SendMode::Broadcast, message: event_data, });
            trace!(target: SPRITE_ANIMATION_SYSTEM, "Sending moving {} for entity {:?} to all clients", moving, being_ent);
        }
    }
    mwriter.write_batch(messages_to_send);
}
#[allow(unused_parens, )]
pub fn client_receive_moving_anim(
    mut mreader: MessageReader<SyncMoveState>,
    mut beings_changed_move_state_writer: MessageWriter<BeingChangedMoveState>,
    mut query: Query<(&mut MoveAnimActive, &mut Grounding, &mut CardinalDirection)>,
) {
    let mut being_changed_state_set: HashSet<BeingChangedMoveState> = HashSet::new();

    for message in mreader.par_read() {
        let SyncMoveState { being_ent, moving, grounding, direction } = message.0;
        trace!(target: SPRITE_ANIMATION_SYSTEM, "Received moving {} for entity {:?}", moving, being_ent);

        if let Ok((mut move_anim, mut grounding_comp, mut direction_comp)) = query.get_mut(*being_ent) {
            move_anim.set(*moving, *being_ent, &mut being_changed_state_set);
            if let Some(grounding) = grounding {
                *grounding_comp = *grounding;
            }
            if let Some(direction) = direction {
                *direction_comp = *direction;
            }
        } else {
            warn!("Received moving state for entity {:?} that does not exist in this client.", being_ent);
        }

    }
    beings_changed_move_state_writer.write_batch(being_changed_state_set);


}
