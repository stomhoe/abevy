

use bevy_replicon::prelude::*;
#[allow(unused_imports)] use bevy::prelude::*;
use bevy_spritesheet_animation::prelude::*;
use common::{common_components::{AssetScoped, EntityPrefix, ImageHolder, ImagePathHolder, StrId}, common_states::GameSetupType};
use game_common::game_common_components::{Directionable, FacingDirection, MyZ, YSortOrigin};
use sprite::{sprite_components::*, sprite_scale_offset_components::{Offset2D, Scale2D}};
use ::sprite_animation_shared::*;

use crate::{sprite_animation_components::*, sprite_animation_events::MoveStateUpdated, sprite_animation_resources::*};



#[allow(unused_parens)]
pub fn init_animations(
    mut cmd: Commands,
    mut anim_handles: ResMut<AnimSerisHandles>,
    mut seris_assets: ResMut<Assets<AnimationSerialization>>,
    mut library: ResMut<AnimationLibrary>,
    //usar state
) {
    use std::mem::take;

    if !library.0.is_empty() {
        return;
    }

    let holder = cmd.spawn((AnimationsHolder, )).id();


    for handle in take(&mut anim_handles.handles) {
        let Some(mut seri) = seris_assets.remove(&handle) else { continue };

        let Ok(_) = ImagePathHolder::new(seri.img_path.clone()) else {
            let err = BevyError::from(format!("Failed to find image for Animation {}: {}", seri.id, "invalid image path"));
            error!(target: "sprite_animation_init", "{}", err);
            continue;
        };

        let str_id = StrId::new_truncated(take(&mut seri.id));
        if !library.0.contains_key(&str_id) {

            let y_sort = seri.y_sort.clone();

            let ent = cmd.spawn((AnimationMain, str_id.clone(), seri, ChildOf(holder))).id();

            if let Some(y_sort) = y_sort {
                cmd.entity(ent).insert(YSortOrigin(y_sort));
            }

            debug!(target: "sprite_animation_init", "Inserting animation '{}' into library.", str_id);
            library.0.insert(str_id, ent);//NO SÉ SI MOVER A OTRO LUGAR
        } else {
            error!(target: "sprite_animation_init", "Animation with id '{}' already present in library, skipping insert.", str_id);
            continue;
        }
    }
}

// ----------------------> NO OLVIDARSE DE AGREGARLO AL Plugin DEL MÓDULO <-----------------------------
//                                                       ^^^^
#[allow(unused_parens)]
pub fn init_animation_sheet_and_handle(mut cmd: Commands, 
    asset_server: Res<AssetServer>,
    mut animation_assets: ResMut<Assets<Animation>>,
    query: Query<(Entity, &StrId, &AnimationSerialization),(With<AnimationSerialization>, Without<AnimationHandle>)>,
) {
    //trace!(target: "sprite_animation_init", "Initializing animation sheets and handles...");
    for (entity, str_id, seri) in query.iter() {
        debug!(
            target: "sprite_animation_init",
            "Initializing sheet and handle for {:?}, {:?}, {}",
            entity, str_id, seri.img_path
        );
        let image_handle = asset_server.load(&seri.img_path);

        let (rows, cols) = seri.rows_cols.unwrap_or((1, 1));
        if rows < 1 || cols < 1 {
            if rows < 1 && cols < 1 {
                error!(
                    target: "sprite_animation_init",
                    "Invalid rows ({}) and cols ({}) for animation '{}', setting both to minimum 1.",
                    rows, cols, str_id
                );
            } else if rows < 1 {
                error!(
                    target: "sprite_animation_init",
                    "Invalid rows ({}) for animation '{}', setting to minimum 1.",
                    rows, str_id
                );
            } else {
                error!(
                    target: "sprite_animation_init",
                    "Invalid cols ({}) for animation '{}', setting to minimum 1.",
                    cols, str_id
                );
            }
        }    let (rows, cols) = (rows.max(1), cols.max(1));

        let sheet = Spritesheet::new(&image_handle, cols, rows);
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

        if clips_len == 0 {
            animation = animation.add_row(0);
        }
        else {
            for (i, cfg) in seri.clips.clone().into_iter().enumerate() {
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
        }
        let handle: Handle<Animation> = animation_assets.add(animation.build());
        cmd.entity(entity).insert((AnimationHandle(handle), AnimationSheet(sheet)));

        if let Some(offset) = seri.offset {
            cmd.entity(entity).insert(Offset2D::from(offset));
        }
        if let Some(scale) = seri.scale {
            cmd.entity(entity).insert(Scale2D::from(scale));
        }

        if let Some(color) = seri.color {
            let (red, green, blue, alpha) = color.into();
            cmd.entity(entity).insert(ColorHolder(Color::srgba_u8(red, green, blue, alpha)));
        }

        cmd.entity(entity).insert(MyZ(seri.z));
    }
}
