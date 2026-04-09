use bevy::platform::collections::*;
#[allow(unused_imports)]
use bevy::prelude::*;
use common::def_db::*;
use common::log_targets::SPRITE_INIT;
use serde::de::DeserializeOwned;
use ::sprite_shared::*;

pub struct LoadedSpriteDef {
    pub rel_path: String,
    pub is_abstract: bool,
    pub seri: SpriteConfigSeri,
}

struct RawSpriteDef {
    id: String,
    rel_path: String,
    base_id: Option<String>,
    is_abstract: bool,
    payload: RawSpritePayload,
}

enum RawSpritePayload {
    Full(SpriteConfigSeri),
    Fields(RawSpriteFields),
}

#[derive(Default)]
struct RawSpriteFields {
    name: Option<String>,
    fallback_img_path: Option<String>,
    z: Option<f32>,
    y_sort: Option<f32>,
    mapped_anims: Option<HashMap<(String, String, String, String), String>>,
    parent_cat: Option<String>,
    tags: Option<HashSet<String>>,
    shares_tag: Option<Vec<bool>>,
    children_sprites: Option<Vec<String>>,
    sfx_every_n_frames: Option<SfxEveryNframesSeri>,
    loop_sfx: Option<SfxLoopSeri>,
    interval_sfx: Option<SfxTimeIntervalSeri>,
    directionable: Option<bool>,
    movement_based: Option<bool>,
    grounding_based: Option<bool>,
    add_up_z_with_anim: Option<bool>,
    is_being_root_sprite: Option<bool>,
    visibility: Option<Option<u8>>,
    offset4children: Option<HashMap<String, (f32, f32, String)>>,
    exclude_from_sys: Option<bool>,
    baseline_move_speed: Option<f32>,
    exclude_from_normal_size_modifier: Option<bool>,
    offset: Option<(f32, f32)>,
    scale: Option<(f32, f32)>,
    scale_up_down: Option<(f32, f32)>,
    scale_sideways: Option<(f32, f32)>,
    flip_horiz_if_dir: Option<Option<u8>>,
    offset_up_down: Option<(f32, f32)>,
    offset_down: Option<(f32, f32)>,
    offset_up: Option<(f32, f32)>,
    offset_sideways: Option<(f32, f32)>,
    extra_y_offset_per_scale_inc: Option<Option<f32>>,
}

enum SpriteBlock {
    MappedAnims,
    Offset4Children,
}

pub fn load_sprite_defs_from_filesystem() -> Vec<LoadedSpriteDef> {
    let Ok(mut files) = discover_assets_files_by_suffixes(&[".sprite.ron", ".sprite"]) else {
        error!(target: SPRITE_INIT, "Failed discovering .sprite/.sprite.ron files");
        return Vec::new();
    };
    files.sort_by(|(a, _), (b, _)| {
        a.precedence_rank()
            .cmp(&b.precedence_rank())
            .then_with(|| a.rel_path.cmp(&b.rel_path))
    });

    let mut raw_by_id = HashMap::<String, RawSpriteDef>::default();

    for (source, path) in files {
        let Ok(content) = std::fs::read_to_string(&path) else {
            warn!(target: SPRITE_INIT, "Failed reading sprite file '{}'", path.to_string_lossy());
            continue;
        };

        let parsed_defs = if source.rel_path.ends_with(".sprite.ron") {
            parse_legacy_sprite_ron_file(&content, &source.rel_path)
        } else {
            parse_sprite_file(&content, &source.rel_path)
        };

        let Ok(parsed_defs) = parsed_defs else {
            warn!(target: SPRITE_INIT, "Failed parsing sprite defs in '{}'", source.rel_path);
            continue;
        };

        for parsed in parsed_defs {
            let id = parsed.id.clone();
            if let Some(prev) = raw_by_id.insert(id.clone(), parsed) {
                debug!(
                    target: SPRITE_INIT,
                    "SpriteConfig '{}' overridden: '{}' -> '{}'",
                    id,
                    prev.rel_path,
                    source.rel_path
                );
            }
        }
    }

    let mut resolved = HashMap::<String, SpriteConfigSeri>::default();
    let mut resolving = HashSet::<String>::default();
    let mut out = Vec::with_capacity(raw_by_id.len());

    let mut ids = Vec::with_capacity(raw_by_id.len());
    ids.extend(raw_by_id.keys().cloned());
    ids.sort();

    for id in ids {
        let Some(raw) = raw_by_id.get(&id) else {
            continue;
        };
        let Ok(seri) = resolve_sprite_seri(&id, &raw_by_id, &mut resolved, &mut resolving) else {
            error!(target: SPRITE_INIT, "Failed resolving sprite '{}'", id);
            continue;
        };
        out.push(LoadedSpriteDef {
            rel_path: raw.rel_path.clone(),
            is_abstract: raw.is_abstract,
            seri,
        });
    }

    if out.is_empty() {
        error!(target: SPRITE_INIT, "No sprite defs loaded from filesystem");
    } else {
        debug!(target: SPRITE_INIT, "Loaded {} sprite defs from filesystem", out.len());
    }

    out
}

fn resolve_sprite_seri(
    id: &str,
    raw_by_id: &HashMap<String, RawSpriteDef>,
    resolved: &mut HashMap<String, SpriteConfigSeri>,
    resolving: &mut HashSet<String>,
) -> Result<SpriteConfigSeri, String> {
    if let Some(seri) = resolved.get(id) {
        return Ok(seri.clone());
    }
    if !resolving.insert(id.to_string()) {
        return Err(format!("Sprite '{}' has a cyclic inheritance chain", id));
    }

    let Some(raw) = raw_by_id.get(id) else {
        resolving.remove(id);
        return Err(format!("Sprite '{}' not found while resolving inheritance", id));
    };

    let mut seri = match &raw.payload {
        RawSpritePayload::Full(seri) => seri.clone(),
        RawSpritePayload::Fields(fields) => {
            let mut seri = if let Some(base_id) = &raw.base_id {
                resolve_sprite_seri(base_id, raw_by_id, resolved, resolving)?
            } else {
                SpriteConfigSeri::default()
            };
            seri.id = raw.id.clone();
            apply_raw_sprite_fields(&mut seri, fields);
            seri
        }
    };
    if seri.id.trim().is_empty() {
        seri.id = raw.id.clone();
    }

    resolving.remove(id);
    resolved.insert(id.to_string(), seri.clone());
    Ok(seri)
}

fn apply_raw_sprite_fields(seri: &mut SpriteConfigSeri, fields: &RawSpriteFields) {
    if let Some(name) = &fields.name {
        seri.name = name.clone();
    }
    if let Some(fallback_img_path) = &fields.fallback_img_path {
        seri.fallback_img_path = fallback_img_path.clone();
    }
    if let Some(z) = fields.z {
        seri.z = z;
    }
    if let Some(y_sort) = fields.y_sort {
        seri.y_sort = y_sort;
    }
    if let Some(mapped_anims) = &fields.mapped_anims {
        seri.mapped_anims = mapped_anims.clone();
    }
    if let Some(parent_cat) = &fields.parent_cat {
        seri.parent_cat = parent_cat.clone();
    }
    if let Some(tags) = &fields.tags {
        seri.tags = tags.clone();
    }
    if let Some(shares_tag) = &fields.shares_tag {
        seri.shares_tag = shares_tag.clone();
    }
    if let Some(children_sprites) = &fields.children_sprites {
        seri.children_sprites = children_sprites.clone();
    }
    if let Some(sfx_every_n_frames) = &fields.sfx_every_n_frames {
        seri.sfx_every_n_frames = SfxEveryNframesSeri {
            paths: sfx_every_n_frames.paths.clone(),
            n: sfx_every_n_frames.n,
        };
    }
    if let Some(loop_sfx) = &fields.loop_sfx {
        seri.loop_sfx = SfxLoopSeri {
            paths: loop_sfx.paths.clone(),
            condition: loop_sfx.condition.clone(),
        };
    }
    if let Some(interval_sfx) = &fields.interval_sfx {
        seri.interval_sfx = SfxTimeIntervalSeri {
            paths: interval_sfx.paths.clone(),
            condition: interval_sfx.condition.clone(),
            secs: interval_sfx.secs,
            shorten_with_anim_playing_speed: interval_sfx.shorten_with_anim_playing_speed,
        };
    }
    if let Some(directionable) = fields.directionable {
        seri.directionable = directionable;
    }
    if let Some(movement_based) = fields.movement_based {
        seri.movement_based = movement_based;
    }
    if let Some(grounding_based) = fields.grounding_based {
        seri.grounding_based = grounding_based;
    }
    if let Some(add_up_z_with_anim) = fields.add_up_z_with_anim {
        seri.add_up_z_with_anim = add_up_z_with_anim;
    }
    if let Some(is_being_root_sprite) = fields.is_being_root_sprite {
        seri.is_being_root_sprite = is_being_root_sprite;
    }
    if let Some(visibility) = fields.visibility {
        seri.visibility = visibility;
    }
    if let Some(offset4children) = &fields.offset4children {
        seri.offset4children = offset4children.clone();
    }
    if let Some(exclude_from_sys) = fields.exclude_from_sys {
        seri.exclude_from_sys = exclude_from_sys;
    }
    if let Some(baseline_move_speed) = fields.baseline_move_speed {
        seri.baseline_move_speed = baseline_move_speed;
    }
    if let Some(exclude_from_normal_size_modifier) = fields.exclude_from_normal_size_modifier {
        seri.exclude_from_normal_size_modifier = exclude_from_normal_size_modifier;
    }
    if let Some(offset) = fields.offset {
        seri.offset = offset;
    }
    if let Some(scale) = fields.scale {
        seri.scale = scale;
    }
    if let Some(scale_up_down) = fields.scale_up_down {
        seri.scale_up_down = scale_up_down;
    }
    if let Some(scale_sideways) = fields.scale_sideways {
        seri.scale_sideways = scale_sideways;
    }
    if let Some(flip_horiz_if_dir) = fields.flip_horiz_if_dir {
        seri.flip_horiz_if_dir = flip_horiz_if_dir;
    }
    if let Some(offset_up_down) = fields.offset_up_down {
        seri.offset_up_down = offset_up_down;
    }
    if let Some(offset_down) = fields.offset_down {
        seri.offset_down = offset_down;
    }
    if let Some(offset_up) = fields.offset_up {
        seri.offset_up = offset_up;
    }
    if let Some(offset_sideways) = fields.offset_sideways {
        seri.offset_sideways = offset_sideways;
    }
    if let Some(extra_y_offset_per_scale_inc) = fields.extra_y_offset_per_scale_inc {
        seri.extra_y_offset_per_scale_inc = extra_y_offset_per_scale_inc;
    }
}

fn parse_legacy_sprite_ron_file(content: &str, rel_path: &str) -> Result<Vec<RawSpriteDef>, String> {
    let parsed = ron::from_str::<OneOrMany<SpriteConfigSeri>>(content)
        .map_err(|err| format!("{rel_path} has invalid .sprite.ron content: {err}"))?;
    let mut out = Vec::new();
    for seri in parsed.into_vec() {
        if seri.id.trim().is_empty() {
            return Err(format!("{rel_path} contains a SpriteConfigSeri with empty id"));
        }
        out.push(RawSpriteDef {
            id: seri.id.clone(),
            rel_path: rel_path.to_string(),
            base_id: None,
            is_abstract: false,
            payload: RawSpritePayload::Full(seri),
        });
    }
    if out.is_empty() {
        return Err(format!("{rel_path} did not contain any sprite defs"));
    }
    Ok(out)
}

fn parse_sprite_file(content: &str, rel_path: &str) -> Result<Vec<RawSpriteDef>, String> {
    let mut out = Vec::new();
    let mut current: Option<RawSpriteDef> = None;
    let mut active_block = None::<SpriteBlock>;

    for (idx, raw_line) in content.lines().enumerate() {
        let line_no = idx + 1;
        let line_without_comment = strip_inline_comment(raw_line);
        let trimmed = line_without_comment.trim();
        if trimmed.is_empty() {
            continue;
        }

        if trimmed == "}" {
            if active_block.is_some() {
                active_block = None;
                continue;
            }
            let Some(def) = current.take() else {
                return Err(format!("{rel_path}:{line_no} has an unmatched closing brace"));
            };
            out.push(def);
            continue;
        }

        if current.is_none() {
            let def = parse_sprite_header(trimmed, rel_path, line_no)?;
            current = Some(def);
            continue;
        }

        let Some(def) = current.as_mut() else {
            return Err(format!("{rel_path}:{line_no} parser lost current sprite state"));
        };

        match active_block {
            Some(SpriteBlock::MappedAnims) => {
                let (anim_type, anim_id) = parse_mapped_anim_entry(trimmed, rel_path, line_no)?;
                let RawSpritePayload::Fields(fields) = &mut def.payload else {
                    return Err(format!("{rel_path}:{line_no} expected editable sprite fields"));
                };
                let mapped_anims = fields.mapped_anims.get_or_insert_with(HashMap::default);
                mapped_anims.insert(anim_type, anim_id);
                continue;
            }
            Some(SpriteBlock::Offset4Children) => {
                let (cat, offset_cfg) = parse_offset4children_entry(trimmed, rel_path, line_no)?;
                let RawSpritePayload::Fields(fields) = &mut def.payload else {
                    return Err(format!("{rel_path}:{line_no} expected editable sprite fields"));
                };
                let offsets = fields.offset4children.get_or_insert_with(HashMap::default);
                offsets.insert(cat, offset_cfg);
                continue;
            }
            None => {}
        }

        if starts_block(trimmed, "mapped_anims") {
            active_block = Some(SpriteBlock::MappedAnims);
            let RawSpritePayload::Fields(fields) = &mut def.payload else {
                return Err(format!("{rel_path}:{line_no} expected editable sprite fields"));
            };
            if fields.mapped_anims.is_none() {
                fields.mapped_anims = Some(HashMap::default());
            }
            continue;
        }
        if starts_block(trimmed, "offset4children") {
            active_block = Some(SpriteBlock::Offset4Children);
            let RawSpritePayload::Fields(fields) = &mut def.payload else {
                return Err(format!("{rel_path}:{line_no} expected editable sprite fields"));
            };
            if fields.offset4children.is_none() {
                fields.offset4children = Some(HashMap::default());
            }
            continue;
        }

        let (key, raw_value) = split_field_line(trimmed, rel_path, line_no)?;
        parse_sprite_field_assignment(def, &key, &raw_value, rel_path, line_no)?;
    }

    if active_block.is_some() {
        return Err(format!("{rel_path} ended while still inside a sprite block section"));
    }
    if let Some(def) = current.take() {
        return Err(format!("{rel_path} ended before closing sprite '{}'", def.id));
    }
    if out.is_empty() {
        return Err(format!("{rel_path} did not contain any sprite blocks"));
    }
    Ok(out)
}

fn parse_sprite_header(trimmed: &str, rel_path: &str, line_no: usize) -> Result<RawSpriteDef, String> {
    let Some(header) = trimmed.strip_suffix('{').map(str::trim) else {
        return Err(format!("{rel_path}:{line_no} sprite header must end with '{{'"));
    };

    let mut tokens = header.split_whitespace();
    let mut is_abstract = false;
    let mut saw_sprite = false;
    let mut id = None::<String>;
    let mut base_id = None::<String>;

    while let Some(token) = tokens.next() {
        match token {
            "abstract" => {
                is_abstract = true;
            }
            "sprite" => {
                saw_sprite = true;
                let Some(raw_id) = tokens.next() else {
                    return Err(format!("{rel_path}:{line_no} missing sprite id"));
                };
                id = Some(parse_text_value(raw_id));
            }
            "extends" => {
                let Some(raw_base) = tokens.next() else {
                    return Err(format!("{rel_path}:{line_no} missing extends id"));
                };
                base_id = Some(parse_text_value(raw_base));
            }
            other if !saw_sprite => {
                return Err(format!("{rel_path}:{line_no} expected 'sprite' but found '{other}'"));
            }
            other => {
                return Err(format!("{rel_path}:{line_no} unexpected token '{other}' in sprite header"));
            }
        }
    }

    let Some(id) = id else {
        return Err(format!("{rel_path}:{line_no} missing sprite id"));
    };
    if id.trim().is_empty() {
        return Err(format!("{rel_path}:{line_no} sprite id is empty"));
    }

    Ok(RawSpriteDef {
        id,
        rel_path: rel_path.to_string(),
        base_id,
        is_abstract,
        payload: RawSpritePayload::Fields(RawSpriteFields::default()),
    })
}

fn parse_sprite_field_assignment(
    def: &mut RawSpriteDef,
    key: &str,
    raw_value: &str,
    rel_path: &str,
    line_no: usize,
) -> Result<(), String> {
    let key = key.trim().to_ascii_lowercase();
    let RawSpritePayload::Fields(fields) = &mut def.payload else {
        return Err(format!("{rel_path}:{line_no} sprite field assignments require editable fields"));
    };

    match key.as_str() {
        "id" => {
            let id = parse_text_value(raw_value);
            if id.trim().is_empty() {
                return Err(format!("{rel_path}:{line_no} sprite id is empty"));
            }
            def.id = id;
        }
        "extends" | "base" => {
            let base_id = parse_text_value(raw_value);
            if base_id.trim().is_empty() {
                return Err(format!("{rel_path}:{line_no} base sprite id is empty"));
            }
            def.base_id = Some(base_id);
        }
        "abstract" => {
            def.is_abstract = parse_bool_value(raw_value, rel_path, line_no)?;
        }
        "name" => {
            fields.name = Some(parse_text_value(raw_value));
        }
        "fallback_img_path" => {
            fields.fallback_img_path = Some(parse_text_value(raw_value));
        }
        "z" => {
            fields.z = Some(parse_f32_value(raw_value, rel_path, line_no)?);
        }
        "y_sort" => {
            fields.y_sort = Some(parse_f32_value(raw_value, rel_path, line_no)?);
        }
        "mapped_anims" | "mapped_animations" => {
            if raw_value.trim().starts_with('{') {
                fields.mapped_anims = Some(parse_ron_value(raw_value, rel_path, line_no)?);
            } else {
                let (anim_type, anim_id) = parse_mapped_anim_entry(raw_value, rel_path, line_no)?;
                let mapped_anims = fields.mapped_anims.get_or_insert_with(HashMap::default);
                mapped_anims.insert(anim_type, anim_id);
            }
        }
        "mapped_anim" | "anim" => {
            let (anim_type, anim_id) = parse_mapped_anim_entry(raw_value, rel_path, line_no)?;
            let mapped_anims = fields.mapped_anims.get_or_insert_with(HashMap::default);
            mapped_anims.insert(anim_type, anim_id);
        }
        "parent_cat" => {
            fields.parent_cat = Some(parse_text_value(raw_value));
        }
        "tags" => {
            fields.tags = Some(parse_string_set(raw_value, rel_path, line_no)?);
        }
        "shares_tag" => {
            fields.shares_tag = Some(parse_bool_vec(raw_value, rel_path, line_no)?);
        }
        "children_sprites" => {
            fields.children_sprites = Some(parse_string_vec(raw_value, rel_path, line_no)?);
        }
        "sfx_every_n_frames" => {
            fields.sfx_every_n_frames = Some(parse_ron_value(raw_value, rel_path, line_no)?);
        }
        "sfx_every_n_frames_paths" => {
            let mut sfx = fields.sfx_every_n_frames.take().unwrap_or_default();
            sfx.paths = parse_string_vec(raw_value, rel_path, line_no)?;
            fields.sfx_every_n_frames = Some(sfx);
        }
        "sfx_every_n_frames_n" => {
            let mut sfx = fields.sfx_every_n_frames.take().unwrap_or_default();
            sfx.n = parse_f32_value(raw_value, rel_path, line_no)?;
            fields.sfx_every_n_frames = Some(sfx);
        }
        "loop_sfx" => {
            fields.loop_sfx = Some(parse_ron_value(raw_value, rel_path, line_no)?);
        }
        "loop_sfx_paths" => {
            let mut loop_sfx = fields.loop_sfx.take().unwrap_or_default();
            loop_sfx.paths = parse_string_vec(raw_value, rel_path, line_no)?;
            fields.loop_sfx = Some(loop_sfx);
        }
        "loop_sfx_condition" => {
            let mut loop_sfx = fields.loop_sfx.take().unwrap_or_default();
            loop_sfx.condition = parse_text_value(raw_value);
            fields.loop_sfx = Some(loop_sfx);
        }
        "interval_sfx" => {
            fields.interval_sfx = Some(parse_ron_value(raw_value, rel_path, line_no)?);
        }
        "interval_sfx_paths" => {
            let mut interval_sfx = fields.interval_sfx.take().unwrap_or_default();
            interval_sfx.paths = parse_string_vec(raw_value, rel_path, line_no)?;
            fields.interval_sfx = Some(interval_sfx);
        }
        "interval_sfx_condition" => {
            let mut interval_sfx = fields.interval_sfx.take().unwrap_or_default();
            interval_sfx.condition = parse_text_value(raw_value);
            fields.interval_sfx = Some(interval_sfx);
        }
        "interval_sfx_secs" => {
            let mut interval_sfx = fields.interval_sfx.take().unwrap_or_default();
            interval_sfx.secs = parse_f32_value(raw_value, rel_path, line_no)?;
            fields.interval_sfx = Some(interval_sfx);
        }
        "interval_sfx_shorten_with_anim_playing_speed" => {
            let mut interval_sfx = fields.interval_sfx.take().unwrap_or_default();
            interval_sfx.shorten_with_anim_playing_speed =
                parse_bool_value(raw_value, rel_path, line_no)?;
            fields.interval_sfx = Some(interval_sfx);
        }
        "directionable" => {
            fields.directionable = Some(parse_bool_value(raw_value, rel_path, line_no)?);
        }
        "movement_based" => {
            fields.movement_based = Some(parse_bool_value(raw_value, rel_path, line_no)?);
        }
        "grounding_based" => {
            fields.grounding_based = Some(parse_bool_value(raw_value, rel_path, line_no)?);
        }
        "add_up_z_with_anim" => {
            fields.add_up_z_with_anim = Some(parse_bool_value(raw_value, rel_path, line_no)?);
        }
        "is_being_root_sprite" => {
            fields.is_being_root_sprite = Some(parse_bool_value(raw_value, rel_path, line_no)?);
        }
        "visibility" => {
            fields.visibility = Some(parse_option_u8_value(raw_value, rel_path, line_no)?);
        }
        "offset4children" => {
            if raw_value.trim().starts_with('{') {
                fields.offset4children = Some(parse_ron_value(raw_value, rel_path, line_no)?);
            } else {
                let (cat, value) = parse_offset4children_entry(raw_value, rel_path, line_no)?;
                let offset4children = fields.offset4children.get_or_insert_with(HashMap::default);
                offset4children.insert(cat, value);
            }
        }
        "offset4child" => {
            let (cat, value) = parse_offset4children_entry(raw_value, rel_path, line_no)?;
            let offset4children = fields.offset4children.get_or_insert_with(HashMap::default);
            offset4children.insert(cat, value);
        }
        "exclude_from_sys" => {
            fields.exclude_from_sys = Some(parse_bool_value(raw_value, rel_path, line_no)?);
        }
        "baseline_move_speed" => {
            fields.baseline_move_speed = Some(parse_f32_value(raw_value, rel_path, line_no)?);
        }
        "exclude_from_normal_size_modifier" => {
            fields.exclude_from_normal_size_modifier =
                Some(parse_bool_value(raw_value, rel_path, line_no)?);
        }
        "offset" => {
            fields.offset = Some(parse_f32_pair(raw_value, rel_path, line_no)?);
        }
        "scale" => {
            fields.scale = Some(parse_f32_pair(raw_value, rel_path, line_no)?);
        }
        "scale_up_down" => {
            fields.scale_up_down = Some(parse_f32_pair(raw_value, rel_path, line_no)?);
        }
        "scale_sideways" => {
            fields.scale_sideways = Some(parse_f32_pair(raw_value, rel_path, line_no)?);
        }
        "flip_horiz_if_dir" => {
            fields.flip_horiz_if_dir = Some(parse_option_u8_value(raw_value, rel_path, line_no)?);
        }
        "offset_up_down" => {
            fields.offset_up_down = Some(parse_f32_pair(raw_value, rel_path, line_no)?);
        }
        "offset_down" => {
            fields.offset_down = Some(parse_f32_pair(raw_value, rel_path, line_no)?);
        }
        "offset_up" => {
            fields.offset_up = Some(parse_f32_pair(raw_value, rel_path, line_no)?);
        }
        "offset_sideways" => {
            fields.offset_sideways = Some(parse_f32_pair(raw_value, rel_path, line_no)?);
        }
        "extra_y_offset_per_scale_inc" => {
            fields.extra_y_offset_per_scale_inc =
                Some(parse_option_f32_value(raw_value, rel_path, line_no)?);
        }
        other => {
            return Err(format!("{rel_path}:{line_no} has unknown sprite field '{other}'"));
        }
    }
    Ok(())
}

fn starts_block(line: &str, field_name: &str) -> bool {
    let normalized = line
        .to_ascii_lowercase()
        .replace(char::is_whitespace, "");
    normalized == format!("{field_name}{{")
}

fn parse_mapped_anim_entry(
    line: &str,
    rel_path: &str,
    line_no: usize,
) -> Result<((String, String, String, String), String), String> {
    let line = line.trim().trim_end_matches(',').trim();
    let (left, right) = if let Some((left, right)) = line.split_once('=') {
        (left.trim(), right.trim())
    } else if let Some((left, right)) = line.split_once(':') {
        (left.trim(), right.trim())
    } else {
        let values = split_scalar_values(line);
        if values.len() < 4 {
            return Err(format!("{rel_path}:{line_no} mapped_anim needs at least 3 key values plus anim id"));
        }
        let anim_id = parse_text_value(values[values.len() - 1]);
        let anim_type = parse_anim_type_values(&values[..values.len() - 1], rel_path, line_no)?;
        return Ok((anim_type, anim_id));
    };

    let anim_id = parse_text_value(right);
    let key_values = split_scalar_values(left);
    let anim_type = parse_anim_type_values(&key_values, rel_path, line_no)?;
    Ok((anim_type, anim_id))
}

fn parse_anim_type_values(
    values: &[&str],
    rel_path: &str,
    line_no: usize,
) -> Result<(String, String, String, String), String> {
    if !(3..=4).contains(&values.len()) {
        return Err(format!(
            "{rel_path}:{line_no} mapped_anim key needs 3 or 4 values, found {}",
            values.len()
        ));
    }
    let dir = parse_text_value(values[0]);
    let moving = parse_text_value(values[1]);
    let grounding = parse_text_value(values[2]);
    let state = values.get(3).map(|value| parse_text_value(value)).unwrap_or_default();
    Ok((
        normalize_placeholder_text(dir),
        normalize_placeholder_text(moving),
        normalize_placeholder_text(grounding),
        normalize_placeholder_text(state),
    ))
}

fn parse_offset4children_entry(
    line: &str,
    rel_path: &str,
    line_no: usize,
) -> Result<(String, (f32, f32, String)), String> {
    let line = line.trim().trim_end_matches(',').trim();
    if let Some((left, right)) = line.split_once('=').or_else(|| line.split_once(':')) {
        let cat = parse_text_value(left);
        let values = split_scalar_values(right);
        if values.len() < 2 {
            return Err(format!("{rel_path}:{line_no} offset4children entry needs at least x and y"));
        }
        let x = parse_f32_token(values[0], rel_path, line_no)?;
        let y = parse_f32_token(values[1], rel_path, line_no)?;
        let direction = values.get(2).map(|v| parse_text_value(v)).unwrap_or_default();
        return Ok((cat, (x, y, normalize_placeholder_text(direction))));
    }

    let values = split_scalar_values(line);
    if values.len() < 3 {
        return Err(format!("{rel_path}:{line_no} offset4children entry must be: cat x y [direction]"));
    }
    let cat = parse_text_value(values[0]);
    let x = parse_f32_token(values[1], rel_path, line_no)?;
    let y = parse_f32_token(values[2], rel_path, line_no)?;
    let direction = values.get(3).map(|v| parse_text_value(v)).unwrap_or_default();
    Ok((cat, (x, y, normalize_placeholder_text(direction))))
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

fn parse_ron_value<T: DeserializeOwned>(raw: &str, rel_path: &str, line_no: usize) -> Result<T, String> {
    ron::from_str::<T>(raw.trim().trim_end_matches(',').trim())
        .map_err(|err| format!("{rel_path}:{line_no} has invalid value '{}': {err}", raw.trim()))
}

fn parse_string_vec(raw: &str, rel_path: &str, line_no: usize) -> Result<Vec<String>, String> {
    let raw = raw.trim().trim_end_matches(',').trim();
    if raw.is_empty() || raw == "[]" {
        return Ok(Vec::new());
    }
    if raw.starts_with('[') {
        return ron::from_str::<Vec<String>>(raw)
            .map_err(|err| format!("{rel_path}:{line_no} has invalid string list '{raw}': {err}"));
    }
    if raw.starts_with('"') && raw.ends_with('"') {
        return Ok(vec![parse_text_value(raw)]);
    }
    let out: Vec<String> = raw
        .split(|c: char| c == ',' || c.is_whitespace())
        .filter(|part| !part.trim().is_empty())
        .map(parse_text_value)
        .collect();
    Ok(out)
}

fn parse_string_set(raw: &str, rel_path: &str, line_no: usize) -> Result<HashSet<String>, String> {
    let values = parse_string_vec(raw, rel_path, line_no)?;
    let mut set = HashSet::with_capacity(values.len());
    for value in values {
        set.insert(value);
    }
    Ok(set)
}

fn parse_bool_vec(raw: &str, rel_path: &str, line_no: usize) -> Result<Vec<bool>, String> {
    let raw = raw.trim().trim_end_matches(',').trim();
    if raw.is_empty() || raw == "[]" {
        return Ok(Vec::new());
    }
    if raw.starts_with('[') {
        return ron::from_str::<Vec<bool>>(raw)
            .map_err(|err| format!("{rel_path}:{line_no} has invalid bool list '{raw}': {err}"));
    }
    let mut out = Vec::new();
    for part in raw.split(|c: char| c == ',' || c.is_whitespace()) {
        if part.trim().is_empty() {
            continue;
        }
        out.push(parse_bool_value(part, rel_path, line_no)?);
    }
    Ok(out)
}

fn parse_bool_value(raw: &str, rel_path: &str, line_no: usize) -> Result<bool, String> {
    match parse_text_value(raw).to_ascii_lowercase().as_str() {
        "true" | "yes" | "on" | "1" => Ok(true),
        "false" | "no" | "off" | "0" => Ok(false),
        other => Err(format!("{rel_path}:{line_no} has invalid boolean value '{other}'")),
    }
}

fn parse_f32_value(raw: &str, rel_path: &str, line_no: usize) -> Result<f32, String> {
    let value = parse_text_value(raw);
    value
        .parse::<f32>()
        .map_err(|err| format!("{rel_path}:{line_no} has invalid f32 value '{value}': {err}"))
}

fn parse_f32_token(raw: &str, rel_path: &str, line_no: usize) -> Result<f32, String> {
    parse_text_value(raw)
        .parse::<f32>()
        .map_err(|err| format!("{rel_path}:{line_no} has invalid f32 value '{raw}': {err}"))
}

fn parse_u8_value(raw: &str, rel_path: &str, line_no: usize) -> Result<u8, String> {
    let value = parse_text_value(raw);
    value
        .parse::<u8>()
        .map_err(|err| format!("{rel_path}:{line_no} has invalid u8 value '{value}': {err}"))
}

fn parse_option_u8_value(raw: &str, rel_path: &str, line_no: usize) -> Result<Option<u8>, String> {
    let value = parse_text_value(raw);
    let normalized = value.to_ascii_lowercase();
    if normalized == "none" || normalized == "unset" || normalized == "null" {
        return Ok(None);
    }
    Ok(Some(parse_u8_value(&value, rel_path, line_no)?))
}

fn parse_option_f32_value(raw: &str, rel_path: &str, line_no: usize) -> Result<Option<f32>, String> {
    let value = parse_text_value(raw);
    let normalized = value.to_ascii_lowercase();
    if normalized == "none" || normalized == "unset" || normalized == "null" {
        return Ok(None);
    }
    Ok(Some(parse_f32_value(&value, rel_path, line_no)?))
}

fn parse_f32_pair(raw: &str, rel_path: &str, line_no: usize) -> Result<(f32, f32), String> {
    let values = split_scalar_values(raw);
    if values.len() != 2 {
        return Err(format!("{rel_path}:{line_no} expected two f32 values, found '{}'", raw.trim()));
    }
    Ok((
        parse_f32_token(values[0], rel_path, line_no)?,
        parse_f32_token(values[1], rel_path, line_no)?,
    ))
}

fn split_scalar_values(raw: &str) -> Vec<&str> {
    raw.trim()
        .trim_start_matches('(')
        .trim_end_matches(')')
        .trim_start_matches('[')
        .trim_end_matches(']')
        .split(|c: char| c.is_whitespace() || c == ',' || c == ';')
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

fn normalize_placeholder_text(value: String) -> String {
    if value == "_" || value.eq_ignore_ascii_case("none") {
        String::new()
    } else {
        value
    }
}

fn strip_inline_comment(line: &str) -> &str {
    let mut in_quotes = false;
    let bytes = line.as_bytes();
    let mut idx = 0usize;
    while idx < bytes.len() {
        let byte = bytes[idx];
        if byte == b'"' {
            in_quotes = !in_quotes;
        } else if !in_quotes {
            if byte == b'#' {
                return &line[..idx];
            }
            if byte == b'/' && idx + 1 < bytes.len() && bytes[idx + 1] == b'/' {
                return &line[..idx];
            }
        }
        idx += 1;
    }
    line
}
