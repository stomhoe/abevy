#[allow(unused_imports)]
use bevy::prelude::*;
use bevy::platform::collections::{HashMap, HashSet};
use common::def_db::discover_assets_files_by_suffixes;
use common::log_targets::TERRPROBE_INIT;

use crate::terrain::terrprobe::terrprobe_seris::TerrainProbeSeri;

#[derive(Clone)]
pub struct LoadedTerrainProbeDef {
    pub rel_path: String,
    pub is_abstract: bool,
    pub seri: TerrainProbeSeri,
}

#[derive(Clone)]
struct RawTerrainProbeDef {
    id: String,
    rel_path: String,
    base_id: Option<String>,
    is_abstract: bool,
    fields: RawTerrainProbeFields,
}

#[derive(Clone, Default)]
struct RawTerrainProbeFields {
    opfilter_id: Option<String>,
    opfilter_tags: Option<HashSet<String>>,
    opfilter_var_name: Option<String>,
    opfilter_min_val: Option<f32>,
    opfilter_max_val: Option<f32>,
    structuregen_whitelist: Option<HashSet<String>>,
    structuregen_blacklist: Option<HashSet<String>>,
    required_tile_tags: Option<HashSet<String>>,
    probe_pattern: Option<String>,
    concentric_sample_spacing: Option<f32>,
    step_size: Option<u16>,
    region_multiplier: Option<f32>,
    max_batches: Option<u16>,
    iterations_per_batch: Option<u16>,
    max_emitted_results: Option<u32>,
    min_result_distance: Option<u16>,
    collect: Option<bool>,
}

pub fn load_terrain_probe_defs_from_filesystem() -> Vec<LoadedTerrainProbeDef> {
    let Ok(mut files) = discover_assets_files_by_suffixes(&[".tpt.ron"]) else {
        error!(target: TERRPROBE_INIT, "Failed discovering .tpt.ron files");
        return Vec::new();
    };
    files.sort_by(|(a, _), (b, _)| {
        a.precedence_rank()
            .cmp(&b.precedence_rank())
            .then_with(|| a.rel_path.cmp(&b.rel_path))
    });

    let mut raw_by_id = HashMap::<String, RawTerrainProbeDef>::default();
    for (source, path) in files {
        let Ok(content) = std::fs::read_to_string(&path) else {
            warn!(target: TERRPROBE_INIT, "Failed reading terrain probe file '{}'", path.to_string_lossy());
            continue;
        };
        let parsed = match parse_terrain_probe_file(&content, &source.rel_path) {
            Ok(parsed) => parsed,
            Err(err) => {
                warn!(target: TERRPROBE_INIT, "Failed parsing terrain probe defs in '{}': {}", source.rel_path, err);
                continue;
            }
        };

        if let Some(prev) = raw_by_id.insert(parsed.id.clone(), parsed) {
            debug!(target: TERRPROBE_INIT, "Terrain probe '{}' overridden: '{}' -> '{}'", prev.id, prev.rel_path, source.rel_path);
        }
    }

    let mut resolved = HashMap::<String, TerrainProbeSeri>::default();
    let mut resolving = HashSet::<String>::default();
    let mut out = Vec::with_capacity(raw_by_id.len());

    let mut ids = Vec::with_capacity(raw_by_id.len());
    ids.extend(raw_by_id.keys().cloned());
    ids.sort();

    for id in ids {
        let Some(raw) = raw_by_id.get(&id) else {
            continue;
        };
        let seri = match resolve_terrain_probe_seri(&id, &raw_by_id, &mut resolved, &mut resolving) {
            Ok(seri) => seri,
            Err(err) => {
                error!(target: TERRPROBE_INIT, "Failed resolving terrain probe '{}': {}", id, err);
                continue;
            }
        };
        out.push(LoadedTerrainProbeDef {
            rel_path: raw.rel_path.clone(),
            is_abstract: raw.is_abstract,
            seri,
        });
    }

    if out.is_empty() {
        error!(target: TERRPROBE_INIT, "No terrain probe defs loaded from filesystem");
    } else {
        trace!(target: TERRPROBE_INIT, "Loaded {} terrain probe defs from filesystem", out.len());
    }

    out
}

fn resolve_terrain_probe_seri(
    id: &str,
    raw_by_id: &HashMap<String, RawTerrainProbeDef>,
    resolved: &mut HashMap<String, TerrainProbeSeri>,
    resolving: &mut HashSet<String>,
) -> Result<TerrainProbeSeri, String> {
    if let Some(seri) = resolved.get(id) {
        return Ok(seri.clone());
    }
    if !resolving.insert(id.to_string()) {
        return Err(format!("Terrain probe '{}' has a cyclic inheritance chain", id));
    }

    let Some(raw) = raw_by_id.get(id) else {
        resolving.remove(id);
        return Err(format!("Terrain probe '{}' not found while resolving inheritance", id));
    };

    let mut seri = if let Some(base_id) = &raw.base_id {
        resolve_terrain_probe_seri(base_id, raw_by_id, resolved, resolving)?
    } else {
        TerrainProbeSeri::default()
    };
    seri.id = raw.id.clone();
    apply_raw_terrain_probe_fields(&mut seri, &raw.fields);

    if seri.id.trim().is_empty() {
        resolving.remove(id);
        return Err(format!("Terrain probe '{}' resolved to an empty id", id));
    }
    if seri.probe_pattern.trim().is_empty() {
        resolving.remove(id);
        return Err(format!("Terrain probe '{}' does not define probe_pattern", id));
    }

    resolving.remove(id);
    resolved.insert(id.to_string(), seri.clone());
    Ok(seri)
}

fn apply_raw_terrain_probe_fields(seri: &mut TerrainProbeSeri, fields: &RawTerrainProbeFields) {
    if let Some(opfilter_id) = &fields.opfilter_id {
        seri.opfilter_id = opfilter_id.clone();
    }
    if let Some(opfilter_tags) = &fields.opfilter_tags {
        seri.opfilter_tags = opfilter_tags.clone();
    }
    if let Some(opfilter_var_name) = &fields.opfilter_var_name {
        seri.opfilter_var_name = opfilter_var_name.clone();
    }
    if let Some(opfilter_min_val) = fields.opfilter_min_val {
        seri.opfilter_min_val = opfilter_min_val;
    }
    if let Some(opfilter_max_val) = fields.opfilter_max_val {
        seri.opfilter_max_val = opfilter_max_val;
    }
    if let Some(structuregen_whitelist) = &fields.structuregen_whitelist {
        seri.structuregen_whitelist = structuregen_whitelist.clone();
    }
    if let Some(structuregen_blacklist) = &fields.structuregen_blacklist {
        seri.structuregen_blacklist = structuregen_blacklist.clone();
    }
    if let Some(required_tile_tags) = &fields.required_tile_tags {
        seri.required_tile_tags = required_tile_tags.clone();
    }
    if let Some(probe_pattern) = &fields.probe_pattern {
        seri.probe_pattern = probe_pattern.clone();
    }
    if let Some(concentric_sample_spacing) = fields.concentric_sample_spacing {
        seri.concentric_sample_spacing = concentric_sample_spacing;
    }
    if let Some(step_size) = fields.step_size {
        seri.step_size = step_size;
    }
    if let Some(region_multiplier) = fields.region_multiplier {
        seri.region_multiplier = region_multiplier;
    }
    if let Some(max_batches) = fields.max_batches {
        seri.max_batches = max_batches;
    }
    if let Some(iterations_per_batch) = fields.iterations_per_batch {
        seri.iterations_per_batch = iterations_per_batch;
    }
    if let Some(max_emitted_results) = fields.max_emitted_results {
        seri.max_emitted_results = max_emitted_results;
    }
    if let Some(min_result_distance) = fields.min_result_distance {
        seri.min_result_distance = min_result_distance;
    }
    if let Some(collect) = fields.collect {
        seri.collect = collect;
    }
}

fn parse_terrain_probe_file(content: &str, rel_path: &str) -> Result<RawTerrainProbeDef, String> {
    let mut id = None::<String>;
    let mut base_id = None::<String>;
    let mut is_abstract = false;
    let mut fields = RawTerrainProbeFields::default();

    for (idx, raw_line) in content.lines().enumerate() {
        let line_no = idx + 1;
        let line_without_comment = strip_inline_comment(raw_line);
        let trimmed = line_without_comment.trim();
        if trimmed.is_empty() {
            continue;
        }

        let (key, raw_value) = split_field_line(trimmed, rel_path, line_no)?;
        let key = key.to_ascii_lowercase();
        match key.as_str() {
            "id" => {
                let parsed = parse_text_value(&raw_value);
                if parsed.is_empty() {
                    return Err(format!("{rel_path}:{line_no} has an empty terrain probe id"));
                }
                id = Some(parsed);
            }
            "extends" | "base" => {
                let parsed = parse_text_value(&raw_value);
                if parsed.is_empty() {
                    return Err(format!("{rel_path}:{line_no} has an empty base id"));
                }
                base_id = Some(parsed);
            }
            "abstract" => {
                is_abstract = parse_abstract_value(&raw_value, rel_path, line_no)?;
            }
            "opfilter_id" => {
                fields.opfilter_id = Some(parse_text_value(&raw_value));
            }
            "opfilter_tags" => {
                fields.opfilter_tags = Some(parse_string_set(&raw_value));
            }
            "opfilter_var_name" => {
                fields.opfilter_var_name = Some(parse_text_value(&raw_value));
            }
            "opfilter_min_val" => {
                fields.opfilter_min_val = Some(parse_f32_value(&raw_value, rel_path, line_no)?);
            }
            "opfilter_max_val" => {
                fields.opfilter_max_val = Some(parse_f32_value(&raw_value, rel_path, line_no)?);
            }
            "structuregen_whitelist" => {
                fields.structuregen_whitelist = Some(parse_string_set(&raw_value));
            }
            "structuregen_blacklist" => {
                fields.structuregen_blacklist = Some(parse_string_set(&raw_value));
            }
            "required_tile_tags" => {
                fields.required_tile_tags = Some(parse_string_set(&raw_value));
            }
            "probe_pattern" => {
                fields.probe_pattern = Some(parse_text_value(&raw_value));
            }
            "concentric_sample_spacing" | "conc_sample_spacing" => {
                fields.concentric_sample_spacing = Some(parse_f32_value(&raw_value, rel_path, line_no)?);
            }
            "step_size" => {
                fields.step_size = Some(parse_u16_value(&raw_value, rel_path, line_no)?);
            }
            "region_multiplier" => {
                fields.region_multiplier = Some(parse_f32_value(&raw_value, rel_path, line_no)?);
            }
            "max_batches" => {
                fields.max_batches = Some(parse_u16_value(&raw_value, rel_path, line_no)?);
            }
            "iterations_per_batch" => {
                fields.iterations_per_batch = Some(parse_u16_value(&raw_value, rel_path, line_no)?);
            }
            "max_emitted_results" => {
                fields.max_emitted_results = Some(parse_u32_value(&raw_value, rel_path, line_no)?);
            }
            "min_result_distance" => {
                fields.min_result_distance = Some(parse_u16_value(&raw_value, rel_path, line_no)?);
            }
            "collect" => {
                fields.collect = Some(parse_bool_value(&raw_value, rel_path, line_no)?);
            }
            other => {
                return Err(format!("{rel_path}:{line_no} has unknown terrain probe field '{other}'"));
            }
        }
    }

    let id = id.unwrap_or_else(|| id_from_rel_path(rel_path));
    if id.trim().is_empty() {
        return Err(format!("{rel_path} has an empty id and fallback file stem is also empty"));
    }

    Ok(RawTerrainProbeDef {
        id,
        rel_path: rel_path.to_string(),
        base_id,
        is_abstract,
        fields,
    })
}

fn split_field_line(trimmed: &str, rel_path: &str, line_no: usize) -> Result<(String, String), String> {
    if let Some((key, raw_value)) = trimmed.split_once(':') {
        let key = key.trim();
        if key.is_empty() {
            return Err(format!("{rel_path}:{line_no} has an empty field name"));
        }
        return Ok((key.to_string(), raw_value.trim().to_string()));
    }

    let mut parts = trimmed.splitn(2, char::is_whitespace);
    let Some(key) = parts.next() else {
        return Err(format!("{rel_path}:{line_no} has an empty field line"));
    };
    if key.trim().is_empty() {
        return Err(format!("{rel_path}:{line_no} has an empty field name"));
    }
    let raw_value = parts.next().unwrap_or_default().trim().to_string();
    Ok((key.trim().to_string(), raw_value))
}

fn parse_string_set(raw: &str) -> HashSet<String> {
    let mut out = HashSet::default();
    for token in raw.split(|c: char| c.is_whitespace() || c == ',') {
        let token = parse_text_value(token);
        if token.is_empty() {
            continue;
        }
        out.insert(token);
    }
    out
}

fn parse_abstract_value(raw: &str, rel_path: &str, line_no: usize) -> Result<bool, String> {
    let raw = raw.trim();
    if raw.is_empty() {
        return Ok(true);
    }
    parse_bool_token(raw, rel_path, line_no)
}

fn parse_bool_value(raw: &str, rel_path: &str, line_no: usize) -> Result<bool, String> {
    parse_bool_token(raw.trim(), rel_path, line_no)
}

fn parse_bool_token(raw: &str, rel_path: &str, line_no: usize) -> Result<bool, String> {
    match parse_text_value(raw).to_ascii_lowercase().as_str() {
        "true" | "yes" | "on" => Ok(true),
        "false" | "no" | "off" => Ok(false),
        other => Err(format!("{rel_path}:{line_no} has invalid bool value '{other}'")),
    }
}

fn parse_u16_value(raw: &str, rel_path: &str, line_no: usize) -> Result<u16, String> {
    parse_text_value(raw)
        .parse::<u16>()
        .map_err(|err| format!("{rel_path}:{line_no} has invalid u16 value: {err}"))
}

fn parse_u32_value(raw: &str, rel_path: &str, line_no: usize) -> Result<u32, String> {
    parse_text_value(raw)
        .parse::<u32>()
        .map_err(|err| format!("{rel_path}:{line_no} has invalid u32 value: {err}"))
}

fn parse_f32_value(raw: &str, rel_path: &str, line_no: usize) -> Result<f32, String> {
    parse_text_value(raw)
        .parse::<f32>()
        .map_err(|err| format!("{rel_path}:{line_no} has invalid f32 value: {err}"))
}

fn strip_inline_comment(line: &str) -> &str {
    let hash_idx = line.find('#');
    let slash_idx = line.find("//");
    match (hash_idx, slash_idx) {
        (Some(hash_idx), Some(slash_idx)) => &line[..hash_idx.min(slash_idx)],
        (Some(hash_idx), None) => &line[..hash_idx],
        (None, Some(slash_idx)) => &line[..slash_idx],
        (None, None) => line,
    }
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

fn id_from_rel_path(rel_path: &str) -> String {
    let file_name = rel_path.rsplit('/').next().unwrap_or_default();
    file_name
        .strip_suffix(".tpt.ron")
        .unwrap_or(file_name)
        .to_string()
}
