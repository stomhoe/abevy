use anyhow::{Context, Result};
use bevy::prelude::*;
use ron::{Map as RonMap, Value as RonValue};
use serde::de::DeserializeOwned;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum DefSourceKind {
    Base,
    Mod,
}

#[derive(Debug, Clone)]
pub struct DefSource {
    pub kind: DefSourceKind,
    /// Path relative to `assets/`, using forward slashes.
    pub rel_path: String,
}

impl DefSource {
    pub fn precedence_rank(&self) -> u8 {
        match self.kind {
            DefSourceKind::Base => 0,
            DefSourceKind::Mod => 1,
        }
    }
}

#[derive(Debug, Clone)]
pub struct DefRecord<T> {
    pub id: String,
    pub source: DefSource,
    pub value: T,
}

#[derive(Debug, Clone)]
pub struct DefRawRecord {
    pub id: String,
    pub source: DefSource,
    pub value: RonValue,
}

#[derive(Debug, Clone)]
pub struct DefOverride {
    pub id: String,
    pub previous_source: DefSource,
    pub replacement_source: DefSource,
}

#[derive(Debug, Clone)]
pub struct AppliedPatch {
    pub def_type: String,
    pub id: String,
    pub op: String,
    pub source: String,
}

#[derive(Debug, Clone, Default)]
pub struct RegisteredDefType {
    pub by_id: HashMap<String, DefRawRecord>,
    pub ordered_ids: Vec<String>,
}

#[derive(Debug, Clone, Default)]
pub struct GlobalDefRegistry {
    pub by_type: HashMap<String, RegisteredDefType>,
    pub patches: Vec<AppliedPatch>,
}

fn global_registry() -> &'static Mutex<GlobalDefRegistry> {
    static REGISTRY: OnceLock<Mutex<GlobalDefRegistry>> = OnceLock::new();
    REGISTRY.get_or_init(|| Mutex::new(GlobalDefRegistry::default()))
}

fn expected_def_types() -> &'static Mutex<HashSet<String>> {
    static EXPECTED: OnceLock<Mutex<HashSet<String>>> = OnceLock::new();
    EXPECTED.get_or_init(|| Mutex::new(HashSet::new()))
}

fn validation_rules() -> &'static Mutex<Vec<DefRefRule>> {
    static RULES: OnceLock<Mutex<Vec<DefRefRule>>> = OnceLock::new();
    RULES.get_or_init(|| Mutex::new(Vec::new()))
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum DefPatchOp {
    Upsert {
        #[serde(alias = "type")]
        def_type: String,
        id: Option<String>,
        value: RonValue,
    },
    Delete {
        #[serde(alias = "type")]
        def_type: String,
        id: String,
    },
    SetField {
        #[serde(alias = "type")]
        def_type: String,
        id: String,
        path: String,
        value: RonValue,
    },
    RemoveField {
        #[serde(alias = "type")]
        def_type: String,
        id: String,
        path: String,
    },
    Merge {
        #[serde(alias = "type")]
        def_type: String,
        id: String,
        value: RonValue,
    },
    Copy {
        #[serde(alias = "type")]
        def_type: String,
        from_id: String,
        to_id: String,
        overwrite: Option<bool>,
    },
}

#[derive(Debug, Clone)]
pub struct DefRefRule {
    pub from_type: String,
    pub from_path: String,
    pub to_type: String,
    pub allow_missing: bool,
}

#[derive(Resource, Debug, Clone)]
pub struct DefValidationConfig {
    pub enabled: bool,
    pub fail_fast: bool,
}

impl Default for DefValidationConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            fail_fast: true,
        }
    }
}

#[derive(Resource, Debug, Default, Clone)]
pub struct DefValidationRuntime {
    pub attempted: bool,
    pub completed: bool,
}

/// Helper for allowing both `T` and `Vec<T>` inside the same `.ron`.
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(untagged)]
pub enum OneOrMany<T> {
    One(T),
    Many(Vec<T>),
}

impl<T> OneOrMany<T> {
    pub fn into_vec(self) -> Vec<T> {
        match self {
            Self::One(v) => vec![v],
            Self::Many(v) => v,
        }
    }
}

#[derive(Resource, Debug, Clone)]
pub struct DefDatabase<T> {
    /// Final merged view (last-wins by source precedence order).
    by_id: HashMap<String, DefRecord<T>>,
    /// Stable insertion order for iteration.
    ordered_ids: Vec<String>,
    /// Override history for diagnostics.
    overrides: Vec<DefOverride>,
}

impl<T> Default for DefDatabase<T> {
    fn default() -> Self {
        Self {
            by_id: HashMap::default(),
            ordered_ids: Vec::new(),
            overrides: Vec::new(),
        }
    }
}

impl<T> DefDatabase<T> {
    pub fn get(&self, id: &str) -> Option<&DefRecord<T>> {
        self.by_id.get(id)
    }

    pub fn iter(&self) -> impl Iterator<Item = &DefRecord<T>> {
        self.ordered_ids
            .iter()
            .filter_map(|id| self.by_id.get(id))
    }

    pub fn len(&self) -> usize {
        self.by_id.len()
    }

    pub fn is_empty(&self) -> bool {
        self.by_id.is_empty()
    }

    pub fn overrides(&self) -> &[DefOverride] {
        &self.overrides
    }

    pub fn into_records(mut self) -> Vec<DefRecord<T>> {
        self.ordered_ids
            .into_iter()
            .filter_map(|id| self.by_id.remove(&id))
            .collect()
    }
}

impl<T: DeserializeOwned> DefDatabase<T> {
    pub fn load_from_assets_dir(
        suffixes: &[&str],
        id_of: impl Fn(&T) -> &str,
    ) -> Result<Self> {
        let fallback_type = std::any::type_name::<T>().to_string();
        Self::load_from_assets_dir_with_type(&fallback_type, suffixes, id_of)
    }

    pub fn load_from_assets_dir_with_type(
        def_type: &str,
        suffixes: &[&str],
        id_of: impl Fn(&T) -> &str,
    ) -> Result<Self> {
        let raw = load_raw_defs_with_patches(def_type, suffixes)?;
        let mut db = DefDatabase::<T>::default();
        for record in raw.records {
            let value = record.value.clone().into_rust::<T>().with_context(|| {
                format!(
                    "Failed deserializing '{}' def '{}' from '{}'",
                    def_type, record.id, record.source.rel_path
                )
            })?;
            let id = id_of(&value).trim().to_string();
            if id.is_empty() {
                warn!("Skipping '{}' def with empty id from '{}'", def_type, record.source.rel_path);
                continue;
            }
            let typed_record = DefRecord {
                id: id.clone(),
                source: record.source,
                value,
            };
            let replacement_source = typed_record.source.clone();
            if let Some(previous) = db.by_id.insert(id.clone(), typed_record) {
                db.overrides.push(DefOverride {
                    id,
                    previous_source: previous.source,
                    replacement_source,
                });
            } else {
                db.ordered_ids.push(id);
            }
        }
        Ok(db)
    }

    pub fn resolve_typed_ref(def_type: &str, id: &str) -> Result<Option<T>> {
        let Some(value) = resolve_def_ref(def_type, id).map(|record| record.value) else {
            return Ok(None);
        };
        Ok(Some(value.into_rust::<T>().with_context(|| {
            format!("Failed to deserialize resolved ref '{}:{}'", def_type, id)
        })?))
    }

    #[allow(dead_code)]
    pub fn load_from_sources(
        mut discovered: Vec<(DefSource, PathBuf)>,
        id_of: impl Fn(&T) -> &str,
    ) -> Result<Self> {
        discovered.sort_by(|(a, _), (b, _)| {
            a.precedence_rank()
                .cmp(&b.precedence_rank())
                .then_with(|| a.rel_path.cmp(&b.rel_path))
        });

        let mut db = DefDatabase::<T>::default();

        for (source, abs_path) in discovered {
            let content = std::fs::read_to_string(&abs_path).with_context(|| {
                format!(
                    "Failed reading def file '{}'",
                    abs_path.to_string_lossy()
                )
            })?;
            let parsed = ron::from_str::<OneOrMany<T>>(&content).with_context(|| {
                format!(
                    "Failed parsing RON def file '{}'",
                    abs_path.to_string_lossy()
                )
            })?;
            for value in parsed.into_vec() {
                let id = id_of(&value).trim().to_string();
                if id.is_empty() {
                    // Skip instead of failing the whole load; we want mods to be resilient.
                    warn!(
                        "Skipping def in '{}' with empty id",
                        source.rel_path
                    );
                    continue;
                }
                let record = DefRecord {
                    id: id.clone(),
                    source: source.clone(),
                    value,
                };
                if let Some(previous) = db.by_id.insert(id.clone(), record) {
                    db.overrides.push(DefOverride {
                        id,
                        previous_source: previous.source,
                        replacement_source: source.clone(),
                    });
                } else {
                    db.ordered_ids.push(id);
                }
            }
        }

        Ok(db)
    }
}

#[derive(Debug, Default)]
struct RawLoadResult {
    records: Vec<DefRawRecord>,
}

pub fn global_registry_snapshot() -> GlobalDefRegistry {
    global_registry()
        .lock()
        .map(|g| g.clone())
        .unwrap_or_default()
}

pub fn register_expected_def_type(def_type: &str) {
    if let Ok(mut expected) = expected_def_types().lock() {
        expected.insert(def_type.to_string());
    }
}

pub fn register_ref_rule(rule: DefRefRule) {
    if let Ok(mut rules) = validation_rules().lock() {
        if !rules.iter().any(|r| {
            r.from_type == rule.from_type
                && r.from_path == rule.from_path
                && r.to_type == rule.to_type
        }) {
            rules.push(rule);
        }
    }
}

pub fn validate_global_registry() -> Result<()> {
    let registry = global_registry_snapshot();
    let rules = validation_rules()
        .lock()
        .map(|rules| rules.clone())
        .unwrap_or_default();
    if rules.is_empty() {
        return Ok(());
    }

    let mut errors = Vec::new();
    for rule in rules {
        let Some(source_defs) = registry.by_type.get(&rule.from_type) else {
            continue;
        };
        for source_id in &source_defs.ordered_ids {
            let Some(record) = source_defs.by_id.get(source_id) else {
                continue;
            };
            let Some(field) = resolve_path_value(&record.value, &rule.from_path) else {
                continue;
            };
            for candidate_id in extract_ref_ids(field) {
                if candidate_id.trim().is_empty() {
                    continue;
                }
                let exists = registry
                    .by_type
                    .get(&rule.to_type)
                    .and_then(|ty| ty.by_id.get(candidate_id.as_str()))
                    .is_some();
                if !exists && !rule.allow_missing {
                    errors.push(format!(
                        "{}:{} path '{}' -> missing {}:{}",
                        rule.from_type, source_id, rule.from_path, rule.to_type, candidate_id
                    ));
                }
            }
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        anyhow::bail!(
            "Def validation failed with {} error(s):\n{}",
            errors.len(),
            errors.join("\n")
        );
    }
}

pub fn expected_types_loaded() -> bool {
    let expected = expected_def_types()
        .lock()
        .map(|set| set.clone())
        .unwrap_or_default();
    if expected.is_empty() {
        return true;
    }
    let snapshot = global_registry_snapshot();
    expected
        .iter()
        .all(|def_type| snapshot.by_type.contains_key(def_type))
}

pub fn resolve_def_ref(def_type: &str, id: &str) -> Option<DefRawRecord> {
    let Ok(registry) = global_registry().lock() else {
        return None;
    };
    registry
        .by_type
        .get(def_type)
        .and_then(|ty| ty.by_id.get(id))
        .cloned()
}

pub fn resolve_def_field(def_type: &str, id: &str, path: &str) -> Option<RonValue> {
    let record = resolve_def_ref(def_type, id)?;
    let tokens = parse_path(path).ok()?;
    let mut cursor = &record.value;
    for token in tokens {
        match token {
            PathToken::Key(key) => {
                let RonValue::Map(map) = cursor else { return None };
                cursor = map.iter().find_map(|(k, v)| match k {
                    RonValue::String(s) if s == &key => Some(v),
                    _ => None,
                })?;
            }
            PathToken::Index(index) => {
                let RonValue::Seq(seq) = cursor else { return None };
                cursor = seq.get(index)?;
            }
        }
    }
    Some(cursor.clone())
}

fn load_raw_defs_with_patches(def_type: &str, suffixes: &[&str]) -> Result<RawLoadResult> {
    let discovered = discover_assets_files_by_suffixes(suffixes)?;
    let mut raw_by_id: HashMap<String, DefRawRecord> = HashMap::new();
    let mut ordered_ids: Vec<String> = Vec::new();

    let mut ordered = discovered;
    ordered.sort_by(|(a, _), (b, _)| {
        a.precedence_rank()
            .cmp(&b.precedence_rank())
            .then_with(|| a.rel_path.cmp(&b.rel_path))
    });

    for (source, abs_path) in ordered {
        let content = std::fs::read_to_string(&abs_path).with_context(|| {
            format!("Failed reading def file '{}'", abs_path.to_string_lossy())
        })?;
        let parsed = ron::from_str::<OneOrMany<RonValue>>(&content).with_context(|| {
            format!("Failed parsing RON def file '{}'", abs_path.to_string_lossy())
        })?;

        for value in parsed.into_vec() {
            let Some(id) = extract_id(&value) else {
                warn!("Skipping '{}' def in '{}' with missing/invalid id", def_type, source.rel_path);
                continue;
            };
            let record = DefRawRecord {
                id: id.clone(),
                source: source.clone(),
                value,
            };
            let existed = raw_by_id.insert(id.clone(), record).is_some();
            if !existed {
                ordered_ids.push(id);
            }
        }
    }

    let mut registry_patches = Vec::new();
    apply_patch_files(def_type, &mut raw_by_id, &mut ordered_ids, &mut registry_patches)?;

    let mut out = RawLoadResult::default();
    out.records = ordered_ids
        .into_iter()
        .filter_map(|id| raw_by_id.remove(&id))
        .collect();

    if let Ok(mut registry) = global_registry().lock() {
        let mut typed = RegisteredDefType::default();
        for record in &out.records {
            typed.ordered_ids.push(record.id.clone());
            typed.by_id.insert(record.id.clone(), record.clone());
        }
        registry.by_type.insert(def_type.to_string(), typed);
        registry.patches.extend(registry_patches);
    }
    Ok(out)
}

fn apply_patch_files(
    def_type: &str,
    raw_by_id: &mut HashMap<String, DefRawRecord>,
    ordered_ids: &mut Vec<String>,
    applied: &mut Vec<AppliedPatch>,
) -> Result<()> {
    let patch_files = discover_assets_files_by_suffixes(&[".defpatch.ron"])?;
    for (source, abs_path) in patch_files {
        let content = match std::fs::read_to_string(&abs_path) {
            Ok(content) => content,
            Err(err) => {
                warn!("Failed reading patch file '{}': {}", source.rel_path, err);
                continue;
            }
        };
        let ops = match ron::from_str::<OneOrMany<DefPatchOp>>(&content) {
            Ok(ops) => ops.into_vec(),
            Err(err) => {
                warn!("Failed parsing patch file '{}': {}", source.rel_path, err);
                continue;
            }
        };
        for op in ops {
            apply_patch_op(def_type, op, raw_by_id, ordered_ids, &source.rel_path, applied);
        }
    }
    Ok(())
}

fn apply_patch_op(
    def_type: &str,
    op: DefPatchOp,
    raw_by_id: &mut HashMap<String, DefRawRecord>,
    ordered_ids: &mut Vec<String>,
    source_rel_path: &str,
    applied: &mut Vec<AppliedPatch>,
) {
    match op {
        DefPatchOp::Upsert { def_type: patch_type, id, mut value } => {
            if patch_type != def_type {
                return;
            }
            let def_id = id.or_else(|| extract_id(&value));
            if let Some(ref id_str) = def_id {
                ensure_id_field(&mut value, id_str);
            }
            let Some(def_id) = def_id else { return };
            let record = DefRawRecord {
                id: def_id.clone(),
                source: DefSource { kind: DefSourceKind::Mod, rel_path: source_rel_path.to_string() },
                value,
            };
            if raw_by_id.insert(def_id.clone(), record).is_none() {
                ordered_ids.push(def_id.clone());
            }
            applied.push(AppliedPatch { def_type: patch_type, id: def_id, op: "upsert".to_string(), source: source_rel_path.to_string() });
        }
        DefPatchOp::Delete { def_type: patch_type, id } => {
            if patch_type != def_type {
                return;
            }
            raw_by_id.remove(&id);
            ordered_ids.retain(|entry| entry != &id);
            applied.push(AppliedPatch { def_type: patch_type, id, op: "delete".to_string(), source: source_rel_path.to_string() });
        }
        DefPatchOp::SetField { def_type: patch_type, id, path, value } => {
            if patch_type != def_type {
                return;
            }
            let Some(record) = raw_by_id.get_mut(&id) else { return };
            if let Ok(tokens) = parse_path(&path) {
                set_path_value(&mut record.value, &tokens, value);
                applied.push(AppliedPatch { def_type: patch_type, id, op: "set_field".to_string(), source: source_rel_path.to_string() });
            }
        }
        DefPatchOp::RemoveField { def_type: patch_type, id, path } => {
            if patch_type != def_type {
                return;
            }
            let Some(record) = raw_by_id.get_mut(&id) else { return };
            if let Ok(tokens) = parse_path(&path) {
                remove_path_value(&mut record.value, &tokens);
                applied.push(AppliedPatch { def_type: patch_type, id, op: "remove_field".to_string(), source: source_rel_path.to_string() });
            }
        }
        DefPatchOp::Merge { def_type: patch_type, id, value } => {
            if patch_type != def_type {
                return;
            }
            let Some(record) = raw_by_id.get_mut(&id) else { return };
            merge_values(&mut record.value, value);
            applied.push(AppliedPatch { def_type: patch_type, id, op: "merge".to_string(), source: source_rel_path.to_string() });
        }
        DefPatchOp::Copy { def_type: patch_type, from_id, to_id, overwrite } => {
            if patch_type != def_type {
                return;
            }
            if overwrite != Some(true) && raw_by_id.contains_key(&to_id) {
                return;
            }
            let Some(from) = raw_by_id.get(&from_id).cloned() else { return };
            let mut cloned = from;
            cloned.id = to_id.clone();
            ensure_id_field(&mut cloned.value, &to_id);
            raw_by_id.insert(to_id.clone(), cloned);
            if !ordered_ids.iter().any(|id| id == &to_id) {
                ordered_ids.push(to_id.clone());
            }
            applied.push(AppliedPatch { def_type: patch_type, id: to_id, op: "copy".to_string(), source: source_rel_path.to_string() });
        }
    }
}

fn ensure_id_field(value: &mut RonValue, id: &str) {
    let RonValue::Map(map) = value else {
        return;
    };
    map.insert(RonValue::String("id".to_string()), RonValue::String(id.to_string()));
}

fn extract_id(value: &RonValue) -> Option<String> {
    let RonValue::Map(map) = value else {
        return None;
    };
    map.iter().find_map(|(key, value)| match (key, value) {
        (RonValue::String(key), RonValue::String(id)) if key == "id" && !id.trim().is_empty() => Some(id.trim().to_string()),
        _ => None,
    })
}

#[derive(Debug)]
enum PathToken {
    Key(String),
    Index(usize),
}

fn parse_path(path: &str) -> Result<Vec<PathToken>> {
    let mut out = Vec::new();
    for segment in path.split('.') {
        let segment = segment.trim();
        if segment.is_empty() {
            continue;
        }
        if let Some(bracket) = segment.find('[') {
            let key = &segment[..bracket];
            if !key.is_empty() {
                out.push(PathToken::Key(key.to_string()));
            }
            let idx_str = segment.trim_start_matches(&format!("{}[", key)).trim_end_matches(']');
            let index = idx_str.parse::<usize>().with_context(|| format!("Invalid index in path '{}'", path))?;
            out.push(PathToken::Index(index));
        } else {
            out.push(PathToken::Key(segment.to_string()));
        }
    }
    Ok(out)
}

fn set_path_value(root: &mut RonValue, tokens: &[PathToken], value: RonValue) {
    if tokens.is_empty() {
        *root = value;
        return;
    }
    let mut cursor = root;
    for token in &tokens[..tokens.len() - 1] {
        match token {
            PathToken::Key(key) => {
                if !matches!(cursor, RonValue::Map(_)) {
                    *cursor = RonValue::Map(RonMap::new());
                }
                let RonValue::Map(map) = cursor else { return };
                let key_value = RonValue::String(key.clone());
                if !map.iter().any(|(existing, _)| existing == &key_value) {
                    map.insert(key_value.clone(), RonValue::Map(RonMap::new()));
                }
                cursor = map.iter_mut().find_map(|(k, v)| if k == &key_value { Some(v) } else { None }).unwrap();
            }
            PathToken::Index(index) => {
                if !matches!(cursor, RonValue::Seq(_)) {
                    *cursor = RonValue::Seq(Vec::new());
                }
                let RonValue::Seq(seq) = cursor else { return };
                while seq.len() <= *index {
                    seq.push(RonValue::Unit);
                }
                cursor = &mut seq[*index];
            }
        }
    }
    match tokens.last().unwrap() {
        PathToken::Key(key) => {
            if !matches!(cursor, RonValue::Map(_)) {
                *cursor = RonValue::Map(RonMap::new());
            }
            let RonValue::Map(map) = cursor else { return };
            let key_value = RonValue::String(key.clone());
            map.insert(key_value, value);
        }
        PathToken::Index(index) => {
            if !matches!(cursor, RonValue::Seq(_)) {
                *cursor = RonValue::Seq(Vec::new());
            }
            let RonValue::Seq(seq) = cursor else { return };
            while seq.len() <= *index {
                seq.push(RonValue::Unit);
            }
            seq[*index] = value;
        }
    }
}

fn remove_path_value(root: &mut RonValue, tokens: &[PathToken]) {
    if tokens.is_empty() {
        return;
    }
    let mut cursor = root;
    for token in &tokens[..tokens.len() - 1] {
        match token {
            PathToken::Key(key) => {
                let RonValue::Map(map) = cursor else { return };
                let key_value = RonValue::String(key.clone());
                let Some(next) = map.iter_mut().find_map(|(k, v)| if k == &key_value { Some(v) } else { None }) else { return };
                cursor = next;
            }
            PathToken::Index(index) => {
                let RonValue::Seq(seq) = cursor else { return };
                let Some(next) = seq.get_mut(*index) else { return };
                cursor = next;
            }
        }
    }
    match tokens.last().unwrap() {
        PathToken::Key(key) => {
            let RonValue::Map(map) = cursor else { return };
            map.remove(&RonValue::String(key.clone()));
        }
        PathToken::Index(index) => {
            let RonValue::Seq(seq) = cursor else { return };
            if *index < seq.len() {
                seq.remove(*index);
            }
        }
    }
}

fn merge_values(target: &mut RonValue, patch: RonValue) {
    match (target, patch) {
        (RonValue::Map(target_map), RonValue::Map(patch_map)) => {
            for (key, patch_value) in patch_map {
                if let Some(target_value) = target_map.iter_mut().find_map(|(existing_key, existing_value)| {
                    if existing_key == &key {
                        Some(existing_value)
                    } else {
                        None
                    }
                }) {
                    merge_values(target_value, patch_value);
                } else {
                    target_map.insert(key, patch_value);
                }
            }
        }
        (RonValue::Seq(target_seq), RonValue::Seq(patch_seq)) => {
            target_seq.extend(patch_seq);
        }
        (target_value, patch_value) => {
            *target_value = patch_value;
        }
    }
}

fn resolve_path_value<'a>(root: &'a RonValue, path: &str) -> Option<&'a RonValue> {
    let tokens = parse_path(path).ok()?;
    let mut cursor = root;
    for token in tokens {
        match token {
            PathToken::Key(key) => {
                let RonValue::Map(map) = cursor else { return None };
                cursor = map.iter().find_map(|(k, v)| match k {
                    RonValue::String(s) if s == &key => Some(v),
                    _ => None,
                })?;
            }
            PathToken::Index(index) => {
                let RonValue::Seq(seq) = cursor else { return None };
                cursor = seq.get(index)?;
            }
        }
    }
    Some(cursor)
}

fn extract_ref_ids(value: &RonValue) -> Vec<String> {
    match value {
        RonValue::String(id) => vec![id.clone()],
        RonValue::Seq(seq) => seq
            .iter()
            .flat_map(extract_ref_ids)
            .collect(),
        RonValue::Map(map) => {
            let mut out = Vec::new();
            for (key, value) in map.iter() {
                if let RonValue::String(key) = key {
                    if key == "id" {
                        if let RonValue::String(id) = value {
                            out.push(id.clone());
                        }
                    }
                }
            }
            out
        }
        _ => Vec::new(),
    }
}

pub fn discover_assets_files_by_suffixes(suffixes: &[&str]) -> Result<Vec<(DefSource, PathBuf)>> {
    let assets_root = Path::new("assets");
    if !assets_root.exists() {
        return Ok(Vec::new());
    }
    discover_assets_files_matching(assets_root, |rel_string| {
        suffixes.iter().any(|s| rel_string.ends_with(s))
    })
}

pub fn discover_assets_files_matching(
    assets_root: &Path,
    matches: impl Fn(&str) -> bool,
) -> Result<Vec<(DefSource, PathBuf)>> {
    if !assets_root.exists() {
        return Ok(Vec::new());
    }

    let mut stack = vec![assets_root.to_path_buf()];
    let mut out: Vec<(DefSource, PathBuf)> = Vec::new();

    while let Some(dir) = stack.pop() {
        let entries = std::fs::read_dir(&dir)
            .with_context(|| format!("Failed reading directory '{}'", dir.to_string_lossy()))?;
        for entry in entries.flatten() {
            let path = entry.path();
            let meta = match entry.metadata() {
                Ok(m) => m,
                Err(_) => continue,
            };
            if meta.is_dir() {
                stack.push(path);
                continue;
            }
            if !meta.is_file() {
                continue;
            }

            let Some(rel_path) = path.strip_prefix(assets_root).ok() else {
                continue;
            };
            let rel_string = to_forward_slash_path(rel_path);
            if !matches(&rel_string) {
                continue;
            }
            let kind = if rel_string.starts_with("mods/") || rel_string.contains("/mods/") {
                DefSourceKind::Mod
            } else {
                DefSourceKind::Base
            };
            out.push((
                DefSource { kind, rel_path: rel_string },
                path,
            ));
        }
    }

    out.sort_by(|(a, _), (b, _)| a.rel_path.cmp(&b.rel_path));
    let mut seen = HashSet::new();
    out.retain(|(source, _)| seen.insert(source.rel_path.clone()));

    Ok(out)
}

pub fn to_forward_slash_path(path: &Path) -> String {
    let mut out = String::new();
    for (idx, part) in path.components().enumerate() {
        if idx > 0 {
            out.push('/');
        }
        out.push_str(&part.as_os_str().to_string_lossy());
    }
    out
}
