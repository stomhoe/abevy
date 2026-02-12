

use bevy::{ecs::entity::EntityHashSet, platform::collections::HashSet};
use bevy_replicon::prelude::*;
use being_shared::{Grounding, ControlledBy};
#[allow(unused_imports)] use bevy::prelude::*;
use bevy_spritesheet_animation::{prelude::*, };
use common::{SPRITE_ANIMATION_SYSTEM, common_components::*};
use game_common::game_common_components::{Directionable, EntityZeroRef, };
use player::player_components::*;
use sprite::sprite_components::*;
use ::sprite_animation_shared::*;
use ::sprite_shared::*;
use ::tilemap_shared::directions::*;

#[allow(unused_imports, )]
use crate::{sprite_animation_components::*, sprite_animation_messages::*, };

//TODO hacer animation speed para walking proporcional a la velocidad real del being

#[allow(unused_parens, )]
pub fn animate_sprite(
    mut cmd: Commands,

    mut move_anims_changed: MessageReader<BeingChangedMoveState>,
    changers: Query<Entity, Or<(Changed<HeldSprites>, Changed<Grounding>, )>>,

    base: Query<(&HeldSprites, Option<&CardinalDirection>, Option<&MoveAnimActive>, &Grounding, ), ()>,

    mut sprites_query: Query<(Entity, Option<&mut SpritesheetAnimation>, &EntityZeroRef,
        Option<&AnimExtraState>, Option<&PlayingSpeed>, Option<&mut AcAnimationProgresses>, Has<SpriteConfigNotFound>), ()>,

    spriteconfig: Query<(&MappedAnimations, Has<Directionable>, Has<MovementBased>, Has<GroundingBased>, ), ()>,

    mut animation_query: Query<(&StrId, &AnimationHandle, &AnimationSheet, &AcZ, Option<&YSortOrigin>, Option<&ClipStartFrames>, Has<SaveAnimationProgress>, Option<&AlternatingStartFramesConfig>, Option<&mut AlternatingStartFramesState>, Option<&PlayingSpeed>, Option<&AnimationSeri>),()>,

    mut atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    images: Res<Assets<Image>>,
) {

    let mut entis_to_iter = EntityHashSet::with_capacity(changers.iter().size_hint().0 + move_anims_changed.len());
    entis_to_iter.extend(changers.iter());
    entis_to_iter.extend(move_anims_changed.read().map(|f| f.0));

    for (held_sprites, direction, moving, grounding) in base.iter_many(entis_to_iter) {
        for held_sprite in held_sprites.entities() {
            let Ok((ent, prev_animation, sprite_cfg_ref, state_id, playing_speed, animation_progresses, has_sprite_config_not_found)) = sprites_query.get_mut(*held_sprite)
            else { error!(target: SPRITE_ANIMATION_SYSTEM, "Failed to get sprite entity {:?}", held_sprite); continue };

            if has_sprite_config_not_found {
                continue;
            }

            let Ok((sprite_cfg_animations_map, directionable, movement_based, grounding_based, )) = spriteconfig.get(sprite_cfg_ref.0)
            else { error!(target: SPRITE_ANIMATION_SYSTEM, "Failed to get SpriteConfigRef entity {:?}", sprite_cfg_ref.0); continue };

            let anim_type = AnimType {
                direction: if directionable { direction.copied().unwrap_or_default() } else { CardinalDirection::default() },
                moving: if movement_based { moving.copied().unwrap_or_default() } else { MoveAnimActive::default() },
                grounding: if grounding_based { *grounding } else { Grounding::default() },
                state_id: state_id.cloned(),
            };
            debug!(target: SPRITE_ANIMATION_SYSTEM, "Determined AnimType {:?} for sprite entity {:?}", anim_type, ent);

            let Some(anim_ent) = sprite_cfg_animations_map.0.get(&anim_type) else {
                warn!(target: SPRITE_ANIMATION_SYSTEM, "No animation found for AnimType {:?} in SpriteCfgAnimationsMap for entity {:?}", anim_type, ent);
                continue;
            };

            let Ok((_, anim_handle, anim_sheet, z, y_sort, clip_start_frames, should_save_anim_progress, alternating_config, mut alternating_state, anim_playing_speed, anim_seri )) = animation_query.get_mut(*anim_ent) else {
                error!(target: SPRITE_ANIMATION_SYSTEM, "Failed to get animation data for animation entity {:?}", anim_ent);
                continue;
            };

            let Some(sprite) = anim_sheet.0.with_loaded_image(&images) else {
                error!(target: SPRITE_ANIMATION_SYSTEM, "Failed to create sprite for animation entity {:?} because image is not loaded yet.", anim_ent);
                continue;
            };
            let sprite = sprite.sprite(&mut atlas_layouts);

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

            let speed_factor = playing_speed
                .map(|speed| speed.0)
                .or_else(|| anim_playing_speed.map(|speed| speed.0))
                .unwrap_or_else(|| PlayingSpeed::default().0);

            let playing = !matches!(anim_seri.and_then(|seri| seri.paused), Some(true));

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


            if let Some(prev_animation) = prev_animation {
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

            if insert_needed {
                cmd.entity(ent).try_insert((sprite, spritesheet_animation, z.clone(), y_sort.cloned().unwrap_or_default()));

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
                mode: SendMode::BroadcastExcept(ClientId::Client(controller.client)),
                message: event_data,
            });
            trace!(target: SPRITE_ANIMATION_SYSTEM, "Sending moving {} for entity {:?} {} to all clients except {:?}", moving, being_ent, id.cloned().unwrap_or_default(), controller.client);
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
