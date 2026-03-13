

#[allow(unused_imports)] use bevy::prelude::*;
use bevy_spritesheet_animation::prelude::*;
use common::common_components::*;
use common::def_db::discover_assets_files_by_suffixes;
use common::log_targets::SPRITE_ANIMATION_INIT;
use ::sprite_shared::prelude::*;
use sprite_systems::prelude::*;
use ::sprite_animation_shared::*;

fn load_animation_defs_from_filesystem() -> Vec<AnimationSeri> {
    let mut out = Vec::new();
    let Ok(files) = discover_assets_files_by_suffixes(&["anim.ron"]) else {
        return out;
    };
    let mut failed = 0usize;
    for (_source, path) in files {
        let Ok(content) = std::fs::read_to_string(&path) else {
            continue;
        };
        if let Ok(multi) = ron::from_str::<MultipleAnimationSeri>(&content) {
            out.extend(multi.0);
            continue;
        }
        if let Ok(one) = ron::from_str::<AnimationSeri>(&content) {
            out.push(one);
            continue;
        }
        if let Ok(many) = ron::from_str::<Vec<AnimationSeri>>(&content) {
            out.extend(many);
            continue;
        }
        failed += 1;
        warn!(target: SPRITE_ANIMATION_INIT, "Failed to parse animation defs in '{}'", path.to_string_lossy());
    }
    if out.is_empty() {
        error!(target: SPRITE_ANIMATION_INIT, "No animation defs loaded from filesystem ({} file parse failures)", failed);
    } else if failed > 0 {
        warn!(target: SPRITE_ANIMATION_INIT, "Loaded {} animation defs with {} file parse failures", out.len(), failed);
    }
    out
}

#[allow(unused_parens)]
pub fn init_animations(
    mut cmd: Commands,
    library: Res<AcAnimationEntityMap>,
    sc_holder: Query<Entity, With<EguiScsHolder>>,
    anim_holder: Query<Entity, With<EguiAcAnimationsHolder>>,
    //usar state
) {
    if !library.0.is_empty() {
        return;
    }
    let sc_holder = if sc_holder.is_empty() {
        debug!(target: SPRITE_ANIMATION_INIT, "Creating AnimationsHolder as SpriteConfigsHolder not found.");
        cmd.spawn((EguiScsHolder, )).id()
    }
    else {
        sc_holder.single().unwrap()
    };

    let anim_holder = if anim_holder.is_empty() {
        cmd.spawn((EguiAcAnimationsHolder, )).id()
    }
    else {
        anim_holder.single().unwrap()
    };
    cmd.entity(anim_holder).try_insert_if_new(ChildOf(sc_holder));

    let mut main_comps = Vec::new();

    let mut merged_seris_vec: Vec<(Entity, AnimationSeri)> =
        load_animation_defs_from_filesystem()
            .into_iter()
            .map(|seri| (Entity::PLACEHOLDER, seri))
            .collect();

    let mut i = 0;
    while i < merged_seris_vec.len() {
        let (ent, seri) = &mut merged_seris_vec[i];

        let Ok(_) = ImagePathHolder::validate_path_exists(seri.img_path.clone()) else {
            let err = BevyError::from(format!("Failed to find image for Animation {}: {}", seri.id, "invalid image path"));
            error!(target: SPRITE_ANIMATION_INIT, "{}", err);
            merged_seris_vec.remove(i);
            continue;
        };
        let str_id = StrId::trunc(std::mem::take(&mut seri.id));

        let y_sort = seri.y_sort.clone();

        *ent = cmd.spawn_empty().id();
        let ent = *ent;

        main_comps.push((ent, (AcAnimation, str_id.clone(), ChildOf(anim_holder), AcZ(seri.z))));

        if let Some(y_sort) = y_sort {
            cmd.entity(ent).insert(YSortOrigin(y_sort));
        }
        if seri.offset != [0.0, 0.0] {
            let offset = seri.offset;
            cmd.entity(ent).insert(Offset2D::from(offset));
        }
        if seri.scale != [1.0, 1.0] {
            let scale = seri.scale;
            cmd.entity(ent).insert(Scale2D::from(scale));
        }
        let default_speed = PlayingSpeed::default().0;
        if (seri.speed - default_speed).abs() > f32::EPSILON {
            cmd.entity(ent).insert(PlayingSpeed(seri.speed));
        }
        if let Some(color) = seri.color {
            let (red, green, blue, alpha) = color.into();
            cmd.entity(ent).insert(ColorHolder(Color::srgba_u8(red, green, blue, alpha)));
        }
        if seri.save_animation_progress {
            cmd.entity(ent).insert(SaveAnimationProgress);
        }
        i += 1;
    }
    cmd.try_insert_batch(merged_seris_vec);
    cmd.try_insert_batch(main_comps);
}
#[allow(unused_parens)]
pub fn init_animation_sheet_and_handle(mut cmd: Commands,
    asset_server: Res<AssetServer>,
    mut animation_assets: ResMut<Assets<Animation>>,
    query: Query<(Entity, &StrId, &AnimationSeri),(With<AnimationSeri>, Without<AnimationHandle>)>,
) {
    //trace!(target: SPRITE_ANIMATION_INIT, "Initializing animation sheets and handles...");
    for (entity, str_id, seri) in query.iter() {
        debug!(
            target: SPRITE_ANIMATION_INIT,
            "Initializing sheet and handle for {:?}, {:?}, {}",
            entity, str_id, seri.img_path
        );
        let image_handle = asset_server.load(&seri.img_path);

        let (rows, cols) = seri.rows_cols;
        if rows < 1 || cols < 1 {
            if rows < 1 && cols < 1 {
                error!(
                    target: SPRITE_ANIMATION_INIT,
                    "Invalid rows ({}) and cols ({}) for animation '{}', setting both to minimum 1.",
                    rows, cols, str_id
                );
            } else if rows < 1 {
                error!(
                    target: SPRITE_ANIMATION_INIT,
                    "Invalid rows ({}) for animation '{}', setting to minimum 1.",
                    rows, str_id
                );
            } else {
                error!(
                    target: SPRITE_ANIMATION_INIT,
                    "Invalid cols ({}) for animation '{}', setting to minimum 1.",
                    cols, str_id
                );
            }
        }
        let (rows, cols) = (rows.max(1), cols.max(1));

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
        let mut clip_start_frames = Vec::new();
        let mut alternating_start_frames_config = Vec::new();
        let mut alternating_start_frames_state = Vec::new();
        let mut valid_clip_count = 0usize;

        if clips_len == 0 {
            animation = animation.add_row(0);
            clip_start_frames.push(0);
            alternating_start_frames_config.push(seri.alternating_start_frames);
            alternating_start_frames_state.push(0);
            valid_clip_count = 1;
        } else {
            for (i, cfg) in seri.clips.iter().enumerate() {
                let Some(frame_count) = validate_clip_bounds(cfg, rows, cols, str_id) else {
                    continue;
                };
                valid_clip_count += 1;
                clip_start_frames.push(cfg.start_frame.unwrap_or(0));
                alternating_start_frames_config.push(seri.alternating_start_frames);
                alternating_start_frames_state.push(0);
                animation = if cfg.is_row {
                    match cfg.partial {
                        Some((start, end)) => animation.add_partial_row(cfg.target, start..=end),
                        None => animation.add_row(cfg.target),
                    }
                } else {
                    match cfg.partial {
                        Some((start, end)) => animation.add_partial_column(cfg.target, start..=end),
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
                if i < clips_len - 1 && frame_count > 0 {
                    animation = animation.start_clip();
                }
            }
        }
        if valid_clip_count == 0 {
            error!(
                target: SPRITE_ANIMATION_INIT,
                "Animation '{}' had no valid clips after bounds validation, falling back to row 0",
                str_id
            );
            animation = animation.add_row(0);
            clip_start_frames.push(0);
            alternating_start_frames_config.push(seri.alternating_start_frames);
            alternating_start_frames_state.push(0);
        }
        let handle: Handle<Animation> = animation_assets.add(animation.build());
        cmd.entity(entity).insert((AnimationHandle(handle), AnimationSheet(sheet), ClipStartFrames(clip_start_frames), AlternatingStartFramesConfig(alternating_start_frames_config), AlternatingStartFramesState(alternating_start_frames_state)));


    }
}

fn validate_clip_bounds(
    cfg: &ClipConfig,
    rows: usize,
    cols: usize,
    str_id: &StrId,
) -> Option<usize> {
    let axis_len = if cfg.is_row { rows } else { cols };
    let frame_len = if cfg.is_row { cols } else { rows };
    if cfg.target >= axis_len {
        error!(
            target: SPRITE_ANIMATION_INIT,
            "Animation '{}' clip target {} is out of bounds for {} count {}",
            str_id,
            cfg.target,
            if cfg.is_row { "row" } else { "col" },
            axis_len
        );
        return None;
    }
    let frame_count = match cfg.partial {
        Some((start, end)) => {
            if start > end {
                error!(
                    target: SPRITE_ANIMATION_INIT,
                    "Animation '{}' clip partial range {}..={} is invalid because start > end",
                    str_id,
                    start,
                    end
                );
                return None;
            }
            if end >= frame_len {
                error!(
                    target: SPRITE_ANIMATION_INIT,
                    "Animation '{}' clip partial range {}..={} exceeds {} frame bound {}",
                    str_id,
                    start,
                    end,
                    if cfg.is_row { "column" } else { "row" },
                    frame_len
                );
                return None;
            }
            end - start + 1
        }
        None => frame_len,
    };
    if let Some(start_frame) = cfg.start_frame
        && start_frame >= frame_count
    {
        error!(
            target: SPRITE_ANIMATION_INIT,
            "Animation '{}' clip start_frame {} is out of bounds for clip frame count {}",
            str_id,
            start_frame,
            frame_count
        );
        return None;
    }
    Some(frame_count)
}
