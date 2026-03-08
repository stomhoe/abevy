use anyhow::{Context, Result};
use bevy::prelude::*;
use serde::de::{
    value::Error as ValueDeError,
    DeserializeOwned, DeserializeSeed, EnumAccess, IntoDeserializer, MapAccess, SeqAccess,
    VariantAccess, Visitor,
};
use serde::ser::{SerializeMap, SerializeSeq};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

const DEF_VALUE_ENUM_VARIANT_KEY: &str = "__def_value_enum_variant";
const DEF_VALUE_ENUM_PAYLOAD_KEY: &str = "__def_value_enum_payload";

#[derive(Debug, Clone, PartialEq)]
pub enum DefValue {
    Bool(bool),
    Char(char),
    String(String),
    Bytes(Vec<u8>),
    I64(i64),
    U64(u64),
    F64(f64),
    Option(Option<Box<DefValue>>),
    Seq(Vec<DefValue>),
    Map(Vec<(DefValue, DefValue)>),
    Unit,
    Enum(String, Box<DefValue>),
}

impl Serialize for DefValue {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Self::Bool(v) => serializer.serialize_bool(*v),
            Self::Char(v) => serializer.serialize_char(*v),
            Self::String(v) => serializer.serialize_str(v),
            Self::Bytes(v) => serializer.serialize_bytes(v),
            Self::I64(v) => serializer.serialize_i64(*v),
            Self::U64(v) => serializer.serialize_u64(*v),
            Self::F64(v) => serializer.serialize_f64(*v),
            Self::Option(Some(v)) => serializer.serialize_some(v),
            Self::Option(None) => serializer.serialize_none(),
            Self::Seq(values) => {
                let mut seq = serializer.serialize_seq(Some(values.len()))?;
                for value in values {
                    seq.serialize_element(value)?;
                }
                seq.end()
            }
            Self::Map(entries) => {
                let mut map = serializer.serialize_map(Some(entries.len()))?;
                for (key, value) in entries {
                    map.serialize_entry(key, value)?;
                }
                map.end()
            }
            Self::Unit => serializer.serialize_unit(),
            Self::Enum(variant, payload) => {
                let mut map = serializer.serialize_map(Some(2))?;
                map.serialize_entry(DEF_VALUE_ENUM_VARIANT_KEY, variant)?;
                map.serialize_entry(DEF_VALUE_ENUM_PAYLOAD_KEY, payload)?;
                map.end()
            }
        }
    }
}

impl<'de> Deserialize<'de> for DefValue {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(DefValueVisitor)
    }
}

struct DefValueVisitor;

impl<'de> Visitor<'de> for DefValueVisitor {
    type Value = DefValue;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a RON value")
    }

    fn visit_bool<E>(self, v: bool) -> std::result::Result<Self::Value, E> {
        Ok(DefValue::Bool(v))
    }

    fn visit_i64<E>(self, v: i64) -> std::result::Result<Self::Value, E> {
        Ok(DefValue::I64(v))
    }

    fn visit_u64<E>(self, v: u64) -> std::result::Result<Self::Value, E> {
        Ok(DefValue::U64(v))
    }

    fn visit_f64<E>(self, v: f64) -> std::result::Result<Self::Value, E> {
        Ok(DefValue::F64(v))
    }

    fn visit_char<E>(self, v: char) -> std::result::Result<Self::Value, E> {
        Ok(DefValue::Char(v))
    }

    fn visit_str<E>(self, v: &str) -> std::result::Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_string(v.to_string())
    }

    fn visit_string<E>(self, v: String) -> std::result::Result<Self::Value, E> {
        Ok(DefValue::String(v))
    }

    fn visit_bytes<E>(self, v: &[u8]) -> std::result::Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_byte_buf(v.to_vec())
    }

    fn visit_byte_buf<E>(self, v: Vec<u8>) -> std::result::Result<Self::Value, E> {
        Ok(DefValue::Bytes(v))
    }

    fn visit_none<E>(self) -> std::result::Result<Self::Value, E> {
        Ok(DefValue::Option(None))
    }

    fn visit_some<D>(self, deserializer: D) -> std::result::Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(DefValue::Option(Some(Box::new(DefValue::deserialize(
            deserializer,
        )?))))
    }

    fn visit_unit<E>(self) -> std::result::Result<Self::Value, E> {
        Ok(DefValue::Unit)
    }

    fn visit_newtype_struct<D>(self, deserializer: D) -> std::result::Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        DefValue::deserialize(deserializer)
    }

    fn visit_seq<A>(self, mut seq: A) -> std::result::Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut values = Vec::new();
        while let Some(value) = seq.next_element::<DefValue>()? {
            values.push(value);
        }
        Ok(DefValue::Seq(values))
    }

    fn visit_map<A>(self, mut map: A) -> std::result::Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut enum_variant = None::<String>;
        let mut enum_payload = None::<DefValue>;
        let mut values = Vec::new();
        while let Some((key, value)) = map.next_entry::<DefValue, DefValue>()? {
            if let DefValue::String(key_str) = &key {
                if key_str == DEF_VALUE_ENUM_VARIANT_KEY {
                    let DefValue::String(variant) = value else {
                        return Err(serde::de::Error::custom(
                            "invalid enum variant in DefValue map",
                        ));
                    };
                    enum_variant = Some(variant);
                    continue;
                }
                if key_str == DEF_VALUE_ENUM_PAYLOAD_KEY {
                    enum_payload = Some(value);
                    continue;
                }
            }
            values.push((key, value));
        }
        if let Some(variant) = enum_variant {
            let payload = enum_payload.unwrap_or(DefValue::Unit);
            return Ok(DefValue::Enum(variant, Box::new(payload)));
        }
        Ok(DefValue::Map(values))
    }

    fn visit_enum<A>(self, data: A) -> std::result::Result<Self::Value, A::Error>
    where
        A: EnumAccess<'de>,
    {
        let (variant, variant_access) = data.variant::<String>()?;
        let payload = match variant_access.newtype_variant::<DefValue>() {
            Ok(value) => value,
            Err(_) => DefValue::Unit,
        };
        Ok(DefValue::Enum(variant, Box::new(payload)))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum DefSourceKind {
    Base,
    Mod,
}

#[derive(Debug, Clone)]
pub struct DefSource {
    pub kind: DefSourceKind,
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
    pub value: DefValue,
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

fn assets_index_cache() -> &'static Mutex<Option<Vec<(DefSource, PathBuf)>>> {
    static CACHE: OnceLock<Mutex<Option<Vec<(DefSource, PathBuf)>>>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(None))
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum DefPatchOp {
    Upsert {
        #[serde(alias = "type")]
        def_type: String,
        id: Option<String>,
        value: DefValue,
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
        value: DefValue,
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
        value: DefValue,
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

#[derive(Debug, Clone, Deserialize)]
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
    by_id: HashMap<String, DefRecord<T>>,
    ordered_ids: Vec<String>,
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
        self.ordered_ids.iter().filter_map(|id| self.by_id.get(id))
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
    pub fn load_from_assets_dir(suffixes: &[&str], id_of: impl Fn(&T) -> &str) -> Result<Self> {
        let fallback_type = std::any::type_name::<T>().to_string();
        Self::load_from_assets_dir_with_type(&fallback_type, suffixes, id_of)
    }

    pub fn load_from_assets_dir_with_type(
        def_type: &str,
        suffixes: &[&str],
        id_of: impl Fn(&T) -> &str,
    ) -> Result<Self> {
        let discovered = discover_assets_files_by_suffixes(suffixes)?;
        Self::load_from_sources(def_type, discovered, id_of)
    }

    pub fn resolve_typed_ref(def_type: &str, id: &str) -> Result<Option<T>> {
        let Some(value) = resolve_def_ref(def_type, id).map(|record| record.value) else {
            return Ok(None);
        };
        Ok(Some(def_value_into_typed(value).with_context(|| {
            format!("Failed to deserialize resolved ref '{}:{}'", def_type, id)
        })?))
    }

    pub fn load_from_sources(
        def_type: &str,
        mut discovered: Vec<(DefSource, PathBuf)>,
        _id_of: impl Fn(&T) -> &str,
    ) -> Result<Self> {
        discovered.sort_by(|(a, _), (b, _)| {
            a.precedence_rank()
                .cmp(&b.precedence_rank())
                .then_with(|| a.rel_path.cmp(&b.rel_path))
        });

        let mut raw_by_id = HashMap::default();
        let mut ordered_ids = Vec::new();
        let mut overrides = Vec::new();

        for (source, abs_path) in discovered {
            let content = std::fs::read_to_string(&abs_path).with_context(|| {
                format!("Failed reading def file '{}'", abs_path.to_string_lossy())
            })?;
            let parsed = parse_def_values(&content).with_context(|| {
                format!("Failed parsing RON def file '{}'", abs_path.to_string_lossy())
            })?;
            for mut raw_value in parsed.into_vec() {
                let Some(id) = extract_id(&raw_value) else {
                    warn!(
                        "Skipping '{}' def in '{}' with missing/invalid id",
                        def_type, source.rel_path
                    );
                    continue;
                };
                ensure_id_field(&mut raw_value, &id);
                let record = DefRawRecord {
                    id: id.clone(),
                    source: source.clone(),
                    value: raw_value,
                };
                if let Some(previous) = raw_by_id.insert(id.clone(), record) {
                    overrides.push(DefOverride {
                        id,
                        previous_source: previous.source,
                        replacement_source: source.clone(),
                    });
                } else {
                    ordered_ids.push(id);
                }
            }
        }

        let mut applied = Vec::new();
        apply_patch_files(def_type, &mut raw_by_id, &mut ordered_ids, &mut applied)?;

        let mut db = DefDatabase::<T> {
            by_id: HashMap::default(),
            ordered_ids,
            overrides,
        };
        let mut registry_type = RegisteredDefType::default();

        for id in &db.ordered_ids {
            let Some(record) = raw_by_id.get(id) else { continue };
            let value = def_value_into_typed::<T>(record.value.clone()).with_context(|| {
                format!(
                    "Failed deserializing patched '{}:{}' from '{}'",
                    def_type, id, record.source.rel_path
                )
            })?;
            registry_type.ordered_ids.push(id.clone());
            registry_type.by_id.insert(id.clone(), record.clone());
            db.by_id.insert(
                id.clone(),
                DefRecord {
                    id: id.clone(),
                    source: record.source.clone(),
                    value,
                },
            );
        }

        if let Ok(mut registry) = global_registry().lock() {
            registry.by_type.insert(def_type.to_string(), registry_type);
            registry.patches.extend(applied);
        }

        Ok(db)
    }
}

fn def_value_into_typed<T: DeserializeOwned>(value: DefValue) -> Result<T> {
    T::deserialize(value).map_err(Into::into)
}

impl<'de> Deserializer<'de> for DefValue {
    type Error = ValueDeError;

    fn deserialize_any<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self {
            DefValue::Bool(v) => visitor.visit_bool(v),
            DefValue::Char(v) => visitor.visit_char(v),
            DefValue::String(v) => visitor.visit_string(v),
            DefValue::Bytes(v) => visitor.visit_byte_buf(v),
            DefValue::I64(v) => visitor.visit_i64(v),
            DefValue::U64(v) => visitor.visit_u64(v),
            DefValue::F64(v) => visitor.visit_f64(v),
            DefValue::Option(Some(v)) => visitor.visit_some(*v),
            DefValue::Option(None) => visitor.visit_none(),
            DefValue::Seq(values) => visitor.visit_seq(DefValueSeqAccess {
                iter: values.into_iter(),
            }),
            DefValue::Map(entries) => visitor.visit_map(DefValueMapAccess {
                iter: entries.into_iter(),
                pending_value: None,
            }),
            DefValue::Unit => visitor.visit_unit(),
            DefValue::Enum(variant, payload) => {
                visitor.visit_enum(DefValueEnumAccess { variant, payload: *payload })
            }
        }
    }

    fn deserialize_option<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self {
            DefValue::Option(Some(v)) => visitor.visit_some(*v),
            DefValue::Option(None) => visitor.visit_none(),
            other => visitor.visit_some(other),
        }
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> std::result::Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self {
            DefValue::Enum(variant, payload) => {
                visitor.visit_enum(DefValueEnumAccess { variant, payload: *payload })
            }
            DefValue::String(variant) => visitor.visit_enum(DefValueEnumAccess {
                variant,
                payload: DefValue::Unit,
            }),
            DefValue::Map(mut entries) => {
                if entries.len() == 1 {
                    let (key, payload) = entries.remove(0);
                    if let DefValue::String(variant) = key {
                        return visitor.visit_enum(DefValueEnumAccess { variant, payload });
                    }
                }
                Err(serde::de::Error::custom(
                    "expected enum map with a single string key",
                ))
            }
            other => Err(serde::de::Error::custom(format!(
                "expected enum DefValue, got {other:?}"
            ))),
        }
    }

    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char str string bytes byte_buf
        unit unit_struct newtype_struct seq tuple tuple_struct map struct identifier
        ignored_any
    }
}

struct DefValueSeqAccess {
    iter: std::vec::IntoIter<DefValue>,
}

impl<'de> SeqAccess<'de> for DefValueSeqAccess {
    type Error = ValueDeError;

    fn next_element_seed<T>(
        &mut self,
        seed: T,
    ) -> std::result::Result<Option<T::Value>, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        let Some(value) = self.iter.next() else {
            return Ok(None);
        };
        seed.deserialize(value).map(Some)
    }

    fn size_hint(&self) -> Option<usize> {
        Some(self.iter.len())
    }
}

struct DefValueMapAccess {
    iter: std::vec::IntoIter<(DefValue, DefValue)>,
    pending_value: Option<DefValue>,
}

impl<'de> MapAccess<'de> for DefValueMapAccess {
    type Error = ValueDeError;

    fn next_key_seed<K>(&mut self, seed: K) -> std::result::Result<Option<K::Value>, Self::Error>
    where
        K: DeserializeSeed<'de>,
    {
        let Some((key, value)) = self.iter.next() else {
            return Ok(None);
        };
        self.pending_value = Some(value);
        seed.deserialize(key).map(Some)
    }

    fn next_value_seed<V>(
        &mut self,
        seed: V,
    ) -> std::result::Result<V::Value, Self::Error>
    where
        V: DeserializeSeed<'de>,
    {
        let Some(value) = self.pending_value.take() else {
            return Err(serde::de::Error::custom("missing value for map key"));
        };
        seed.deserialize(value)
    }

    fn size_hint(&self) -> Option<usize> {
        Some(self.iter.len())
    }
}

struct DefValueEnumAccess {
    variant: String,
    payload: DefValue,
}

impl<'de> EnumAccess<'de> for DefValueEnumAccess {
    type Error = ValueDeError;
    type Variant = DefValueVariantAccess;

    fn variant_seed<V>(
        self,
        seed: V,
    ) -> std::result::Result<(V::Value, Self::Variant), Self::Error>
    where
        V: DeserializeSeed<'de>,
    {
        let variant = seed.deserialize(self.variant.into_deserializer())?;
        Ok((
            variant,
            DefValueVariantAccess {
                payload: self.payload,
            },
        ))
    }
}

struct DefValueVariantAccess {
    payload: DefValue,
}

impl<'de> VariantAccess<'de> for DefValueVariantAccess {
    type Error = ValueDeError;

    fn unit_variant(self) -> std::result::Result<(), Self::Error> {
        match self.payload {
            DefValue::Unit => Ok(()),
            other => Err(serde::de::Error::custom(format!(
                "expected unit variant payload, got {other:?}"
            ))),
        }
    }

    fn newtype_variant_seed<T>(self, seed: T) -> std::result::Result<T::Value, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        seed.deserialize(self.payload)
    }

    fn tuple_variant<V>(
        self,
        _len: usize,
        visitor: V,
    ) -> std::result::Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.payload {
            DefValue::Seq(values) => visitor.visit_seq(DefValueSeqAccess {
                iter: values.into_iter(),
            }),
            other => Err(serde::de::Error::custom(format!(
                "expected tuple variant payload sequence, got {other:?}"
            ))),
        }
    }

    fn struct_variant<V>(
        self,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> std::result::Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.payload {
            DefValue::Map(entries) => visitor.visit_map(DefValueMapAccess {
                iter: entries.into_iter(),
                pending_value: None,
            }),
            other => Err(serde::de::Error::custom(format!(
                "expected struct variant payload map, got {other:?}"
            ))),
        }
    }
}

fn parse_typed_defs<T: DeserializeOwned>(content: &str) -> Result<OneOrMany<T>> {
    if let Ok(parsed) = ron::from_str::<OneOrMany<T>>(content) {
        return Ok(parsed);
    }
    let Some(stripped) = strip_outer_named_wrapper(content) else {
        return ron::from_str::<OneOrMany<T>>(content).map_err(Into::into);
    };
    ron::from_str::<OneOrMany<T>>(&stripped).map_err(Into::into)
}

fn parse_def_values(content: &str) -> Result<OneOrMany<DefValue>> {
    if let Some(stripped) = strip_outer_named_wrapper(content) {
        if let Ok(parsed) = ron::from_str::<OneOrMany<DefValue>>(&stripped) {
            return Ok(parsed);
        }
    }
    if let Ok(parsed) = ron::from_str::<OneOrMany<DefValue>>(content) {
        return Ok(parsed);
    }
    let Some(stripped) = strip_outer_named_wrapper(content) else {
        return ron::from_str::<OneOrMany<DefValue>>(content).map_err(Into::into);
    };
    ron::from_str::<OneOrMany<DefValue>>(&stripped).map_err(Into::into)
}

fn strip_outer_named_wrapper(content: &str) -> Option<String> {
    let start = content.find(|c: char| !c.is_whitespace())?;
    let rest = &content[start..];
    let ident_len = rest
        .chars()
        .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
        .count();
    if ident_len == 0 {
        return None;
    }
    let after_ident = &rest[ident_len..];
    let open_rel = after_ident.find(|c: char| !c.is_whitespace())?;
    let open_abs = start + ident_len + open_rel;
    if content[open_abs..].chars().next()? != '(' {
        return None;
    }
    let close_abs = content.rfind(')')?;
    if close_abs <= open_abs {
        return None;
    }
    let mut out = String::with_capacity(content.len().saturating_sub(ident_len));
    out.push_str(&content[..start]);
    out.push('(');
    out.push_str(&content[open_abs + 1..close_abs]);
    out.push(')');
    out.push_str(&content[close_abs + 1..]);
    Some(out)
}

pub fn global_registry_snapshot() -> GlobalDefRegistry {
    global_registry().lock().map(|g| g.clone()).unwrap_or_default()
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

pub fn resolve_def_field(def_type: &str, id: &str, path: &str) -> Option<DefValue> {
    let record = resolve_def_ref(def_type, id)?;
    resolve_path_value(&record.value, path).cloned()
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
        let ops = match parse_typed_defs::<DefPatchOp>(&content) {
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
        DefPatchOp::Upsert {
            def_type: patch_type,
            id,
            mut value,
        } => {
            if patch_type != def_type {
                return;
            }
            let def_id = id.or_else(|| extract_id(&value));
            let Some(def_id) = def_id else { return };
            ensure_id_field(&mut value, &def_id);
            let record = DefRawRecord {
                id: def_id.clone(),
                source: DefSource {
                    kind: DefSourceKind::Mod,
                    rel_path: source_rel_path.to_string(),
                },
                value,
            };
            if raw_by_id.insert(def_id.clone(), record).is_none() {
                ordered_ids.push(def_id.clone());
            }
            applied.push(AppliedPatch {
                def_type: patch_type,
                id: def_id,
                op: "upsert".to_string(),
                source: source_rel_path.to_string(),
            });
        }
        DefPatchOp::Delete { def_type: patch_type, id } => {
            if patch_type != def_type {
                return;
            }
            raw_by_id.remove(&id);
            ordered_ids.retain(|entry| entry != &id);
            applied.push(AppliedPatch {
                def_type: patch_type,
                id,
                op: "delete".to_string(),
                source: source_rel_path.to_string(),
            });
        }
        DefPatchOp::SetField {
            def_type: patch_type,
            id,
            path,
            value,
        } => {
            if patch_type != def_type {
                return;
            }
            let Some(record) = raw_by_id.get_mut(&id) else { return };
            let Ok(tokens) = parse_path(&path) else { return };
            set_path_value(&mut record.value, &tokens, value);
            applied.push(AppliedPatch {
                def_type: patch_type,
                id,
                op: "set_field".to_string(),
                source: source_rel_path.to_string(),
            });
        }
        DefPatchOp::RemoveField {
            def_type: patch_type,
            id,
            path,
        } => {
            if patch_type != def_type {
                return;
            }
            let Some(record) = raw_by_id.get_mut(&id) else { return };
            let Ok(tokens) = parse_path(&path) else { return };
            remove_path_value(&mut record.value, &tokens);
            applied.push(AppliedPatch {
                def_type: patch_type,
                id,
                op: "remove_field".to_string(),
                source: source_rel_path.to_string(),
            });
        }
        DefPatchOp::Merge {
            def_type: patch_type,
            id,
            value,
        } => {
            if patch_type != def_type {
                return;
            }
            let Some(record) = raw_by_id.get_mut(&id) else { return };
            merge_values(&mut record.value, value);
            applied.push(AppliedPatch {
                def_type: patch_type,
                id,
                op: "merge".to_string(),
                source: source_rel_path.to_string(),
            });
        }
        DefPatchOp::Copy {
            def_type: patch_type,
            from_id,
            to_id,
            overwrite,
        } => {
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
            applied.push(AppliedPatch {
                def_type: patch_type,
                id: to_id,
                op: "copy".to_string(),
                source: source_rel_path.to_string(),
            });
        }
    }
}

fn ensure_id_field(value: &mut DefValue, id: &str) {
    let Some(entries) = map_entries_mut(value) else {
        return;
    };
    if let Some((_, value)) = find_string_key_mut(entries, "id") {
        *value = DefValue::String(id.to_string());
        return;
    }
    entries.push((DefValue::String("id".to_string()), DefValue::String(id.to_string())));
}

fn extract_id(value: &DefValue) -> Option<String> {
    let entries = map_entries(value)?;
    let (_, DefValue::String(id)) = find_string_key(entries, "id")? else { return None };
    if id.trim().is_empty() {
        return None;
    }
    Some(id.trim().to_string())
}

fn map_entries(value: &DefValue) -> Option<&[(DefValue, DefValue)]> {
    match value {
        DefValue::Map(entries) => Some(entries),
        DefValue::Enum(_, payload) => map_entries(payload),
        _ => None,
    }
}

fn map_entries_mut(value: &mut DefValue) -> Option<&mut Vec<(DefValue, DefValue)>> {
    match value {
        DefValue::Map(entries) => Some(entries),
        DefValue::Enum(_, payload) => map_entries_mut(payload),
        _ => None,
    }
}

fn unwrap_enum_payload_ref(mut value: &DefValue) -> &DefValue {
    while let DefValue::Enum(_, payload) = value {
        value = payload;
    }
    value
}

fn unwrap_enum_payload_mut(mut value: &mut DefValue) -> &mut DefValue {
    while let DefValue::Enum(_, payload) = value {
        value = payload;
    }
    value
}

fn find_string_key<'a>(
    entries: &'a [(DefValue, DefValue)],
    key: &str,
) -> Option<&'a (DefValue, DefValue)> {
    entries
        .iter()
        .find(|(entry_key, _)| matches!(entry_key, DefValue::String(s) if s == key))
}

fn find_string_key_mut<'a>(
    entries: &'a mut [(DefValue, DefValue)],
    key: &str,
) -> Option<&'a mut (DefValue, DefValue)> {
    entries
        .iter_mut()
        .find(|(entry_key, _)| matches!(entry_key, DefValue::String(s) if s == key))
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
            let idx_str = segment
                .trim_start_matches(&format!("{}[", key))
                .trim_end_matches(']');
            let index = idx_str
                .parse::<usize>()
                .with_context(|| format!("Invalid index in path '{}'", path))?;
            out.push(PathToken::Index(index));
        } else {
            out.push(PathToken::Key(segment.to_string()));
        }
    }
    Ok(out)
}

fn set_path_value(root: &mut DefValue, tokens: &[PathToken], value: DefValue) {
    if tokens.is_empty() {
        *root = value;
        return;
    }
    let mut cursor = unwrap_enum_payload_mut(root);
    for token in &tokens[..tokens.len() - 1] {
        cursor = unwrap_enum_payload_mut(cursor);
        match token {
            PathToken::Key(key) => {
                if !matches!(cursor, DefValue::Map(_)) {
                    *cursor = DefValue::Map(Vec::new());
                }
                let DefValue::Map(entries) = cursor else { return };
                if find_string_key(entries, key).is_none() {
                    entries.push((DefValue::String(key.clone()), DefValue::Map(Vec::new())));
                }
                let Some((_, next)) = find_string_key_mut(entries, key) else { return };
                cursor = next;
            }
            PathToken::Index(index) => {
                if !matches!(cursor, DefValue::Seq(_)) {
                    *cursor = DefValue::Seq(Vec::new());
                }
                let DefValue::Seq(seq) = cursor else { return };
                while seq.len() <= *index {
                    seq.push(DefValue::Unit);
                }
                cursor = &mut seq[*index];
            }
        }
    }
    cursor = unwrap_enum_payload_mut(cursor);
    match tokens.last() {
        Some(PathToken::Key(key)) => {
            if !matches!(cursor, DefValue::Map(_)) {
                *cursor = DefValue::Map(Vec::new());
            }
            let DefValue::Map(entries) = cursor else { return };
            if let Some((_, existing)) = find_string_key_mut(entries, key) {
                *existing = value;
            } else {
                entries.push((DefValue::String(key.clone()), value));
            }
        }
        Some(PathToken::Index(index)) => {
            if !matches!(cursor, DefValue::Seq(_)) {
                *cursor = DefValue::Seq(Vec::new());
            }
            let DefValue::Seq(seq) = cursor else { return };
            while seq.len() <= *index {
                seq.push(DefValue::Unit);
            }
            seq[*index] = value;
        }
        None => {}
    }
}

fn remove_path_value(root: &mut DefValue, tokens: &[PathToken]) {
    if tokens.is_empty() {
        return;
    }
    let mut cursor = unwrap_enum_payload_mut(root);
    for token in &tokens[..tokens.len() - 1] {
        cursor = unwrap_enum_payload_mut(cursor);
        match token {
            PathToken::Key(key) => {
                let DefValue::Map(entries) = cursor else { return };
                let Some((_, next)) = find_string_key_mut(entries, key) else { return };
                cursor = next;
            }
            PathToken::Index(index) => {
                let DefValue::Seq(seq) = cursor else { return };
                let Some(next) = seq.get_mut(*index) else { return };
                cursor = next;
            }
        }
    }
    cursor = unwrap_enum_payload_mut(cursor);
    match tokens.last() {
        Some(PathToken::Key(key)) => {
            let DefValue::Map(entries) = cursor else { return };
            entries.retain(|(entry_key, _)| !matches!(entry_key, DefValue::String(s) if s == key));
        }
        Some(PathToken::Index(index)) => {
            let DefValue::Seq(seq) = cursor else { return };
            if *index < seq.len() {
                seq.remove(*index);
            }
        }
        None => {}
    }
}

fn merge_values(target: &mut DefValue, patch: DefValue) {
    if let DefValue::Enum(_, target_payload) = target {
        merge_values(target_payload, patch);
        return;
    }
    if let DefValue::Enum(_, patch_payload) = patch {
        merge_values(target, *patch_payload);
        return;
    }
    match (target, patch) {
        (DefValue::Map(target_entries), DefValue::Map(patch_entries)) => {
            for (patch_key, patch_value) in patch_entries {
                if let Some((_, target_value)) =
                    target_entries.iter_mut().find(|(target_key, _)| target_key == &patch_key)
                {
                    merge_values(target_value, patch_value);
                } else {
                    target_entries.push((patch_key, patch_value));
                }
            }
        }
        (DefValue::Seq(target_seq), DefValue::Seq(patch_seq)) => {
            target_seq.extend(patch_seq);
        }
        (target_value, patch_value) => {
            *target_value = patch_value;
        }
    }
}

fn resolve_path_value<'a>(root: &'a DefValue, path: &str) -> Option<&'a DefValue> {
    let tokens = parse_path(path).ok()?;
    let mut cursor = unwrap_enum_payload_ref(root);
    for token in tokens {
        cursor = unwrap_enum_payload_ref(cursor);
        match token {
            PathToken::Key(key) => {
                let DefValue::Map(entries) = cursor else { return None };
                cursor = &find_string_key(entries, &key)?.1;
            }
            PathToken::Index(index) => {
                let DefValue::Seq(seq) = cursor else { return None };
                cursor = seq.get(index)?;
            }
        }
    }
    Some(cursor)
}

fn extract_ref_ids(value: &DefValue) -> Vec<String> {
    match unwrap_enum_payload_ref(value) {
        DefValue::String(id) => vec![id.clone()],
        DefValue::Seq(values) => values.iter().flat_map(extract_ref_ids).collect(),
        DefValue::Map(entries) => {
            let Some((_, DefValue::String(id))) = find_string_key(entries, "id") else {
                return Vec::new();
            };
            vec![id.clone()]
        }
        _ => Vec::new(),
    }
}

pub fn discover_assets_files_by_suffixes(suffixes: &[&str]) -> Result<Vec<(DefSource, PathBuf)>> {
    let assets_root = Path::new("assets");
    if !assets_root.exists() {
        return Ok(Vec::new());
    }
    let all_files = get_or_build_assets_index(assets_root)?;
    let mut out = Vec::new();
    for (source, abs_path) in all_files {
        if suffixes.iter().any(|suffix| source.rel_path.ends_with(suffix)) {
            out.push((source.clone(), abs_path.clone()));
        }
    }
    Ok(out)
}

pub fn discover_assets_files_matching(
    assets_root: &Path,
    matches: impl Fn(&str) -> bool,
) -> Result<Vec<(DefSource, PathBuf)>> {
    if !assets_root.exists() {
        return Ok(Vec::new());
    }

    let mut stack = vec![assets_root.to_path_buf()];
    let mut out = Vec::new();

    while let Some(dir) = stack.pop() {
        let entries = std::fs::read_dir(&dir)
            .with_context(|| format!("Failed reading directory '{}'", dir.to_string_lossy()))?;
        for entry in entries.flatten() {
            let path = entry.path();
            let meta = match entry.metadata() {
                Ok(meta) => meta,
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
            out.push((DefSource { kind, rel_path: rel_string }, path));
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

fn get_or_build_assets_index(assets_root: &Path) -> Result<Vec<(DefSource, PathBuf)>> {
    if let Ok(cache) = assets_index_cache().lock()
        && let Some(files) = &*cache
    {
        return Ok(files.clone());
    }
    let files = discover_assets_files_matching(assets_root, |_| true)?;
    if let Ok(mut cache) = assets_index_cache().lock() {
        *cache = Some(files.clone());
    }
    Ok(files)
}
