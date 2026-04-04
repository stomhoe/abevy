use bevy::platform::collections::{HashMap, HashSet};
use bevy::prelude::*;
use common::def_db::discover_assets_files_by_suffixes;
use common::log_targets::SPRITE_ANIMATION_INIT;
use ::sprite_animation_shared::*;
use serde::de::DeserializeOwned;

#[derive(Clone)]
pub struct LoadedAnimationDef {
    pub rel_path: String,
    pub is_abstract: bool,
    pub seri: AnimationSeri,
}

#[derive(Clone)]
struct RawAnimationDef {
    id: String,
    rel_path: String,
    base_id: Option<String>,
    is_abstract: bool,
    fields: RawAnimationFields,
}

#[derive(Clone, Default)]
struct RawAnimationFields {
    img_path: Option<String>,
    clips: Option<Vec<ClipConfig>>,
    anim_format_id: Option<String>,
    rows_cols: Option<(usize, usize)>,
    save_animation_progress: Option<bool>,
    alternating_start_frames: Option<(usize, usize)>,
    dir: Option<u8>,
    reps: Option<u32>,
    dur_frame: Option<u32>,
    dur_rep: Option<u32>,
    offset: Option<[f32; 2]>,
    scale: Option<[f32; 2]>,
    y_sort: Option<f32>,
    z: Option<f32>,
    color: Option<[u8; 4]>,
    paused: Option<bool>,
    flip_x: Option<bool>,
    flip_y: Option<bool>,
    cardinal_rotation: Option<CardinalRotation>,
    speed: Option<f32>,
    sound_effects: Option<Vec<String>>,
    sound_effects_every_n_frames: Option<f32>,
}

pub fn load_animation_defs_from_filesystem() -> Vec<LoadedAnimationDef> {
    let Ok(mut files) = discover_assets_files_by_suffixes(&[".anim"]) else {
        error!(target: SPRITE_ANIMATION_INIT, "Failed discovering .anim files");
        return Vec::new();
    };
    files.sort_by(|(a, _), (b, _)| {
        a.precedence_rank()
            .cmp(&b.precedence_rank())
            .then_with(|| a.rel_path.cmp(&b.rel_path))
    });

    let mut raw_by_id = HashMap::<String, RawAnimationDef>::default();
    for (source, path) in files {
        let Ok(content) = std::fs::read_to_string(&path) else {
            warn!(target: SPRITE_ANIMATION_INIT, "Failed reading animation file '{}'", path.to_string_lossy());
            continue;
        };
        let Ok(parsed_defs) = parse_animation_file(&content, &source.rel_path) else {
            warn!(target: SPRITE_ANIMATION_INIT, "Failed parsing animation defs in '{}'", source.rel_path);
            continue;
        };

        for parsed in parsed_defs {
            if let Some(prev) = raw_by_id.insert(parsed.id.clone(), parsed) {
                debug!(target: SPRITE_ANIMATION_INIT, "Animation '{}' overridden: '{}' -> '{}'", prev.id, prev.rel_path, source.rel_path);
            }
        }
    }

    let mut resolved = HashMap::<String, AnimationSeri>::default();
    let mut resolving = HashSet::<String>::default();
    let mut out = Vec::with_capacity(raw_by_id.len());

    let mut ids = Vec::with_capacity(raw_by_id.len());
    ids.extend(raw_by_id.keys().cloned());
    ids.sort();

    for id in ids {
        let Some(raw) = raw_by_id.get(&id) else {
            continue;
        };
        let Ok(seri) = resolve_animation_seri(&id, &raw_by_id, &mut resolved, &mut resolving) else {
            error!(target: SPRITE_ANIMATION_INIT, "Failed resolving animation '{}'", id);
            continue;
        };
        out.push(LoadedAnimationDef {
            rel_path: raw.rel_path.clone(),
            is_abstract: raw.is_abstract,
            seri,
        });
    }

    if out.is_empty() {
        error!(target: SPRITE_ANIMATION_INIT, "No animation defs loaded from filesystem");
    } else {
        debug!(target: SPRITE_ANIMATION_INIT, "Loaded {} animation defs from filesystem", out.len());
    }

    out
}

fn resolve_animation_seri(
    id: &str,
    raw_by_id: &HashMap<String, RawAnimationDef>,
    resolved: &mut HashMap<String, AnimationSeri>,
    resolving: &mut HashSet<String>,
) -> Result<AnimationSeri, String> {
    if let Some(seri) = resolved.get(id) {
        return Ok(seri.clone());
    }
    if !resolving.insert(id.to_string()) {
        return Err(format!("Animation '{}' has a cyclic inheritance chain", id));
    }

    let Some(raw) = raw_by_id.get(id) else {
        resolving.remove(id);
        return Err(format!("Animation '{}' not found while resolving inheritance", id));
    };

    let mut seri = if let Some(base_id) = &raw.base_id {
        resolve_animation_seri(base_id, raw_by_id, resolved, resolving)?
    } else {
        AnimationSeri::default()
    };
    seri.id = raw.id.clone();
    apply_raw_animation_fields(&mut seri, &raw.fields);

    resolving.remove(id);
    resolved.insert(id.to_string(), seri.clone());
    Ok(seri)
}

fn apply_raw_animation_fields(seri: &mut AnimationSeri, fields: &RawAnimationFields) {
    if let Some(img_path) = &fields.img_path {
        seri.img_path = img_path.clone();
    }
    if let Some(clips) = &fields.clips {
        seri.clips = clips.clone();
    }
    if let Some(anim_format_id) = &fields.anim_format_id {
        seri.anim_format_id = anim_format_id.clone();
    }
    if let Some(rows_cols) = fields.rows_cols {
        seri.rows_cols = rows_cols;
    }
    if let Some(save_animation_progress) = fields.save_animation_progress {
        seri.save_animation_progress = save_animation_progress;
    }
    if let Some(alternating_start_frames) = fields.alternating_start_frames {
        seri.alternating_start_frames = alternating_start_frames;
    }
    if let Some(dir) = fields.dir {
        seri.dir = dir;
    }
    if let Some(reps) = fields.reps {
        seri.reps = reps;
    }
    if let Some(dur_frame) = fields.dur_frame {
        seri.dur_frame = dur_frame;
    }
    if let Some(dur_rep) = fields.dur_rep {
        seri.dur_rep = dur_rep;
    }
    if let Some(offset) = fields.offset {
        seri.offset = offset;
    }
    if let Some(scale) = fields.scale {
        seri.scale = scale;
    }
    if let Some(y_sort) = fields.y_sort {
        seri.y_sort = y_sort;
    }
    if let Some(z) = fields.z {
        seri.z = z;
    }
    if let Some(color) = fields.color {
        seri.color = Some(color);
    }
    if let Some(paused) = fields.paused {
        seri.paused = paused;
    }
    if let Some(flip_x) = fields.flip_x {
        seri.flip_x = flip_x;
    }
    if let Some(flip_y) = fields.flip_y {
        seri.flip_y = flip_y;
    }
    if let Some(cardinal_rotation) = fields.cardinal_rotation {
        seri.cardinal_rotation = cardinal_rotation;
    }
    if let Some(speed) = fields.speed {
        seri.speed = speed;
    }
    if let Some(sound_effects) = &fields.sound_effects {
        seri.sound_effects = sound_effects.clone();
    }
    if let Some(sound_effects_every_n_frames) = fields.sound_effects_every_n_frames {
        seri.sound_effects_every_n_frames = sound_effects_every_n_frames;
    }
}

fn parse_animation_file(content: &str, rel_path: &str) -> Result<Vec<RawAnimationDef>, String> {
    let mut out = Vec::new();
    let mut current: Option<RawAnimationDef> = None;
    let mut in_clips = false;

    for (idx, raw_line) in content.lines().enumerate() {
        let line_no = idx + 1;
        let line_without_comment = strip_inline_comment(raw_line);
        let trimmed = line_without_comment.trim();
        if trimmed.is_empty() {
            continue;
        }

        if trimmed == "}" {
            if in_clips {
                in_clips = false;
                continue;
            }
            let Some(def) = current.take() else {
                return Err(format!("{rel_path}:{line_no} has an unmatched closing brace"));
            };
            out.push(def);
            continue;
        }

        if current.is_none() {
            let def = parse_animation_header(trimmed, rel_path, line_no)?;
            current = Some(def);
            continue;
        }

        let Some(def) = current.as_mut() else {
            return Err(format!("{rel_path}:{line_no} parser lost current animation state"));
        };

        if in_clips {
            if trimmed.eq_ignore_ascii_case("clips {") {
                return Err(format!("{rel_path}:{line_no} nested clips blocks are not supported"));
            }
            let clip = parse_clip_line(trimmed, rel_path, line_no)?;
            if let Some(clips) = &mut def.fields.clips {
                clips.push(clip);
            } else {
                def.fields.clips = Some(vec![clip]);
            }
            continue;
        }

        if trimmed.eq_ignore_ascii_case("clips {") {
            in_clips = true;
            if def.fields.clips.is_none() {
                def.fields.clips = Some(Vec::new());
            }
            continue;
        }

        let (key, raw_value) = split_field_line(trimmed, rel_path, line_no)?;
        parse_field_assignment(def, &key, &raw_value, rel_path, line_no)?;
    }

    if in_clips {
        return Err(format!("{rel_path} ended while still inside a clips block"));
    }
    if let Some(def) = current.take() {
        return Err(format!("{rel_path} ended before closing animation '{}'", def.id));
    }

    if out.is_empty() {
        return Err(format!("{rel_path} did not contain any animation blocks"));
    }

    Ok(out)
}

fn parse_animation_header(trimmed: &str, rel_path: &str, line_no: usize) -> Result<RawAnimationDef, String> {
    let Some(header) = trimmed.strip_suffix('{').map(str::trim) else {
        return Err(format!("{rel_path}:{line_no} animation header must end with '{{'"));
    };

    let mut tokens = header.split_whitespace();
    let mut is_abstract = false;
    let mut saw_anim = false;
    let mut id = None::<String>;
    let mut base_id = None::<String>;

    while let Some(token) = tokens.next() {
        match token {
            "abstract" => {
                is_abstract = true;
            }
            "anim" => {
                saw_anim = true;
                let Some(raw_id) = tokens.next() else {
                    return Err(format!("{rel_path}:{line_no} missing animation id"));
                };
                id = Some(parse_text_value(raw_id));
            }
            "extends" => {
                let Some(raw_base) = tokens.next() else {
                    return Err(format!("{rel_path}:{line_no} missing extends id"));
                };
                base_id = Some(parse_text_value(raw_base));
            }
            other if !saw_anim => {
                return Err(format!("{rel_path}:{line_no} expected 'anim' but found '{other}'"));
            }
            other => {
                return Err(format!("{rel_path}:{line_no} unexpected token '{other}' in animation header"));
            }
        }
    }

    let Some(id) = id else {
        return Err(format!("{rel_path}:{line_no} missing animation id"));
    };
    if id.trim().is_empty() {
        return Err(format!("{rel_path}:{line_no} animation id is empty"));
    }

    Ok(RawAnimationDef {
        id,
        rel_path: rel_path.to_string(),
        base_id,
        is_abstract,
        fields: RawAnimationFields::default(),
    })
}

fn parse_field_assignment(
    def: &mut RawAnimationDef,
    key: &str,
    raw_value: &str,
    rel_path: &str,
    line_no: usize,
) -> Result<(), String> {
    let key = key.trim().to_ascii_lowercase();
    match key.as_str() {
        "id" => {
            def.id = parse_text_value(raw_value);
        }
        "extends" | "base" => {
            let parsed = parse_text_value(raw_value);
            if parsed.is_empty() {
                return Err(format!("{rel_path}:{line_no} has an empty base id"));
            }
            def.base_id = Some(parsed);
        }
        "abstract" => {
            def.is_abstract = parse_bool_value(raw_value, rel_path, line_no)?;
        }
        "img_path" => {
            def.fields.img_path = Some(parse_text_value(raw_value));
        }
        "anim_format_id" => {
            def.fields.anim_format_id = Some(parse_text_value(raw_value));
        }
        "rows_cols" => {
            def.fields.rows_cols = Some(parse_usize_pair(raw_value, rel_path, line_no)?);
        }
        "save_animation_progress" => {
            def.fields.save_animation_progress = Some(parse_bool_value(raw_value, rel_path, line_no)?);
        }
        "alternating_start_frames" => {
            def.fields.alternating_start_frames = Some(parse_usize_pair(raw_value, rel_path, line_no)?);
        }
        "dir" => {
            def.fields.dir = Some(parse_animation_dir(raw_value, rel_path, line_no)?);
        }
        "reps" => {
            def.fields.reps = Some(parse_usize_value(raw_value, rel_path, line_no)? as u32);
        }
        "dur_frame" => {
            def.fields.dur_frame = Some(parse_usize_value(raw_value, rel_path, line_no)? as u32);
        }
        "dur_rep" => {
            def.fields.dur_rep = Some(parse_usize_value(raw_value, rel_path, line_no)? as u32);
        }
        "offset" => {
            def.fields.offset = Some(parse_f32_pair(raw_value, rel_path, line_no)?);
        }
        "scale" => {
            def.fields.scale = Some(parse_f32_pair(raw_value, rel_path, line_no)?);
        }
        "y_sort" => {
            def.fields.y_sort = Some(parse_f32_value(raw_value, rel_path, line_no)?);
        }
        "z" => {
            def.fields.z = Some(parse_f32_value(raw_value, rel_path, line_no)?);
        }
        "color" => {
            def.fields.color = Some(parse_u8_quad(raw_value, rel_path, line_no)?);
        }
        "paused" => {
            def.fields.paused = Some(parse_bool_value(raw_value, rel_path, line_no)?);
        }
        "flip_x" => {
            def.fields.flip_x = Some(parse_bool_value(raw_value, rel_path, line_no)?);
        }
        "flip_y" => {
            def.fields.flip_y = Some(parse_bool_value(raw_value, rel_path, line_no)?);
        }
        "cardinal_rotation" => {
            def.fields.cardinal_rotation = Some(parse_cardinal_rotation(raw_value, rel_path, line_no)?);
        }
        "speed" => {
            def.fields.speed = Some(parse_f32_value(raw_value, rel_path, line_no)?);
        }
        "sound_effects" => {
            def.fields.sound_effects = Some(parse_string_vec(raw_value, rel_path, line_no)?);
        }
        "sound_effects_every_n_frames" => {
            def.fields.sound_effects_every_n_frames = Some(parse_f32_value(raw_value, rel_path, line_no)?);
        }
        other => {
            return Err(format!("{rel_path}:{line_no} has unknown animation field '{other}'"));
        }
    }
    Ok(())
}

fn parse_clip_line(line: &str, rel_path: &str, line_no: usize) -> Result<ClipConfig, String> {
    let tokens = line.split_whitespace().collect::<Vec<_>>();
    let Some(first) = tokens.first() else {
        return Err(format!("{rel_path}:{line_no} has an empty clip line"));
    };

    let mut clip = ClipConfig::default();
    let mut idx = 0usize;

    if first.eq_ignore_ascii_case("clip") {
        idx += 1;
    }

    if idx >= tokens.len() {
        return Err(format!("{rel_path}:{line_no} clip line is missing a target"));
    }

    match tokens[idx].to_ascii_lowercase().as_str() {
        "row" => {
            clip.is_row = true;
            idx += 1;
            let Some(target) = tokens.get(idx) else {
                return Err(format!("{rel_path}:{line_no} row clip is missing a target index"));
            };
            clip.target = parse_usize_token(target, rel_path, line_no)?;
            idx += 1;
        }
        "col" | "column" => {
            clip.is_row = false;
            idx += 1;
            let Some(target) = tokens.get(idx) else {
                return Err(format!("{rel_path}:{line_no} column clip is missing a target index"));
            };
            clip.target = parse_usize_token(target, rel_path, line_no)?;
            idx += 1;
        }
        "target" => {
            idx += 1;
            let Some(target) = tokens.get(idx) else {
                return Err(format!("{rel_path}:{line_no} clip line is missing a target index"));
            };
            clip.target = parse_usize_token(target, rel_path, line_no)?;
            idx += 1;
        }
        other => {
            return Err(format!("{rel_path}:{line_no} clip line must start with row/col/target, found '{other}'"));
        }
    }

    while idx < tokens.len() {
        let key = tokens[idx].to_ascii_lowercase();
        idx += 1;
        match key.as_str() {
            "is_row" => {
                let Some(value) = tokens.get(idx) else {
                    return Err(format!("{rel_path}:{line_no} missing is_row value"));
                };
                clip.is_row = parse_bool_token(value, rel_path, line_no)?;
                idx += 1;
            }
            "partial" => {
                let Some(start) = tokens.get(idx) else {
                    return Err(format!("{rel_path}:{line_no} missing partial start value"));
                };
                let Some(end) = tokens.get(idx + 1) else {
                    return Err(format!("{rel_path}:{line_no} missing partial end value"));
                };
                clip.partial = (parse_usize_token(start, rel_path, line_no)?, parse_usize_token(end, rel_path, line_no)?);
                idx += 2;
            }
            "start" | "start_frame" => {
                let Some(value) = tokens.get(idx) else {
                    return Err(format!("{rel_path}:{line_no} missing start_frame value"));
                };
                clip.start_frame = parse_usize_token(value, rel_path, line_no)?;
                idx += 1;
            }
            "dir" => {
                let Some(value) = tokens.get(idx) else {
                    return Err(format!("{rel_path}:{line_no} missing dir value"));
                };
                clip.dir = parse_animation_dir(value, rel_path, line_no)?;
                idx += 1;
            }
            "reps" => {
                let Some(value) = tokens.get(idx) else {
                    return Err(format!("{rel_path}:{line_no} missing reps value"));
                };
                clip.reps = parse_usize_token(value, rel_path, line_no)? as u32;
                idx += 1;
            }
            "dur_frame" => {
                let Some(value) = tokens.get(idx) else {
                    return Err(format!("{rel_path}:{line_no} missing dur_frame value"));
                };
                clip.dur_frame = parse_usize_token(value, rel_path, line_no)? as u32;
                idx += 1;
            }
            "dur_rep" => {
                let Some(value) = tokens.get(idx) else {
                    return Err(format!("{rel_path}:{line_no} missing dur_rep value"));
                };
                clip.dur_rep = parse_usize_token(value, rel_path, line_no)? as u32;
                idx += 1;
            }
            other => {
                return Err(format!("{rel_path}:{line_no} has unknown clip token '{other}'"));
            }
        }
    }

    Ok(clip)
}

fn split_field_line(line: &str, rel_path: &str, line_no: usize) -> Result<(String, String), String> {
    if let Some((key, value)) = line.split_once(':') {
        let key = key.trim();
        let value = value.trim();
        if key.is_empty() || value.is_empty() {
            return Err(format!("{rel_path}:{line_no} has an invalid field assignment"));
        }
        return Ok((key.to_string(), value.to_string()));
    }
    if let Some((key, value)) = line.split_once('=') {
        let key = key.trim();
        let value = value.trim();
        if key.is_empty() || value.is_empty() {
            return Err(format!("{rel_path}:{line_no} has an invalid field assignment"));
        }
        return Ok((key.to_string(), value.to_string()));
    }

    let mut parts = line.splitn(2, char::is_whitespace);
    let Some(key) = parts.next().map(str::trim) else {
        return Err(format!("{rel_path}:{line_no} has an invalid field assignment"));
    };
    let Some(value) = parts.next().map(str::trim) else {
        return Err(format!("{rel_path}:{line_no} field '{key}' is missing a value"));
    };
    if key.is_empty() || value.is_empty() {
        return Err(format!("{rel_path}:{line_no} has an invalid field assignment"));
    }
    Ok((key.to_string(), value.to_string()))
}

fn parse_bool_value(raw: &str, rel_path: &str, line_no: usize) -> Result<bool, String> {
    match parse_text_value(raw).to_ascii_lowercase().as_str() {
        "true" | "yes" | "on" => Ok(true),
        "false" | "no" | "off" => Ok(false),
        other => Err(format!("{rel_path}:{line_no} has invalid boolean value '{other}'")),
    }
}

fn parse_bool_token(raw: &str, rel_path: &str, line_no: usize) -> Result<bool, String> {
    parse_bool_value(raw, rel_path, line_no)
}

fn parse_usize_value(raw: &str, rel_path: &str, line_no: usize) -> Result<usize, String> {
    let value = parse_text_value(raw);
    value
        .parse::<usize>()
        .map_err(|err| format!("{rel_path}:{line_no} has invalid usize value '{value}': {err}"))
}

fn parse_f32_value(raw: &str, rel_path: &str, line_no: usize) -> Result<f32, String> {
    let value = parse_text_value(raw);
    value
        .parse::<f32>()
        .map_err(|err| format!("{rel_path}:{line_no} has invalid f32 value '{value}': {err}"))
}

fn parse_usize_token(raw: &str, rel_path: &str, line_no: usize) -> Result<usize, String> {
    raw.parse::<usize>()
        .map_err(|err| format!("{rel_path}:{line_no} has invalid usize value '{raw}': {err}"))
}

fn parse_usize_pair(raw: &str, rel_path: &str, line_no: usize) -> Result<(usize, usize), String> {
    let values = split_scalar_values(raw);
    if values.len() != 2 {
        return Err(format!("{rel_path}:{line_no} expected two usize values, found '{}'", raw.trim()));
    }
    Ok((
        parse_usize_token(values[0], rel_path, line_no)?,
        parse_usize_token(values[1], rel_path, line_no)?,
    ))
}

fn parse_f32_pair(raw: &str, rel_path: &str, line_no: usize) -> Result<[f32; 2], String> {
    let values = split_scalar_values(raw);
    if values.len() != 2 {
        return Err(format!("{rel_path}:{line_no} expected two f32 values, found '{}'", raw.trim()));
    }
    Ok([
        parse_f32_value(values[0], rel_path, line_no)?,
        parse_f32_value(values[1], rel_path, line_no)?,
    ])
}

fn parse_u8_quad(raw: &str, rel_path: &str, line_no: usize) -> Result<[u8; 4], String> {
    let values = split_scalar_values(raw);
    if values.len() != 4 {
        return Err(format!("{rel_path}:{line_no} expected four u8 values, found '{}'", raw.trim()));
    }
    Ok([
        parse_u8_token(values[0], rel_path, line_no)?,
        parse_u8_token(values[1], rel_path, line_no)?,
        parse_u8_token(values[2], rel_path, line_no)?,
        parse_u8_token(values[3], rel_path, line_no)?,
    ])
}

fn parse_u8_token(raw: &str, rel_path: &str, line_no: usize) -> Result<u8, String> {
    raw.parse::<u8>()
        .map_err(|err| format!("{rel_path}:{line_no} has invalid u8 value '{raw}': {err}"))
}

fn parse_animation_dir(raw: &str, rel_path: &str, line_no: usize) -> Result<u8, String> {
    match parse_text_value(raw).to_ascii_lowercase().as_str() {
        "forwards" | "forward" | "none" | "unset" => Ok(AnimationSeri::DIR_UNSET),
        "backwards" | "backward" | "reverse" => Ok(AnimationSeri::DIR_BACKWARDS),
        "pingpong" | "ping-pong" | "ping_pong" => Ok(AnimationSeri::DIR_PINGPONG),
        other => Err(format!("{rel_path}:{line_no} has invalid direction '{other}'")),
    }
}

fn parse_cardinal_rotation(raw: &str, rel_path: &str, line_no: usize) -> Result<CardinalRotation, String> {
    match parse_text_value(raw).to_ascii_lowercase().as_str() {
        "none" => Ok(CardinalRotation::None),
        "west" => Ok(CardinalRotation::West),
        "north" => Ok(CardinalRotation::North),
        "east" => Ok(CardinalRotation::East),
        other => Err(format!("{rel_path}:{line_no} has invalid cardinal rotation '{other}'")),
    }
}

fn parse_string_vec(raw: &str, rel_path: &str, line_no: usize) -> Result<Vec<String>, String> {
    let raw = raw.trim();
    if raw.is_empty() || raw == "[]" {
        return Ok(Vec::new());
    }
    if raw.starts_with('[') {
        return ron::from_str::<Vec<String>>(raw)
            .map_err(|err| format!("{rel_path}:{line_no} has invalid string list '{raw}': {err}"));
    }
    let mut out = Vec::new();
    for part in raw.split(',') {
        let value = parse_text_value(part);
        if !value.is_empty() {
            out.push(value);
        }
    }
    if out.is_empty() {
        out.push(parse_text_value(raw));
    }
    Ok(out)
}

fn split_scalar_values(raw: &str) -> Vec<&str> {
    raw.trim()
        .trim_start_matches('(')
        .trim_end_matches(')')
        .trim_start_matches('[')
        .trim_end_matches(']')
        .split(|c: char| c.is_whitespace() || c == ',')
        .filter(|part| !part.is_empty())
        .collect()
}

fn parse_text_value(raw: &str) -> String {
    let raw = raw.trim().trim_end_matches(',').trim();
    if let Some(quoted) = raw
        .strip_prefix('"')
        .and_then(|rest| rest.strip_suffix('"'))
    {
        return quoted.to_string();
    }
    raw.to_string()
}

fn strip_inline_comment(line: &str) -> &str {
    let Some(comment_idx) = line.find('#') else {
        return line;
    };
    &line[..comment_idx]
}

#[allow(dead_code)]
fn _deserialize_ron_value<T: DeserializeOwned>(raw: &str) -> Result<T, String> {
    ron::from_str::<T>(raw).map_err(|err| err.to_string())
}
