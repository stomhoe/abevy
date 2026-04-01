use std::path::Path;

use bevy::{platform::collections::{HashMap, HashSet}, prelude::*};
use common::{common_tag_components::TagSet, def_db, log_targets::BEING_TEMPLATE_INIT};
use tilemap_shared::tilemap_shared_samplers::NormalDistSeri;

use being_shared::WanderSeri;

pub type SpawnWeight = f32;
pub type RankDist = NormalDistSeri;

#[derive(Debug, Clone)]
pub struct PackMemberConfigSeri {
    pub weight: SpawnWeight,
    pub rank_dist: RankDist,
    pub min: u32,
    pub max: u32,
    pub race_first: bool,
}

impl Default for PackMemberConfigSeri {
    fn default() -> Self {
        Self {
            weight: 1.0,
            rank_dist: default_rank_dist(),
            min: 0,
            max: u32::MAX,
            race_first: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PackSeri {
    pub id: String,
    pub tags: HashSet<String>,
    pub spawn_pack_entity: bool,
    pub spawn_being_count_normal_dist: NormalDistSeri,
    pub pack_spawn_radius: u8,
    pub ids: HashMap<String, PackMemberConfigSeri>,
    pub avgpos_rank_based_weight_multiplier: f32,
    pub avgpos_rank_based_weight_multipliers: HashMap<String, f32>,
    pub biome_affinity: HashMap<String, f32>,
    pub behavior_on_member_attack: String,
    pub attack_alert_effectiveness_falloff: f32,
    pub counter_regroup_tightness: f32,
    pub wander: WanderSeri,
    pub chunk_separation_to_others: HashMap<String, u8>,//u8: inbetween chunks of separation (should work radially)
}

impl Default for PackSeri {
    fn default() -> Self {
        Self {
            id: String::default(),
            tags: HashSet::default(),
            spawn_pack_entity: true,
            spawn_being_count_normal_dist: NormalDistSeri::default(),
            pack_spawn_radius: being_shared::PackSpawnRadius::default().0,
            ids: HashMap::default(),
            avgpos_rank_based_weight_multiplier: 1.0,
            avgpos_rank_based_weight_multipliers: HashMap::default(),
            biome_affinity: HashMap::default(),
            behavior_on_member_attack: String::default(),
            attack_alert_effectiveness_falloff: 0.05,
            counter_regroup_tightness: 1.5,
            wander: WanderSeri::default(),
            chunk_separation_to_others: HashMap::default(),
        }
    }
}

impl PackSeri {
    pub fn tags_with_id(&self) -> TagSet {
        TagSet::new(self.tags.iter().chain(std::iter::once(&self.id)))
    }
}

fn default_rank_dist() -> RankDist {
    RankDist {
        min_dev: 0.0,
        max_dev: 0.0,
        mean: 1.0,
        std_dev: 0.0,
    }
}

#[derive(Debug, Clone, PartialEq)]
enum PackArgValue {
    Str(String),
    Bool(bool),
    Int(i64),
    Float(f64),
    List(Vec<PackArgValue>),
    Map(HashMap<String, PackArgValue>),
    Null,
}

impl PackArgValue {
    fn as_map(&self) -> Option<&HashMap<String, PackArgValue>> {
        let Self::Map(map) = self else {
            return None;
        };
        Some(map)
    }

    fn as_list(&self) -> Option<&[PackArgValue]> {
        let Self::List(list) = self else {
            return None;
        };
        Some(list.as_slice())
    }

    fn as_bool(&self) -> Option<bool> {
        match self {
            Self::Bool(value) => Some(*value),
            Self::Int(value) => Some(*value != 0),
            Self::Str(value) => match value.as_str() {
                "true" => Some(true),
                "false" => Some(false),
                _ => None,
            },
            _ => None,
        }
    }

    fn as_string(&self) -> Option<String> {
        match self {
            Self::Str(value) => Some(value.clone()),
            Self::Int(value) => Some(value.to_string()),
            Self::Float(value) => Some(value.to_string()),
            Self::Bool(value) => Some(value.to_string()),
            _ => None,
        }
    }

    fn as_u32(&self) -> Option<u32> {
        match self {
            Self::Int(value) => u32::try_from(*value).ok(),
            Self::Float(value) => {
                if !value.is_finite() {
                    return None;
                }
                let rounded = value.round();
                if (*value - rounded).abs() > f64::EPSILON {
                    return None;
                }
                u32::try_from(rounded as i128).ok()
            }
            Self::Str(value) => value.parse::<u32>().ok(),
            _ => None,
        }
    }

    fn as_u8(&self) -> Option<u8> {
        self.as_u32().and_then(|value| u8::try_from(value).ok())
    }

    fn as_f32(&self) -> Option<f32> {
        match self {
            Self::Int(value) => Some(*value as f32),
            Self::Float(value) => Some(*value as f32),
            Self::Str(value) => value.parse::<f32>().ok(),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum PackTokenKind {
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
struct PackToken {
    kind: PackTokenKind,
    line: usize,
    column: usize,
}

struct PackTokenizer<'a> {
    chars: std::iter::Peekable<std::str::CharIndices<'a>>,
    source: &'a str,
    line: usize,
    column: usize,
}

impl<'a> PackTokenizer<'a> {
    fn new(source: &'a str) -> Self {
        Self {
            chars: source.char_indices().peekable(),
            source,
            line: 1,
            column: 1,
        }
    }

    fn tokenize(mut self) -> Result<Vec<PackToken>, String> {
        let mut tokens = Vec::new();
        while let Some((_idx, ch)) = self.peek_char() {
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
                    PackTokenKind::LBrace
                }
                '}' => {
                    self.consume_char();
                    PackTokenKind::RBrace
                }
                '[' => {
                    self.consume_char();
                    PackTokenKind::LBracket
                }
                ']' => {
                    self.consume_char();
                    PackTokenKind::RBracket
                }
                '=' => {
                    self.consume_char();
                    PackTokenKind::Eq
                }
                ':' => {
                    self.consume_char();
                    PackTokenKind::Colon
                }
                ',' => {
                    self.consume_char();
                    PackTokenKind::Comma
                }
                '"' => {
                    let parsed = self.parse_string()?;
                    PackTokenKind::String(parsed)
                }
                '-' | '+' | '0'..='9' => {
                    let parsed = self.parse_number();
                    PackTokenKind::Number(parsed)
                }
                _ if is_ident_start(ch) => {
                    let parsed = self.parse_ident();
                    match parsed.as_str() {
                        "true" => PackTokenKind::Bool(true),
                        "false" => PackTokenKind::Bool(false),
                        "null" => PackTokenKind::Null,
                        _ => PackTokenKind::Ident(parsed),
                    }
                }
                _ => {
                    return Err(format!(
                        "Unexpected character '{}' at {}:{}",
                        ch, line, column
                    ));
                }
            };
            tokens.push(PackToken { kind, line, column });
        }
        Ok(tokens)
    }

    fn peek_char(&mut self) -> Option<(usize, char)> {
        self.chars.peek().copied()
    }

    fn peek_next_is(&self, expected: char) -> bool {
        let mut iter = self.chars.clone();
        let Some((_idx, _curr)) = iter.next() else {
            return false;
        };
        let Some((_idx, next)) = iter.next() else {
            return false;
        };
        next == expected
    }

    fn consume_char(&mut self) -> Option<(usize, char)> {
        let next = self.chars.next();
        if let Some((_idx, ch)) = next {
            if ch == '\n' {
                self.line += 1;
                self.column = 1;
            } else {
                self.column += 1;
            }
        }
        next
    }

    fn consume_line_comment(&mut self) {
        while let Some((_idx, ch)) = self.peek_char() {
            if ch == '\n' {
                break;
            }
            self.consume_char();
        }
    }

    fn parse_string(&mut self) -> Result<String, String> {
        let Some((_idx, quote)) = self.consume_char() else {
            return Err("Unexpected EOF while parsing string".to_string());
        };
        if quote != '"' {
            return Err("Expected opening quote".to_string());
        }

        let mut output = String::new();
        while let Some((_idx, ch)) = self.consume_char() {
            match ch {
                '"' => return Ok(output),
                '\\' => {
                    let Some((_idx, escaped)) = self.consume_char() else {
                        return Err("Unexpected EOF in string escape".to_string());
                    };
                    let resolved = match escaped {
                        '"' => '"',
                        '\\' => '\\',
                        'n' => '\n',
                        'r' => '\r',
                        't' => '\t',
                        other => other,
                    };
                    output.push(resolved);
                }
                _ => output.push(ch),
            }
        }
        Err("Unterminated string literal".to_string())
    }

    fn parse_number(&mut self) -> String {
        let Some((start_idx, _start_ch)) = self.peek_char() else {
            return String::new();
        };
        self.consume_char();
        while let Some((_idx, ch)) = self.peek_char() {
            if ch.is_ascii_digit() || matches!(ch, '.' | '_' | 'e' | 'E' | '+' | '-') {
                self.consume_char();
                continue;
            }
            break;
        }
        let end_idx = self
            .chars
            .peek()
            .map(|(idx, _)| *idx)
            .unwrap_or(self.source.len());
        self.source[start_idx..end_idx].to_string()
    }

    fn parse_ident(&mut self) -> String {
        let Some((start_idx, _start_ch)) = self.peek_char() else {
            return String::new();
        };
        self.consume_char();
        while let Some((_idx, ch)) = self.peek_char() {
            if is_ident_continue(ch) {
                self.consume_char();
                continue;
            }
            break;
        }
        let end_idx = self
            .chars
            .peek()
            .map(|(idx, _)| *idx)
            .unwrap_or(self.source.len());
        self.source[start_idx..end_idx].to_string()
    }
}

fn is_ident_start(ch: char) -> bool {
    ch.is_ascii_alphabetic() || ch == '_' || ch == '$'
}

fn is_ident_continue(ch: char) -> bool {
    is_ident_start(ch) || ch.is_ascii_digit() || ch == '-' || ch == '.'
}

struct PackParser {
    tokens: Vec<PackToken>,
    pos: usize,
}

impl PackParser {
    fn new(tokens: Vec<PackToken>) -> Self {
        Self { tokens, pos: 0 }
    }

    fn parse_root(&mut self) -> Result<HashMap<String, PackArgValue>, String> {
        if self.peek_ident_is("pack") {
            self.pos += 1;
        }
        if self.peek_kind(&PackTokenKind::LBrace) {
            return self.parse_object();
        }

        let mut map = HashMap::default();
        while self.pos < self.tokens.len() {
            let key = self.expect_key()?;
            self.expect_separator()?;
            let value = self.parse_value()?;
            map.insert(key, value);
            if !self.consume_if_kind(&PackTokenKind::Comma) && self.peek_kind(&PackTokenKind::RBrace) {
                break;
            }
        }
        Ok(map)
    }

    fn parse_object(&mut self) -> Result<HashMap<String, PackArgValue>, String> {
        self.expect_kind(PackTokenKind::LBrace)?;
        let mut map = HashMap::default();
        while !self.peek_kind(&PackTokenKind::RBrace) {
            let key = self.expect_key()?;
            self.expect_separator()?;
            let value = self.parse_value()?;
            map.insert(key, value);
            self.consume_if_kind(&PackTokenKind::Comma);
        }
        self.expect_kind(PackTokenKind::RBrace)?;
        Ok(map)
    }

    fn parse_list(&mut self) -> Result<Vec<PackArgValue>, String> {
        self.expect_kind(PackTokenKind::LBracket)?;
        let mut out = Vec::new();
        while !self.peek_kind(&PackTokenKind::RBracket) {
            out.push(self.parse_value()?);
            if !self.consume_if_kind(&PackTokenKind::Comma) {
                break;
            }
        }
        self.expect_kind(PackTokenKind::RBracket)?;
        Ok(out)
    }

    fn parse_value(&mut self) -> Result<PackArgValue, String> {
        let Some(token) = self.tokens.get(self.pos) else {
            return Err("Unexpected EOF while parsing value".to_string());
        };
        match &token.kind {
            PackTokenKind::String(value) => {
                self.pos += 1;
                Ok(PackArgValue::Str(value.clone()))
            }
            PackTokenKind::Ident(value) => {
                self.pos += 1;
                Ok(PackArgValue::Str(value.clone()))
            }
            PackTokenKind::Bool(value) => {
                self.pos += 1;
                Ok(PackArgValue::Bool(*value))
            }
            PackTokenKind::Null => {
                self.pos += 1;
                Ok(PackArgValue::Null)
            }
            PackTokenKind::Number(value) => {
                self.pos += 1;
                let cleaned = value.replace('_', "");
                if cleaned.contains('.') || cleaned.contains('e') || cleaned.contains('E') {
                    let parsed = cleaned.parse::<f64>().map_err(|_| {
                        format!("Invalid float '{}' at {}:{}", value, token.line, token.column)
                    })?;
                    return Ok(PackArgValue::Float(parsed));
                }
                let parsed = cleaned.parse::<i64>().map_err(|_| {
                    format!("Invalid integer '{}' at {}:{}", value, token.line, token.column)
                })?;
                Ok(PackArgValue::Int(parsed))
            }
            PackTokenKind::LBracket => Ok(PackArgValue::List(self.parse_list()?)),
            PackTokenKind::LBrace => Ok(PackArgValue::Map(self.parse_object()?)),
            other => Err(format!(
                "Unexpected token {:?} while parsing value at {}:{}",
                other, token.line, token.column
            )),
        }
    }

    fn expect_key(&mut self) -> Result<String, String> {
        let Some(token) = self.tokens.get(self.pos) else {
            return Err("Unexpected EOF while parsing key".to_string());
        };
        match &token.kind {
            PackTokenKind::Ident(value) => {
                self.pos += 1;
                Ok(value.clone())
            }
            PackTokenKind::String(value) => {
                self.pos += 1;
                Ok(value.clone())
            }
            _ => Err(format!(
                "Expected identifier/string key at {}:{}, found {:?}",
                token.line, token.column, token.kind
            )),
        }
    }

    fn expect_separator(&mut self) -> Result<(), String> {
        if self.consume_if_kind(&PackTokenKind::Eq) || self.consume_if_kind(&PackTokenKind::Colon) {
            return Ok(());
        }
        let Some(token) = self.tokens.get(self.pos) else {
            return Err("Unexpected EOF while parsing separator".to_string());
        };
        Err(format!(
            "Expected ':' or '=' at {}:{}, found {:?}",
            token.line, token.column, token.kind
        ))
    }

    fn consume_if_kind(&mut self, expected: &PackTokenKind) -> bool {
        if self.peek_kind(expected) {
            self.pos += 1;
            return true;
        }
        false
    }

    fn peek_kind(&self, expected: &PackTokenKind) -> bool {
        let Some(token) = self.tokens.get(self.pos) else {
            return false;
        };
        std::mem::discriminant(&token.kind) == std::mem::discriminant(expected)
    }

    fn peek_ident_is(&self, expected: &str) -> bool {
        let Some(token) = self.tokens.get(self.pos) else {
            return false;
        };
        let PackTokenKind::Ident(value) = &token.kind else {
            return false;
        };
        value == expected
    }

    fn expect_kind(&mut self, expected: PackTokenKind) -> Result<(), String> {
        if self.peek_kind(&expected) {
            self.pos += 1;
            return Ok(());
        }
        let Some(token) = self.tokens.get(self.pos) else {
            return Err("Unexpected EOF while parsing token".to_string());
        };
        Err(format!(
            "Expected {:?} at {}:{}, found {:?}",
            expected, token.line, token.column, token.kind
        ))
    }
}

fn parse_pack_seri(content: &str, path: &Path) -> Result<PackSeri, String> {
    let tokens = PackTokenizer::new(content).tokenize()?;
    let mut parser = PackParser::new(tokens);
    let mut fields = parser.parse_root()?;

    let mut seri = PackSeri::default();
    seri.id = take_required_string(&mut fields, "id")?;
    seri.tags = take_string_set(&mut fields, "tags");
    if let Some(value) = fields.remove("spawn_pack_entity").and_then(|value| value.as_bool()) {
        seri.spawn_pack_entity = value;
    }
    seri.spawn_being_count_normal_dist = take_normal_dist(
        &mut fields,
        "spawn_being_count_normal_dist",
        seri.spawn_being_count_normal_dist.clone(),
    );
    if let Some(value) = fields.remove("pack_spawn_radius").and_then(|value| value.as_u8()) {
        seri.pack_spawn_radius = value;
    }
    seri.ids = take_ids_map(&mut fields, "ids");
    seri.avgpos_rank_based_weight_multiplier = fields
        .remove("avgpos_rank_based_weight_multiplier")
        .and_then(|value| value.as_f32())
        .unwrap_or(seri.avgpos_rank_based_weight_multiplier);
    seri.avgpos_rank_based_weight_multipliers =
        take_string_f32_map(&mut fields, "avgpos_rank_based_weight_multipliers");
    seri.biome_affinity = take_string_f32_map(&mut fields, "biome_affinity");
    seri.behavior_on_member_attack = fields
        .remove("behavior_on_member_attack")
        .and_then(|value| value.as_string())
        .unwrap_or_default();
    seri.attack_alert_effectiveness_falloff = fields
        .remove("attack_alert_effectiveness_falloff")
        .and_then(|value| value.as_f32())
        .unwrap_or(seri.attack_alert_effectiveness_falloff);
    seri.counter_regroup_tightness = fields
        .remove("counter_regroup_tightness")
        .and_then(|value| value.as_f32())
        .unwrap_or(seri.counter_regroup_tightness);
    seri.chunk_separation_to_others = take_string_u8_map(&mut fields, "chunk_separation_to_others");
    seri.wander = take_wander_seri(&mut fields, "wander", seri.wander.clone());

    if !fields.is_empty() {
        trace!(
            target: BEING_TEMPLATE_INIT,
            "Pack '{}' ignored unknown fields in {}: {}",
            seri.id,
            path.display(),
            fields.keys().cloned().collect::<Vec<_>>().join(","),
        );
    }
    Ok(seri)
}

fn take_required_string(
    fields: &mut HashMap<String, PackArgValue>,
    key: &str,
) -> Result<String, String> {
    let Some(value) = fields.remove(key) else {
        return Err(format!("Missing required field '{}'", key));
    };
    value
        .as_string()
        .ok_or_else(|| format!("Field '{}' must be string-like", key))
}

fn take_string_set(fields: &mut HashMap<String, PackArgValue>, key: &str) -> HashSet<String> {
    let Some(value) = fields.remove(key) else {
        return HashSet::default();
    };
    let Some(list) = value.as_list() else {
        return HashSet::default();
    };
    let mut out = HashSet::with_capacity(list.len());
    for item in list {
        let Some(item) = item.as_string() else {
            continue;
        };
        if item.trim().is_empty() {
            continue;
        }
        out.insert(item);
    }
    out
}

fn take_string_f32_map(
    fields: &mut HashMap<String, PackArgValue>,
    key: &str,
) -> HashMap<String, f32> {
    let Some(value) = fields.remove(key) else {
        return HashMap::default();
    };
    let Some(map) = value.as_map() else {
        return HashMap::default();
    };
    let mut out = HashMap::with_capacity(map.len());
    for (entry_key, entry_value) in map {
        let Some(num) = entry_value.as_f32() else {
            continue;
        };
        out.insert(entry_key.clone(), num);
    }
    out
}

fn take_string_u8_map(
    fields: &mut HashMap<String, PackArgValue>,
    key: &str,
) -> HashMap<String, u8> {
    let Some(value) = fields.remove(key) else {
        return HashMap::default();
    };
    let Some(map) = value.as_map() else {
        return HashMap::default();
    };
    let mut out = HashMap::with_capacity(map.len());
    for (entry_key, entry_value) in map {
        let Some(num) = entry_value.as_u8() else {
            continue;
        };
        out.insert(entry_key.clone(), num);
    }
    out
}

fn parse_normal_dist(value: &PackArgValue, default: RankDist) -> RankDist {
    if let Some(map) = value.as_map() {
        let mut out = default;
        if let Some(next) = map.get("min_dev").and_then(PackArgValue::as_f32) {
            out.min_dev = next;
        }
        if let Some(next) = map.get("max_dev").and_then(PackArgValue::as_f32) {
            out.max_dev = next;
        }
        if let Some(next) = map.get("mean").and_then(PackArgValue::as_f32) {
            out.mean = next;
        }
        if let Some(next) = map.get("std_dev").and_then(PackArgValue::as_f32) {
            out.std_dev = next;
        }
        return out;
    }
    if let Some(list) = value.as_list() && list.len() == 4 {
        let mut out = default;
        if let Some(next) = list[0].as_f32() {
            out.min_dev = next;
        }
        if let Some(next) = list[1].as_f32() {
            out.max_dev = next;
        }
        if let Some(next) = list[2].as_f32() {
            out.mean = next;
        }
        if let Some(next) = list[3].as_f32() {
            out.std_dev = next;
        }
        return out;
    }
    default
}

fn take_normal_dist(
    fields: &mut HashMap<String, PackArgValue>,
    key: &str,
    default: RankDist,
) -> RankDist {
    let Some(value) = fields.remove(key) else {
        return default;
    };
    parse_normal_dist(&value, default)
}

fn parse_member_key(raw: &str) -> (String, bool) {
    if let Some(stripped) = raw.strip_prefix("race:") {
        return (stripped.trim().to_string(), true);
    }
    if let Some(stripped) = raw.strip_prefix("bit:") {
        return (stripped.trim().to_string(), false);
    }
    (raw.trim().to_string(), false)
}

fn parse_member_config(value: &PackArgValue, inherited_race_first: bool) -> PackMemberConfigSeri {
    let mut config = PackMemberConfigSeri {
        race_first: inherited_race_first,
        ..PackMemberConfigSeri::default()
    };
    match value {
        PackArgValue::Int(_) | PackArgValue::Float(_) => {
            if let Some(weight) = value.as_f32() {
                config.weight = weight;
            }
        }
        PackArgValue::Map(map) => {
            if let Some(weight) = map.get("weight").and_then(PackArgValue::as_f32) {
                config.weight = weight;
            }
            if let Some(rank) = map.get("rank") {
                config.rank_dist = parse_normal_dist(rank, config.rank_dist.clone());
            }
            if let Some(min) = map.get("min").and_then(PackArgValue::as_u32) {
                config.min = min;
            }
            if let Some(max) = map.get("max").and_then(PackArgValue::as_u32) {
                config.max = max;
            }
            if let Some(race_first) = map.get("race_first").and_then(PackArgValue::as_bool) {
                config.race_first = race_first;
            }
        }
        _ => {}
    }
    if config.max < config.min {
        config.max = config.min;
    }
    config
}

fn take_ids_map(
    fields: &mut HashMap<String, PackArgValue>,
    key: &str,
) -> HashMap<String, PackMemberConfigSeri> {
    let Some(value) = fields.remove(key) else {
        return HashMap::default();
    };
    let Some(map) = value.as_map() else {
        return HashMap::default();
    };
    let mut out = HashMap::with_capacity(map.len());
    for (raw_id, value) in map {
        let (id, race_first) = parse_member_key(raw_id);
        if id.trim().is_empty() {
            continue;
        }
        let config = parse_member_config(value, race_first);
        out.insert(id, config);
    }
    out
}

fn take_wander_seri(
    fields: &mut HashMap<String, PackArgValue>,
    key: &str,
    default: WanderSeri,
) -> WanderSeri {
    let Some(value) = fields.remove(key) else {
        return default;
    };
    let Some(map) = value.as_map() else {
        return default;
    };
    let mut out = default;
    if let Some(next) = map.get("dir_secs_min").and_then(PackArgValue::as_f32) {
        out.dir_secs_min = next;
    }
    if let Some(next) = map.get("dir_secs_max").and_then(PackArgValue::as_f32) {
        out.dir_secs_max = next;
    }
    if let Some(next) = map.get("move_secs_min").and_then(PackArgValue::as_f32) {
        out.move_secs_min = next;
    }
    if let Some(next) = map.get("move_secs_max").and_then(PackArgValue::as_f32) {
        out.move_secs_max = next;
    }
    if let Some(next) = map.get("halt_secs_min").and_then(PackArgValue::as_f32) {
        out.halt_secs_min = next;
    }
    if let Some(next) = map.get("halt_secs_max").and_then(PackArgValue::as_f32) {
        out.halt_secs_max = next;
    }
    if let Some(next) = map.get("speed_min").and_then(PackArgValue::as_f32) {
        out.speed_min = next;
    }
    if let Some(next) = map.get("speed_max").and_then(PackArgValue::as_f32) {
        out.speed_max = next;
    }
    if let Some(next) = map.get("max_drift").and_then(PackArgValue::as_f32) {
        out.max_drift = next;
    }
    if let Some(next) = map.get("allow_hard_flips_to_return").and_then(PackArgValue::as_bool) {
        out.allow_hard_flips_to_return = next;
    }
    if let Some(next) = map.get("wander_around_leader").and_then(PackArgValue::as_bool) {
        out.wander_around_leader = next;
    }
    if let Some(next) = map.get("avoid_blacklisted_spawn_tiles").and_then(PackArgValue::as_bool) {
        out.avoid_blacklisted_spawn_tiles = next;
    }
    if let Some(next) = map.get("pack_orbit_radius").and_then(PackArgValue::as_f32) {
        out.pack_orbit_radius = next;
    }
    if let Some(next) = map
        .get("pack_orbit_retarget_secs_min")
        .and_then(PackArgValue::as_f32)
    {
        out.pack_orbit_retarget_secs_min = next;
    }
    if let Some(next) = map
        .get("pack_orbit_retarget_secs_max")
        .and_then(PackArgValue::as_f32)
    {
        out.pack_orbit_retarget_secs_max = next;
    }
    if let Some(avoid_tags) = map.get("avoid_tile_tags").and_then(PackArgValue::as_list) {
        out.avoid_tile_tags.clear();
        for tag in avoid_tags {
            let Some(tag) = tag.as_string() else {
                continue;
            };
            if tag.trim().is_empty() {
                continue;
            }
            out.avoid_tile_tags.insert(tag);
        }
    }
    out
}

pub fn load_pack_seri_defs() -> Vec<PackSeri> {
    let mut discovered = match def_db::discover_assets_files_by_suffixes(&[".pack.ron"]) {
        Ok(discovered) => discovered,
        Err(err) => {
            error!(target: BEING_TEMPLATE_INIT, "Failed discovering pack defs: {}", err);
            return Vec::new();
        }
    };
    discovered.sort_by(|(a, _), (b, _)| {
        a.precedence_rank()
            .cmp(&b.precedence_rank())
            .then_with(|| a.rel_path.cmp(&b.rel_path))
    });

    let mut by_id: HashMap<String, PackSeri> = HashMap::default();
    let mut id_order = Vec::new();
    for (source, abs_path) in discovered {
        let content = match std::fs::read_to_string(&abs_path) {
            Ok(content) => content,
            Err(err) => {
                error!(
                    target: BEING_TEMPLATE_INIT,
                    "Failed reading pack file '{}': {}",
                    source.rel_path,
                    err,
                );
                continue;
            }
        };
        let parsed = match parse_pack_seri(&content, &abs_path) {
            Ok(parsed) => parsed,
            Err(err) => {
                error!(
                    target: BEING_TEMPLATE_INIT,
                    "Failed parsing pack file '{}': {}",
                    source.rel_path,
                    err,
                );
                continue;
            }
        };
        if parsed.id.trim().is_empty() {
            error!(
                target: BEING_TEMPLATE_INIT,
                "Skipping pack file '{}': empty id",
                source.rel_path,
            );
            continue;
        }
        let id = parsed.id.clone();
        let Some(_) = by_id.insert(id.clone(), parsed) else {
            id_order.push(id);
            continue;
        };
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
