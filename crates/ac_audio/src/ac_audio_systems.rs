use bevy::prelude::*;
use bevy_kira_audio::prelude::*;
use bevy_kira_audio::{DefaultSpatialRadius, SpatialAudioEmitter};
use bevy_spritesheet_animation::prelude::*;
use being::being_components::Being;
use being::race::race_components::{ProducesStepSfx, RaceFootstepSfxConfig};
use being::race::race_resources::RaceRef;
use being_shared::ComputedLocally;
use game_common::game_common_components::EntityZeroRef;
use sprite_shared::prelude::*;
use sprite_animation_shared::MoveAnimActive;
use tilemap::tile::tile_components::{TileStepSfx, TileStepSfxConfig};
use tilemap_shared::{DimensionRef, GlobalTilePos, TileGatheringParamSet};
use std::hash::{DefaultHasher, Hash, Hasher};

use crate::ac_audio_components::*;
use crate::SpatialAudioSettings;

pub fn play_sprite_animation_sfx_on_frame_change(
    mut cmd: Commands,
    mut sprites: Query<(
        Entity,
        &SpritesheetAnimation,
        &EntityZeroRef,
        Option<&mut SpatialAudioEmitter>,
        Option<&mut AnimationFrameSfxState>,
    ), Changed<SpritesheetAnimation>>,
    sprite_configs: Query<&SpriteAnimSfx>,
    asset_server: Res<AssetServer>,
    audio: Res<Audio>,
) {
    for (ent, anim, sprite_cfg_ref, spatial_emitter, state) in &mut sprites {
        let Ok(cfg_sfx) = sprite_configs.get(sprite_cfg_ref.0) else { continue };
        if cfg_sfx.sound_paths.is_empty() {
            continue;
        }
        let Some(mut state) = state else {
            cmd.entity(ent).insert(AnimationFrameSfxState {
                last_frame: anim.progress.frame,
                frame_changes_acc: 0.0,
            });
            continue;
        };
        if anim.progress.frame == state.last_frame {
            continue;
        }
        state.last_frame = anim.progress.frame;
        state.frame_changes_acc += 1.0;
        let interval = cfg_sfx.every_n_frame_changes.max(0.001);
        if state.frame_changes_acc + f32::EPSILON < interval {
            continue;
        }
        state.frame_changes_acc %= interval;
        let mut handles = Vec::new();
        for sfx in cfg_sfx.sound_paths.iter() {
            let path = sfx.trim();
            if path.is_empty() {
                continue;
            }
            let mut play_cmd = audio.play(asset_server.load(path.to_string()));
            play_cmd.with_emitter(ent);
            handles.push(play_cmd.handle());
        }
        if !handles.is_empty() {
            if let Some(mut emitter) = spatial_emitter {
                emitter.instances.extend(handles);
            } else {
                cmd.entity(ent).insert(SpatialAudioEmitter { instances: handles });
            }
        }
    }
}

pub fn play_animation_seri_sfx_on_frame_change(
    mut cmd: Commands,
    mut sprites: Query<(
        Entity,
        &SpritesheetAnimation,
        Option<&mut SpatialAudioEmitter>,
        Option<&AnimationSeriSfxConfig>,
        Option<&mut AnimationSeriSfxState>
    ), Changed<SpritesheetAnimation>>,
    asset_server: Res<AssetServer>,
    audio: Res<Audio>,
) {
    for (ent, anim, spatial_emitter, cfg, state) in &mut sprites {
        let Some(cfg) = cfg else { continue };
        let Some(mut state) = state else {
            cmd.entity(ent).insert(AnimationSeriSfxState {
                last_frame: anim.progress.frame,
                frame_changes_acc: 0.0,
            });
            continue;
        };
        if anim.progress.frame == state.last_frame {
            continue;
        }
        state.last_frame = anim.progress.frame;
        state.frame_changes_acc += 1.0;
        let interval = cfg.every_n_frame_changes.max(0.001);
        if state.frame_changes_acc + f32::EPSILON < interval {
            continue;
        }
        state.frame_changes_acc %= interval;
        let mut handles = Vec::new();
        for sfx in cfg.sound_paths.iter() {
            let path = sfx.trim();
            if path.is_empty() {
                continue;
            }
            let mut play_cmd = audio.play(asset_server.load(path.to_string()));
            play_cmd.with_emitter(ent);
            handles.push(play_cmd.handle());
        }
        if !handles.is_empty() {
            if let Some(mut emitter) = spatial_emitter {
                emitter.instances.extend(handles);
            } else {
                cmd.entity(ent).insert(SpatialAudioEmitter { instances: handles });
            }
        }
    }
}

pub fn sync_sprite_loop_sfx(
    mut cmd: Commands,
    mut sprites: Query<(
        Entity,
        &EntityZeroRef,
        &BaseHolderRef,
        Option<&SpritesheetAnimation>,
        Option<&mut SpatialAudioEmitter>,
        Option<&SpriteLoopSfxState>,
    )>,
    sprite_cfgs: Query<&SpriteLoopSfx>,
    move_anims: Query<&MoveAnimActive>,
    mut audio_instances: ResMut<Assets<AudioInstance>>,
    asset_server: Res<AssetServer>,
    audio: Res<Audio>,
) {
    for (ent, sprite_cfg_ref, base_holder, sprite_anim, spatial_emitter, loop_state) in &mut sprites {
        let Ok(loop_cfg) = sprite_cfgs.get(sprite_cfg_ref.0) else {
            if let Some(loop_state) = loop_state {
                for handle in loop_state.instances.iter() {
                    let Some(instance) = audio_instances.get_mut(handle) else { continue };
                    instance.stop(AudioTween::default());
                }
                cmd.entity(ent).remove::<SpriteLoopSfxState>();
            }
            continue;
        };
        if loop_cfg.sound_paths.is_empty() {
            if let Some(loop_state) = loop_state {
                for handle in loop_state.instances.iter() {
                    let Some(instance) = audio_instances.get_mut(handle) else { continue };
                    instance.stop(AudioTween::default());
                }
                cmd.entity(ent).remove::<SpriteLoopSfxState>();
            }
            continue;
        }
        let should_play = match loop_cfg.condition {
            SfxPlayCondition::WhileAnimationPlaying => sprite_anim.map_or(false, |a| a.playing && a.speed_factor.abs() > 0.0001),
            SfxPlayCondition::WhileMoveActive => move_anims.get(base_holder.base).is_ok_and(|m| m.get()),
        };
        if !should_play {
            if let Some(loop_state) = loop_state {
                for handle in loop_state.instances.iter() {
                    let Some(instance) = audio_instances.get_mut(handle) else { continue };
                    instance.stop(AudioTween::default());
                }
                cmd.entity(ent).remove::<SpriteLoopSfxState>();
            }
            continue;
        }
        let Some(_loop_state) = loop_state else {
            let mut handles = Vec::new();
            for sfx in loop_cfg.sound_paths.iter() {
                let path = sfx.trim();
                if path.is_empty() {
                    continue;
                }
                let mut play_cmd = audio.play(asset_server.load(path.to_string()));
                play_cmd.looped().with_emitter(ent);
                handles.push(play_cmd.handle());
            }
            if !handles.is_empty() {
                if let Some(mut emitter) = spatial_emitter {
                    emitter.instances.extend(handles.clone());
                } else {
                    cmd.entity(ent).insert(SpatialAudioEmitter { instances: handles.clone() });
                }
                cmd.entity(ent).insert(SpriteLoopSfxState { instances: handles });
            }
            continue;
        };
    }
}

pub fn sync_sprite_timed_sfx(
    mut cmd: Commands,
    mut sprites: Query<(
        Entity,
        &EntityZeroRef,
        &BaseHolderRef,
        Option<&SpritesheetAnimation>,
        Option<&mut SpatialAudioEmitter>,
        Option<&SpriteTimedSfxState>,
    )>,
    sprite_cfgs: Query<&SpriteTimedSfx>,
    move_anims: Query<&MoveAnimActive>,
    asset_server: Res<AssetServer>,
    audio: Res<Audio>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();
    if dt <= 0.0 {
        return;
    }
    for (ent, sprite_cfg_ref, base_holder, sprite_anim, spatial_emitter, state) in &mut sprites {
        let Ok(cfg) = sprite_cfgs.get(sprite_cfg_ref.0) else {
            if state.is_some() { cmd.entity(ent).remove::<SpriteTimedSfxState>(); }
            continue;
        };
        if cfg.condition == SfxPlayCondition::WhileMoveActive {
            if state.is_some() { cmd.entity(ent).remove::<SpriteTimedSfxState>(); }
            continue;
        }
        if cfg.sound_paths.is_empty() {
            if state.is_some() { cmd.entity(ent).remove::<SpriteTimedSfxState>(); }
            continue;
        }
        let should_play = match cfg.condition {
            SfxPlayCondition::WhileAnimationPlaying => sprite_anim.map_or(false, |a| a.playing && a.speed_factor.abs() > 0.0001),
            SfxPlayCondition::WhileMoveActive => move_anims.get(base_holder.base).is_ok_and(|m| m.get()),
        };
        if !should_play {
            if state.is_some() { cmd.entity(ent).remove::<SpriteTimedSfxState>(); }
            continue;
        }
        let speed_scale = if cfg.scale_interval_with_animation_speed {
            sprite_anim.map_or(1.0, |a| a.speed_factor.abs().max(0.001))
        } else {
            1.0
        };
        let interval = (cfg.time_interval_secs / speed_scale).max(0.001);
        let mut elapsed = state.map_or(interval, |s| s.elapsed_secs + dt);
        if elapsed < interval {
            cmd.entity(ent).insert(SpriteTimedSfxState { elapsed_secs: elapsed });
            continue;
        }

        let mut handles = Vec::new();
        while elapsed >= interval {
            elapsed -= interval;
            for sfx in cfg.sound_paths.iter() {
                let path = sfx.trim();
                if path.is_empty() {
                    continue;
                }
                let mut play_cmd = audio.play(asset_server.load(path.to_string()));
                play_cmd.with_emitter(ent);
                handles.push(play_cmd.handle());
            }
        }
        if !handles.is_empty() {
            if let Some(mut emitter) = spatial_emitter {
                emitter.instances.extend(handles);
            } else {
                cmd.entity(ent).insert(SpatialAudioEmitter { instances: handles });
            }
        }
        cmd.entity(ent).insert(SpriteTimedSfxState { elapsed_secs: elapsed });
    }
}

pub fn apply_spatial_audio_settings(
    settings: Res<SpatialAudioSettings>,
    mut default_radius: ResMut<DefaultSpatialRadius>,
) {
    default_radius.radius = settings.max_distance_m.max(0.1) * settings.pixels_per_meter.max(0.001);
}

pub fn play_step_sfx_from_moved_distance(
    mut cmd: Commands,
    mut beings: Query<(
        Entity,
        &Transform,
        &DimensionRef,
        Has<ComputedLocally>,
        Option<&HeldSprites>,
        Option<&RaceRef>,
        &MoveAnimActive,
        Option<&mut SpatialAudioEmitter>,
        Option<&mut StepDistanceSfxState>,
    ), With<Being>>,
    race_step_sfx_enabled: Query<&ProducesStepSfx>,
    race_footstep_sfx_cfgs: Query<&RaceFootstepSfxConfig>,
    sprite_entity_zero_refs: Query<&EntityZeroRef>,
    sprite_step_cfgs: Query<&SpriteTimedSfx>,
    tile_entity_zero_refs: Query<&EntityZeroRef>,
    tile_step_sfxs: Query<(&TileStepSfx, Option<&TileStepSfxConfig>)>,
    mut tile_gathering: TileGatheringParamSet,
    settings: Res<SpatialAudioSettings>,
    asset_server: Res<AssetServer>,
    audio: Res<Audio>,
    mut step_paths: Local<Vec<String>>,
) {
    fn hash_path(path: &str) -> u64 {
        let mut hasher = DefaultHasher::new();
        path.hash(&mut hasher);
        hasher.finish()
    }
    let ppm = settings.pixels_per_meter.max(0.001);
    let step_distance_m = settings.footstep_distance_m.max(0.01);
    let teleport_threshold_m = settings.footstep_teleport_threshold_m.max(step_distance_m);
    for (being_ent, transform, &dim_ref, is_locally_controlled, held_sprites, race_ref, move_anim, spatial_emitter, step_state) in &mut beings {
        let current_pos_px = transform.translation.truncate();
        let Some(mut step_state) = step_state else {
            cmd.entity(being_ent).insert(StepDistanceSfxState {
                last_pos_px: current_pos_px,
                accumulated_distance_m: 0.0,
                last_sfx_path_hash: 0,
            });
            continue;
        };

        let moved_px = current_pos_px.distance(step_state.last_pos_px);
        let moved_m = moved_px / ppm;
        step_state.last_pos_px = current_pos_px;
        if moved_m > teleport_threshold_m {
            step_state.accumulated_distance_m = 0.0;
            continue;
        }
        step_state.accumulated_distance_m += moved_m;

        if !move_anim.get() {
            continue;
        }
        if let Some(race_ref) = race_ref {
            if race_step_sfx_enabled.get(race_ref.0).is_err() {
                continue;
            }
        }
        if step_state.accumulated_distance_m + f32::EPSILON < step_distance_m {
            continue;
        }

        step_paths.clear();
        let mut prevent_repeat = true;
        let mut disable_tile_step_sfx = false;
        if let Some(race_ref) = race_ref {
            if let Ok(race_footstep_cfg) = race_footstep_sfx_cfgs.get(race_ref.0) {
                disable_tile_step_sfx = race_footstep_cfg.disable_tile_step_sfx;
                for path in race_footstep_cfg.paths.iter() {
                    if !path.trim().is_empty() {
                        step_paths.push(path.clone());
                    }
                }
            }
        }
        if let Some(held_sprites) = held_sprites {
            for held_sprite in held_sprites.iter() {
                let Ok(sprite_cfg_ref) = sprite_entity_zero_refs.get(held_sprite) else { continue };
                let Ok(step_cfg) = sprite_step_cfgs.get(sprite_cfg_ref.0) else { continue };
                if step_cfg.condition != SfxPlayCondition::WhileMoveActive {
                    continue;
                }
                for path in step_cfg.sound_paths.iter() {
                    if !path.trim().is_empty() {
                        step_paths.push(path.clone());
                    }
                }
                if !step_paths.is_empty() {
                    break;
                }
            }
        }

        if !disable_tile_step_sfx {
            for tile_ent in tile_gathering.gather_tiles_at_to_drain(dim_ref, GlobalTilePos::from(current_pos_px)) {
                let Ok(tile_cfg_ref) = tile_entity_zero_refs.get(*tile_ent) else { continue };
                let Ok((tile_step_sfx, tile_step_sfx_cfg)) = tile_step_sfxs.get(tile_cfg_ref.0) else { continue };
                if tile_step_sfx_cfg.is_some_and(|cfg| !cfg.prevent_repeat) {
                    prevent_repeat = false;
                }
                let Some(weighted_group) = tile_step_sfx.sample_with_rng(&mut rand::rng()) else { continue };
                for path in weighted_group.iter() {
                    let trimmed = path.trim();
                    if !trimmed.is_empty() {
                        step_paths.push(trimmed.to_string());
                    }
                }
            }
        }
        if step_paths.is_empty() {
            step_state.accumulated_distance_m %= step_distance_m;
            continue;
        }

        let mut spatial_handles = Vec::new();
        while step_state.accumulated_distance_m + f32::EPSILON >= step_distance_m {
            step_state.accumulated_distance_m -= step_distance_m;
            let chosen_i = if prevent_repeat {
                let mut candidate_indices = Vec::with_capacity(step_paths.len());
                for (i, path) in step_paths.iter().enumerate() {
                    if hash_path(path) != step_state.last_sfx_path_hash {
                        candidate_indices.push(i);
                    }
                }
                if candidate_indices.is_empty() {
                    rand::random_range(0..step_paths.len())
                } else {
                    candidate_indices[rand::random_range(0..candidate_indices.len())]
                }
            } else {
                rand::random_range(0..step_paths.len())
            };
            let chosen_path = &step_paths[chosen_i];
            let mut play_cmd = audio.play(asset_server.load(chosen_path.clone()));
            if is_locally_controlled {
                let _ = play_cmd.handle();
            } else {
                play_cmd.with_emitter(being_ent);
                spatial_handles.push(play_cmd.handle());
            }
            step_state.last_sfx_path_hash = hash_path(chosen_path);
        }
        if is_locally_controlled || spatial_handles.is_empty() {
            continue;
        }
        if let Some(mut emitter) = spatial_emitter {
            emitter.instances.extend(spatial_handles);
        } else {
            cmd.entity(being_ent).insert(SpatialAudioEmitter { instances: spatial_handles });
        }
    }
}
