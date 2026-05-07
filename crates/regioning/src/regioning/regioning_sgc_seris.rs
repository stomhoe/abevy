use std::path::Path;

use bevy::{platform::collections::*, prelude::*};
use common::def_db;
pub use tilemap_shared::{SgcArgValue, SgcArgsDict};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct SgcSeri {
    pub id: String,
    pub structure_id: String,
    pub tags: Vec<String>,
    pub args: SgcArgsDict,
    pub disabled: bool,
    pub weight: f32,
    pub priority: f32,
    pub pdisk_mindist_and_tag: Vec<(Option<u8>, String)>,
    pub min_dists_from_other_structures: HashMap<String, u8>,
    pub exclusive_for_dimensions: Vec<String>,
    pub run_before_sgcs_with_tags: HashSet<String>,
    pub run_after_sgcs_with_tags: HashSet<String>,
    pub whitelisted_tags: HashSet<String>,
    pub blacklisted_tags: HashSet<String>,
    pub max_per_region: u32,
    pub max_being_count: Option<u32>,
}

fn default_max_per_region() -> u32 {
    1024
}

fn default_weight() -> f32 {
    f32::NEG_INFINITY
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum SgcTokenKind {
    Ident(String),
    String(String),
    Number(String),
    Bool(bool),
    Null,
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    Eq,
    Colon,
    Comma,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SgcToken {
    kind: SgcTokenKind,
    line: usize,
    column: usize,
}

struct SgcTokenizer<'a> {
    chars: std::iter::Peekable<std::str::CharIndices<'a>>,
    source: &'a str,
    line: usize,
    column: usize,
}

impl<'a> SgcTokenizer<'a> {
    fn new(source: &'a str) -> Self {
        Self {
            chars: source.char_indices().peekable(),
            source,
            line: 1,
            column: 1,
        }
    }

    fn tokenize(mut self) -> Result<Vec<SgcToken>, String> {
        let mut tokens = Vec::new();
        while let Some((idx, ch)) = self.peek_char() {
            if ch.is_whitespace() {
                self.consume_char();
                continue;
            }
            if ch == '#' {
                self.consume_line_comment();
                continue;
            }
            if ch == '/' && self.peek_next_is('/') {
                self.consume_char();
                self.consume_char();
                self.consume_line_comment();
                continue;
            }
            let line = self.line;
            let column = self.column;
            let kind = match ch {
                '{' => {
                    self.consume_char();
                    SgcTokenKind::LBrace
                }
                '}' => {
                    self.consume_char();
                    SgcTokenKind::RBrace
                }
                '[' => {
                    self.consume_char();
                    SgcTokenKind::LBracket
                }
                ']' => {
                    self.consume_char();
                    SgcTokenKind::RBracket
                }
                '=' => {
                    self.consume_char();
                    SgcTokenKind::Eq
                }
                ':' => {
                    self.consume_char();
                    SgcTokenKind::Colon
                }
                ',' => {
                    self.consume_char();
                    SgcTokenKind::Comma
                }
                '"' => {
                    let parsed = self.parse_string()?;
                    SgcTokenKind::String(parsed)
                }
                '-' | '0'..='9' => {
                    let parsed = self.parse_number();
                    SgcTokenKind::Number(parsed)
                }
                _ if is_ident_start(ch) => {
                    let parsed = self.parse_ident();
                    match parsed.as_str() {
                        "true" => SgcTokenKind::Bool(true),
                        "false" => SgcTokenKind::Bool(false),
                        "null" => SgcTokenKind::Null,
                        _ => SgcTokenKind::Ident(parsed),
                    }
                }
                _ => {
                    return Err(format!(
                        "Unexpected character '{}' at {}:{}",
                        ch, line, column
                    ));
                }
            };
            tokens.push(SgcToken { kind, line, column });
            if idx >= self.source.len() {
                break;
            }
        }
        Ok(tokens)
    }

    fn peek_char(&mut self) -> Option<(usize, char)> {
        self.chars.peek().copied()
    }

    fn peek_next_is(&mut self, expected: char) -> bool {
        let Some((idx, _)) = self.peek_char() else {
            return false;
        };
        self.source[idx..].chars().nth(1) == Some(expected)
    }

    fn consume_char(&mut self) -> Option<char> {
        let (_, ch) = self.chars.next()?;
        if ch == '\n' {
            self.line = self.line.saturating_add(1);
            self.column = 1;
        } else {
            self.column = self.column.saturating_add(1);
        }
        Some(ch)
    }

    fn consume_line_comment(&mut self) {
        while let Some(ch) = self.consume_char() {
            if ch == '\n' {
                break;
            }
        }
    }

    fn parse_ident(&mut self) -> String {
        let mut out = String::new();
        while let Some((_, ch)) = self.peek_char() {
            if !is_ident_continue(ch) {
                break;
            }
            let Some(ch) = self.consume_char() else {
                break;
            };
            out.push(ch);
        }
        out
    }

    fn parse_number(&mut self) -> String {
        let mut out = String::new();
        while let Some((_, ch)) = self.peek_char() {
            if !(ch.is_ascii_digit() || matches!(ch, '-' | '+' | '.' | 'e' | 'E')) {
                break;
            }
            let Some(ch) = self.consume_char() else {
                break;
            };
            out.push(ch);
        }
        out
    }

    fn parse_string(&mut self) -> Result<String, String> {
        let start_line = self.line;
        let start_col = self.column;
        let Some(open) = self.consume_char() else {
            return Err(format!("Unexpected EOF while parsing string at {}:{}", start_line, start_col));
        };
        if open != '"' {
            return Err(format!("Expected '\"' at {}:{}", start_line, start_col));
        }
        let mut out = String::new();
        while let Some(ch) = self.consume_char() {
            if ch == '"' {
                return Ok(out);
            }
            if ch == '\\' {
                let Some(escaped) = self.consume_char() else {
                    return Err(format!("Unterminated escape sequence at {}:{}", self.line, self.column));
                };
                let value = match escaped {
                    'n' => '\n',
                    'r' => '\r',
                    't' => '\t',
                    '\\' => '\\',
                    '"' => '"',
                    _ => escaped,
                };
                out.push(value);
                continue;
            }
            out.push(ch);
        }
        Err(format!(
            "Unterminated string literal started at {}:{}",
            start_line, start_col
        ))
    }
}

fn is_ident_start(ch: char) -> bool {
    ch.is_ascii_alphabetic() || matches!(ch, '_' | '*' | '.')
}

fn is_ident_continue(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-' | '.' | '*' | '/')
}

struct SgcParser {
    tokens: Vec<SgcToken>,
    cursor: usize,
}

impl SgcParser {
    fn new(tokens: Vec<SgcToken>) -> Self {
        Self { tokens, cursor: 0 }
    }

    fn parse_root(&mut self) -> Result<HashMap<String, SgcArgValue>, String> {
        let mut inferred_id: Option<String> = None;
        if self.peek_ident_is("sgc") {
            self.cursor += 1;
            if let Some(id) = self.consume_ident() {
                inferred_id = Some(id);
            }
        }
        let mut map = self.parse_object()?;
        if let Some(inferred_id) = inferred_id
            && !map.contains_key("id")
        {
            map.insert("id".to_string(), SgcArgValue::Str(inferred_id));
        }
        Ok(map)
    }

    fn parse_object(&mut self) -> Result<HashMap<String, SgcArgValue>, String> {
        self.expect_kind(SgcTokenKind::LBrace)?;
        let mut map = HashMap::default();
        while !self.peek_kind(&SgcTokenKind::RBrace) {
            let Some(key) = self.consume_key() else {
                return Err(self.error_here("Expected key in object"));
            };
            let value = if self.peek_kind(&SgcTokenKind::LBrace) {
                SgcArgValue::Map(self.parse_object()?)
            } else {
                self.expect_eq_or_colon()?;
                self.parse_value()?
            };
            map.insert(key, value);
            self.consume_if_kind(&SgcTokenKind::Comma);
        }
        self.expect_kind(SgcTokenKind::RBrace)?;
        Ok(map)
    }

    fn parse_list(&mut self) -> Result<Vec<SgcArgValue>, String> {
        self.expect_kind(SgcTokenKind::LBracket)?;
        let mut values = Vec::new();
        while !self.peek_kind(&SgcTokenKind::RBracket) {
            values.push(self.parse_value()?);
            if !self.consume_if_kind(&SgcTokenKind::Comma) {
                break;
            }
        }
        self.expect_kind(SgcTokenKind::RBracket)?;
        Ok(values)
    }

    fn parse_value(&mut self) -> Result<SgcArgValue, String> {
        let Some(token) = self.tokens.get(self.cursor) else {
            return Err(self.error_here("Unexpected EOF while parsing value"));
        };
        match &token.kind {
            SgcTokenKind::String(value) => {
                self.cursor += 1;
                Ok(SgcArgValue::Str(value.clone()))
            }
            SgcTokenKind::Ident(value) => {
                self.cursor += 1;
                Ok(SgcArgValue::Str(value.clone()))
            }
            SgcTokenKind::Bool(value) => {
                self.cursor += 1;
                Ok(SgcArgValue::Bool(*value))
            }
            SgcTokenKind::Null => {
                self.cursor += 1;
                Ok(SgcArgValue::Null)
            }
            SgcTokenKind::Number(value) => {
                self.cursor += 1;
                if value.contains('.') || value.contains('e') || value.contains('E') {
                    let parsed = value.parse::<f64>().map_err(|_| {
                        self.error_here(format!("Invalid float literal '{}'", value).as_str())
                    })?;
                    return Ok(SgcArgValue::Float(parsed));
                }
                let parsed = value.parse::<i64>().map_err(|_| {
                    self.error_here(format!("Invalid int literal '{}'", value).as_str())
                })?;
                Ok(SgcArgValue::Int(parsed))
            }
            SgcTokenKind::LBracket => Ok(SgcArgValue::List(self.parse_list()?)),
            SgcTokenKind::LBrace => Ok(SgcArgValue::Map(self.parse_object()?)),
            _ => Err(self.error_here("Expected value")),
        }
    }

    fn consume_if_kind(&mut self, expected: &SgcTokenKind) -> bool {
        if !self.peek_kind(expected) {
            return false;
        }
        self.cursor += 1;
        true
    }

    fn peek_kind(&self, expected: &SgcTokenKind) -> bool {
        let Some(token) = self.tokens.get(self.cursor) else {
            return false;
        };
        &token.kind == expected
    }

    fn expect_eq_or_colon(&mut self) -> Result<(), String> {
        if self.consume_if_kind(&SgcTokenKind::Eq) || self.consume_if_kind(&SgcTokenKind::Colon) {
            return Ok(());
        }
        Err(self.error_here("Expected '=' or ':'"))
    }

    fn consume_ident(&mut self) -> Option<String> {
        let token = self.tokens.get(self.cursor)?;
        let SgcTokenKind::Ident(value) = &token.kind else {
            return None;
        };
        self.cursor += 1;
        Some(value.clone())
    }

    fn consume_key(&mut self) -> Option<String> {
        let token = self.tokens.get(self.cursor)?;
        match &token.kind {
            SgcTokenKind::Ident(value) => {
                self.cursor += 1;
                Some(value.clone())
            }
            SgcTokenKind::String(value) => {
                self.cursor += 1;
                Some(value.clone())
            }
            _ => None,
        }
    }

    fn expect_kind(&mut self, expected: SgcTokenKind) -> Result<(), String> {
        let Some(token) = self.tokens.get(self.cursor) else {
            return Err(self.error_here("Unexpected EOF"));
        };
        if token.kind == expected {
            self.cursor += 1;
            return Ok(());
        }
        Err(format!(
            "Expected {:?} at {}:{}, got {:?}",
            expected, token.line, token.column, token.kind
        ))
    }

    fn peek_ident_is(&self, expected: &str) -> bool {
        let Some(token) = self.tokens.get(self.cursor) else {
            return false;
        };
        let SgcTokenKind::Ident(value) = &token.kind else {
            return false;
        };
        value == expected
    }

    fn error_here(&self, message: &str) -> String {
        let Some(token) = self.tokens.get(self.cursor) else {
            return format!("{} at EOF", message);
        };
        format!("{} at {}:{}", message, token.line, token.column)
    }
}

fn parse_sgc_seri(content: &str, _path: &Path) -> Result<SgcSeri, String> {
    if content.trim().is_empty() {
        return Err("File is empty".to_string());
    }
    let tokens = SgcTokenizer::new(content).tokenize()?;
    if tokens.is_empty() {
        return Err("File contains no parseable tokens".to_string());
    }
    let mut parser = SgcParser::new(tokens);
    let mut fields = parser.parse_root()?;

    let id = take_required_string(&mut fields, "id")?;
    let structure_id = take_required_string(&mut fields, "structure_id")?;
    let tags = take_string_list(&mut fields, "tags");
    let args = take_args_dict(&mut fields, "args");
    let disabled = take_bool(&mut fields, "disabled").unwrap_or(false);
    let weight = take_f32(&mut fields, "weight").unwrap_or_else(default_weight);
    let priority = take_f32(&mut fields, "priority").unwrap_or_default();
    let pdisk_mindist_and_tag = take_pdisk_pairs(&mut fields, "pdisk_mindist_and_tag");
    let min_dists_from_other_structures = take_string_u8_map(&mut fields, "min_dists_from_other_structures");
    let exclusive_for_dimensions = take_string_list(&mut fields, "exclusive_for_dimensions");
    let run_before_sgcs_with_tags = HashSet::from_iter(take_string_list(&mut fields, "run_before_sgcs_with_tags"));
    let run_after_sgcs_with_tags = HashSet::from_iter(take_string_list(&mut fields, "run_after_sgcs_with_tags"));
    let whitelisted_tags = HashSet::from_iter(take_string_list(&mut fields, "whitelisted_tags"));
    let blacklisted_tags = HashSet::from_iter(take_string_list(&mut fields, "blacklisted_tags"));
    let max_per_region = take_u32(&mut fields, "max_per_region").unwrap_or_else(default_max_per_region);
    let max_being_count = take_u32(&mut fields, "max_being_count")
        .or_else(|| take_u32(&mut fields, "max_spawn_being_count"));

    Ok(SgcSeri {
        id,
        structure_id,
        tags,
        args,
        disabled,
        weight,
        priority,
        pdisk_mindist_and_tag,
        min_dists_from_other_structures,
        exclusive_for_dimensions,
        run_before_sgcs_with_tags,
        run_after_sgcs_with_tags,
        whitelisted_tags,
        blacklisted_tags,
        max_per_region,
        max_being_count,
    })
}

fn take_required_string(fields: &mut HashMap<String, SgcArgValue>, key: &str) -> Result<String, String> {
    let Some(value) = fields.remove(key) else {
        return Err(format!("Missing required field '{}'", key));
    };
    value
        .as_scalar_string()
        .ok_or_else(|| format!("Field '{}' must be a scalar", key))
}

fn take_args_dict(fields: &mut HashMap<String, SgcArgValue>, key: &str) -> SgcArgsDict {
    let Some(value) = fields.remove(key) else {
        return SgcArgsDict::default();
    };
    let Some(map) = value.as_map() else {
        return SgcArgsDict::default();
    };
    SgcArgsDict(map.clone())
}

fn take_string_list(fields: &mut HashMap<String, SgcArgValue>, key: &str) -> Vec<String> {
    let Some(value) = fields.remove(key) else {
        return Vec::new();
    };
    match value {
        SgcArgValue::List(values) => {
            let mut out = Vec::with_capacity(values.len());
            for value in values {
                let Some(value) = value.as_scalar_string() else {
                    continue;
                };
                out.push(value);
            }
            out
        }
        scalar => scalar
            .as_scalar_string()
            .map(|value| vec![value])
            .unwrap_or_default(),
    }
}

fn take_u32(fields: &mut HashMap<String, SgcArgValue>, key: &str) -> Option<u32> {
    let value = fields.remove(key)?;
    match value {
        SgcArgValue::Int(value) => u32::try_from(value).ok(),
        SgcArgValue::Float(value) => {
            if value.is_finite() && value >= 0.0 {
                u32::try_from(value.round() as i64).ok()
            } else {
                None
            }
        }
        SgcArgValue::Str(value) => value.parse::<u32>().ok(),
        _ => None,
    }
}

fn take_bool(fields: &mut HashMap<String, SgcArgValue>, key: &str) -> Option<bool> {
    let value = fields.remove(key)?;
    match value {
        SgcArgValue::Bool(value) => Some(value),
        SgcArgValue::Int(value) => match value {
            0 => Some(false),
            1 => Some(true),
            _ => None,
        },
        SgcArgValue::Str(value) => match value.as_str() {
            "true" | "1" | "yes" | "on" => Some(true),
            "false" | "0" | "no" | "off" => Some(false),
            _ => None,
        },
        _ => None,
    }
}

fn take_f32(fields: &mut HashMap<String, SgcArgValue>, key: &str) -> Option<f32> {
    let value = fields.remove(key)?;
    match value {
        SgcArgValue::Int(value) => Some(value as f32),
        SgcArgValue::Float(value) => Some(value as f32),
        SgcArgValue::Str(value) => value.parse::<f32>().ok(),
        _ => None,
    }
}

fn take_string_u8_map(fields: &mut HashMap<String, SgcArgValue>, key: &str) -> HashMap<String, u8> {
    let Some(value) = fields.remove(key) else {
        return HashMap::default();
    };
    let Some(map) = value.as_map() else {
        return HashMap::default();
    };
    let mut out = HashMap::with_capacity(map.len());
    for (key, value) in map {
        let Some(value) = value.as_u8() else {
            continue;
        };
        out.insert(key.clone(), value);
    }
    out
}

fn take_pdisk_pairs(fields: &mut HashMap<String, SgcArgValue>, key: &str) -> Vec<(Option<u8>, String)> {
    let Some(value) = fields.remove(key) else {
        return Vec::new();
    };
    let Some(values) = value.as_list() else {
        return Vec::new();
    };
    let mut out = Vec::with_capacity(values.len());
    for value in values {
        let Some(list) = value.as_list() else {
            continue;
        };
        if list.len() != 2 {
            continue;
        }
        let min_dist = match &list[0] {
            SgcArgValue::Null => None,
            value => value.as_u8(),
        };
        let Some(tag) = list[1].as_scalar_string() else {
            continue;
        };
        out.push((min_dist, tag));
    }
    out
}

pub fn load_sgc_seri_defs() -> Vec<SgcSeri> {
    let mut discovered = match def_db::discover_assets_files_by_suffixes(&[".sgc"]) {
        Ok(discovered) => discovered,
        Err(_) => {
            return Vec::new();
        }
    };
    discovered.sort_by(|(a, _), (b, _)| {
        a.precedence_rank()
            .cmp(&b.precedence_rank())
            .then_with(|| a.rel_path.cmp(&b.rel_path))
    });
    let mut by_id: HashMap<String, SgcSeri> = HashMap::default();
    let mut id_order = Vec::new();
    let mut id_source: HashMap<String, String> = HashMap::default();

    for (source, abs_path) in discovered {
        let content = match std::fs::read_to_string(&abs_path) {
            Ok(content) => content,
        Err(_) => {
            continue;
        }
    };
        let parsed = match parse_sgc_seri(&content, &abs_path) {
            Ok(parsed) => parsed,
        Err(_) => {
            continue;
        }
    };
        if parsed.disabled {
            continue;
        }
        let id = parsed.id.clone();
        let Some(_) = by_id.insert(id.clone(), parsed) else {
            id_order.push(id.clone());
            id_source.insert(id, source.rel_path);
            continue;
        };
        id_source.insert(id, source.rel_path);
    }

    let mut out = Vec::with_capacity(id_order.len());
    for id in id_order {
        let Some(parsed) = by_id.remove(&id) else {
            continue;
        };
        out.push(parsed);
    }
    out
}

#[derive(Deserialize, Asset, TypePath, Clone, Debug)]
pub struct StructureGenerationSettingsSeri {
    #[serde(default = "default_structure_build_timeout_secs")]
    pub structure_build_timeout_secs: f64,
    #[serde(default = "default_claimlist_advance_timeout_secs")]
    pub claimlist_advance_timeout_secs: f32,
    #[serde(default = "default_region_offer_timeout_secs")]
    pub region_offer_timeout_secs: f32,
    #[serde(default = "default_max_used_chunks_per_region_ratio")]
    pub max_used_chunks_per_region_ratio: f32,
}

impl StructureGenerationSettingsSeri {
    pub fn to_structure_generation_settings(&self) -> tilemap_shared::StructureGenerationSettings {
        tilemap_shared::StructureGenerationSettings {
            structure_build_timeout_secs: self.structure_build_timeout_secs,
            claimlist_advance_timeout_secs: self.claimlist_advance_timeout_secs,
            region_offer_timeout_secs: self.region_offer_timeout_secs,
            max_used_chunks_per_region_ratio: self.max_used_chunks_per_region_ratio,
        }
    }
}

fn default_structure_build_timeout_secs() -> f64 {
    4.0
}

fn default_claimlist_advance_timeout_secs() -> f32 {
    0.1
}

fn default_region_offer_timeout_secs() -> f32 {
    2.0
}

fn default_max_used_chunks_per_region_ratio() -> f32 {
    0.07
}

pub fn load_structure_generation_settings_seri_defs() -> Vec<StructureGenerationSettingsSeri> {
    let db = match common::def_db::DefDatabase::<StructureGenerationSettingsSeri>::load_from_assets_dir_with_type(
        stringify!(StructureGenerationSettingsSeri),
        &["structure_generation.settings.ron"],
        |_| "structure_generation_settings",
    ) {
        Ok(db) => db,
        Err(err) => {
            error!(
                target: common::log_targets::TERRGEN_INIT,
                "Failed loading StructureGenerationSettingsSeri defs: {err:#}"
            );
            return Vec::new();
        }
    };
    db.into_records().into_iter().map(|r| r.value).collect()
}
