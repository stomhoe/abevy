

use bevy_replicon::prelude::*;
use being_shared::{Grounding, ControlledBy};
use bevy::ecs::entity_disabling::Disabled;
#[allow(unused_imports)] use bevy::prelude::*;
use bevy_replicon_renet::renet::RenetClient;
use bevy_spritesheet_animation::prelude::*;
use common::{common_components::{ImageHolder, StrId}, common_states::GameSetupType};
use game_common::game_common_components::{Directionable, FacingDirection};
use player::player_components::*;
use sprite::sprite_components::*;
use sprite_animation_shared::AnimationLibrary;

use crate::{sprite_animation_components::*, sprite_animation_events::MoveStateUpdated, sprite_animation_resources::*};


#[allow(unused_parens)]
pub fn init_animations(
    mut cmd: Commands,
    mut anim_handles: ResMut<AnimSerisHandles>,
    mut seris_assets: ResMut<Assets<AnimationSeri>>,
    mut animation_assets: ResMut<Assets<Animation>>,
    mut library: ResMut<AnimationLibrary>,
    asset_server: Res<AssetServer>, 
    //usar state
) {
    use std::mem::take;


    for handle in take(&mut anim_handles.handles) {
        let Some(seri) = seris_assets.remove(&handle) else { continue };

        let img_holder = match ImageHolder::new(&asset_server, seri.img_path) {
            Ok(holder) => holder,
            Err(e) => {
                let err = BevyError::from(format!("Failed to load image for Animation {}: {}", seri.id, e));
                error!(target: "sprite_loading", "{}", err);
                continue;
            }
        };
        let sheet = Spritesheet::new(img_holder.handle(), seri.rows_cols.1, seri.rows_cols.0,);

        let mut animation = sheet
            .create_animation()
            .set_repetitions(
                match seri.reps {
                    None => AnimationRepeat::Loop,
                    Some(n) => AnimationRepeat::Times(n),
                }
            )
            .set_direction(
                match seri.dir {
                    None => AnimationDirection::Forwards,
                    Some(true) => AnimationDirection::Backwards,
                    Some(false) => AnimationDirection::PingPong,
                }
            )
            .set_duration(
                match (seri.dur_frame, seri.dur_rep) {
                    (None, None) => AnimationDuration::default(),
                    (None, Some(rep_dur)) => AnimationDuration::PerRepetition(rep_dur),
                    (Some(frame_dur), None) => AnimationDuration::PerFrame(frame_dur),
                    (Some(frame_dur), Some(_rep_dur)) => AnimationDuration::PerFrame(frame_dur),
                }
            );

        let clips_len = seri.clips.len();
        for (i, cfg) in seri.clips.into_iter().enumerate() {
            animation = if cfg.is_row {
                match cfg.partial {
                    Some((start, end)) => animation.add_partial_row(cfg.target, start..end),
                    None => animation.add_row(cfg.target),
                }
            } else {
                match cfg.partial {
                    Some((start, end)) => animation.add_partial_column(cfg.target, start..end),
                    None => animation.add_column(cfg.target),
                }
            };
            animation = match cfg.dir {
                None => animation.set_direction(AnimationDirection::Forwards),
                Some(true) => animation.set_direction(AnimationDirection::Backwards),
                Some(false) => animation.set_direction(AnimationDirection::PingPong),
            };
            animation = match cfg.reps {
                None => animation.set_repetitions(AnimationRepeat::Loop),
                Some(n) => animation.set_repetitions(AnimationRepeat::Times(n)),
            };
            animation = match (cfg.dur_frame, cfg.dur_rep) {
                (None, None) => animation,
                (None, Some(rep_dur)) => animation.set_duration(AnimationDuration::PerRepetition(rep_dur)),
                (Some(frame_dur), None) => animation.set_duration(AnimationDuration::PerFrame(frame_dur)),
                (Some(frame_dur), Some(_rep_dur)) => animation.set_duration(AnimationDuration::PerFrame(frame_dur)),
            };
            if i < clips_len - 1 {
                animation = animation.start_clip();
            }
        }
        let handle: Handle<Animation> = animation_assets.add(animation.build());

        if !library.0.contains_key(&StrId::new_truncated(&seri.id)) {
            debug!(target: "sprite_animation", "Inserting animation '{}' into library.", seri.id);
            library.0.insert(StrId::new_truncated(&seri.id), (sheet, handle.clone()));
        } else {
            error!(target: "sprite_animation", "Animation with name '{}' already present in library, skipping insert.", seri.id);
        }
    }
}