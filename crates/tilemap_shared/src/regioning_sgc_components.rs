use bevy::{ecs::entity::MapEntities, platform::collections::*, prelude::*};
use bevy_replicon::prelude::*;
use ::common::*;
use serde::{Deserialize, Serialize};
use crate::tilemap_shared::*;

define_entity_map_systems!(
    main_component: StructuredGenConfig,
    with_filters: (),
    abbreviation: Sgc,
    target: "sgc",
    entity_prefix: "SGC",
    despawn_trigger: StructuredGenConfig,
    id_type: StrId,
);


#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SgcArgValue {
    Str(String),
    Bool(bool),
    Int(i64),
    Float(f64),
    List(Vec<SgcArgValue>),
    Map(HashMap<String, SgcArgValue>),
    Null,
}

impl Default for SgcArgValue {
    fn default() -> Self {
        Self::Null
    }
}

impl SgcArgValue {
    pub fn as_map(&self) -> Option<&HashMap<String, SgcArgValue>> {
        let Self::Map(map) = self else { return None; };
        Some(map)
    }

    pub fn as_list(&self) -> Option<&[SgcArgValue]> {
        let Self::List(list) = self else { return None; };
        Some(list.as_slice())
    }

    pub fn first(&self) -> Option<&str> {
        self.as_list()
            .and_then(|list| list.first())
            .and_then(SgcArgValue::as_str)
            .or_else(|| self.as_str())
    }

    pub fn as_str(&self) -> Option<&str> {
        let Self::Str(value) = self else { return None; };
        Some(value.as_str())
    }

    pub fn as_u8(&self) -> Option<u8> {
        match self {
            Self::Int(value) => u8::try_from(*value).ok(),
            _ => None,
        }
    }

    pub fn as_u16(&self) -> Option<u16> {
        match self {
            Self::Int(value) => u16::try_from(*value).ok(),
            _ => None,
        }
    }

    pub fn as_f32(&self) -> Option<f32> {
        match self {
            Self::Float(value) => Some(*value as f32),
            Self::Int(value) => Some(*value as f32),
            _ => None,
        }
    }

    pub fn as_scalar_string(&self) -> Option<String> {
        match self {
            Self::Str(value) => Some(value.clone()),
            Self::Bool(value) => Some(value.to_string()),
            Self::Int(value) => Some(value.to_string()),
            Self::Float(value) => Some(value.to_string()),
            Self::Null => None,
            Self::List(_) => None,
            Self::Map(_) => None,
        }
    }
}

#[derive(Component, Debug, Clone, Default, Serialize, Deserialize)]
pub struct SgcArgsDict(pub HashMap<String, SgcArgValue>);

pub type ArgsDict = SgcArgsDict;

impl SgcArgsDict {
    pub fn with_capacity(capacity: usize) -> Self {
        Self(HashMap::with_capacity(capacity))
    }

    pub fn iter(&self) -> impl Iterator<Item = (&String, &SgcArgValue)> {
        self.0.iter()
    }

    pub fn insert<T: Into<String>>(&mut self, key: T, value: SgcArgValue) {
        self.0.insert(key.into(), value);
    }

    pub fn get(&self, key: &str) -> Option<&SgcArgValue> {
        self.0.get(key)
    }

    pub fn get_string(&self, key: &str) -> Option<&str> {
        self.get(key).and_then(SgcArgValue::as_str)
    }

    pub fn get_map(&self, key: &str) -> Option<&HashMap<String, SgcArgValue>> {
        self.get(key).and_then(SgcArgValue::as_map)
    }

    pub fn parse_arg<T: std::str::FromStr + Clone>(&self, key: &str, default: T) -> T {
        self.parse_opt_arg(key).unwrap_or(default)
    }

    pub fn parse_opt_arg<T: std::str::FromStr>(&self, key: &str) -> Option<T> {
        let value = self.get(key)?;
        match value {
            SgcArgValue::Str(value) => value.parse::<T>().ok(),
            SgcArgValue::Bool(value) => value.to_string().parse::<T>().ok(),
            SgcArgValue::Int(value) => value.to_string().parse::<T>().ok(),
            SgcArgValue::Float(value) => value.to_string().parse::<T>().ok(),
            SgcArgValue::Null => None,
            SgcArgValue::List(list) => list
                .first()
                .and_then(SgcArgValue::as_scalar_string)
                .and_then(|value| value.parse::<T>().ok()),
            SgcArgValue::Map(_) => None,
        }
    }

    pub fn room_spawn_shape_keys(&self) -> HashSet<String> {
        let mut shapes = HashSet::default();
        let Some(room_spawn_map) = self.get_map("room_spawn") else {
            for key in self.0.keys() {
                let Some((_, rest)) = key.split_once("room_spawn.") else { continue; };
                let Some((shape, _)) = rest.split_once('.') else { continue; };
                if shape.is_empty() { continue; }
                shapes.insert(shape.to_string());
            }
            return shapes;
        };
        for shape in room_spawn_map.keys() {
            if shape.trim().is_empty() { continue; }
            shapes.insert(shape.to_string());
        }
        shapes
    }
}

#[derive(Component, Debug, Deserialize, Serialize, Clone)]
#[require(Replicated, Prefix::trunc("StructureGenerationSettings"), AssetScoped, SelectedForHotReload)]
pub struct StructureGenerationSettings {
    pub structure_build_timeout_secs: f64,
    pub claimlist_advance_timeout_secs: f32,
    pub region_offer_timeout_secs: f32,
    pub max_used_chunks_per_region_ratio: f32,
}

impl Default for StructureGenerationSettings {
    fn default() -> Self {
        Self {
            structure_build_timeout_secs: 4.0,
            claimlist_advance_timeout_secs: crate::DEFAULT_CLAIMLIST_ADVANCE_TIMEOUT_SECS,
            region_offer_timeout_secs: 2.0,
            max_used_chunks_per_region_ratio: 0.07,
        }
    }
}

#[derive(Component, Debug, Deserialize, Serialize, Clone)]
#[require(AssetScoped, Prefix::trunc("SGC"))]
pub struct StructuredGenConfig {
    structure_id: StrId,
    structure_hash_id: HashId,
    pub max_per_region: u32,
    pub max_being_count: Option<u32>,
    pub args: SgcArgsDict,
    pub typed_args: SgcArgsDict,
    pub whitelisted_tags: TagSet,
    pub blacklisted_tags: TagSet,
}

impl StructuredGenConfig {
    pub fn new<S: AsRef<str>>(structure_id: S) -> Self {
        Self {
            structure_id: StrId::trunc(structure_id.as_ref()),
            structure_hash_id: HashId::hash(structure_id.as_ref()),
            max_per_region: 1024,
            max_being_count: None,
            args: SgcArgsDict::default(),
            typed_args: SgcArgsDict::default(),
            whitelisted_tags: TagSet::default(),
            blacklisted_tags: TagSet::default(),
        }
    }
    pub fn structure_id(&self) -> &StrId {
        &self.structure_id
    }
    pub fn structure_hash_id(&self) -> HashId {
        self.structure_hash_id
    }
    pub fn tolerates_tags(&self, other_tags: &TagSet) -> bool {
        passes_tag_filters(Some(other_tags), Some(&self.whitelisted_tags), Some(&self.blacklisted_tags))
    }
}

#[derive(Component, Debug, Deserialize, Serialize, Copy, Clone, MapEntities)]
#[relationship(relationship_target = AcceptedFilters)]
pub struct WhitelistedFilterOf {
    #[relationship]
    #[entities]
    pub structured_gen_cfg: Entity,
}

impl WhitelistedFilterOf {
    pub fn new(structured_gen_cfg: Entity) -> Self {
        Self { structured_gen_cfg }
    }
}

#[derive(Component, Debug, Clone)]
#[relationship_target(relationship = WhitelistedFilterOf)]
pub struct AcceptedFilters(Vec<Entity>);

#[derive(Component, Debug, Default, Deserialize, Serialize, Copy, Clone)]
#[require(AssetScoped, Prefix::trunc("SGCsWeightedSampler"))]
pub struct SgcsWeightedSampler;

#[derive(Debug, Clone, Default)]
pub struct SgcCommandSchema {
    pub room_spawn_shapes: HashSet<String>,
}

#[derive(Resource, Debug, Clone, Default)]
pub struct SgcCommandRegistry(pub HashMap<String, SgcCommandSchema>);

impl SgcCommandRegistry {
    pub fn with_builtins() -> Self {
        let mut registry = Self::default();
        registry.register_room_spawn_shapes("chamberscorridors", ["rectangle", "circle", "triangle", "regular_polygon", "pentacle"]);
        registry.register_room_spawn_shapes("maze", ["square_room", "circle_room", "island_circle", "island_triangle", "island_hexagon", "island_square"]);
        registry.register_room_spawn_shapes("drunkwalk", ["chamber_circle"]);
        registry.register_room_spawn_shapes("spiral", ["center_circle", "arm_inner", "arm_outer"]);
        registry.register_room_spawn_shapes("archi", ["center_spiral"]);
        registry
    }

    pub fn register_room_spawn_shapes<S, I>(&mut self, structure_id: &str, room_shapes: I)
    where
        S: AsRef<str>,
        I: IntoIterator<Item = S>,
    {
        let schema = self.0.entry(structure_id.to_string()).or_default();
        for room_shape in room_shapes {
            schema.room_spawn_shapes.insert(room_shape.as_ref().to_string());
        }
    }

    pub fn allowed_room_spawn_shapes_for(&self, structure_id: &str) -> Option<&HashSet<String>> {
        self.0.get(structure_id).map(|schema| &schema.room_spawn_shapes)
    }
}
