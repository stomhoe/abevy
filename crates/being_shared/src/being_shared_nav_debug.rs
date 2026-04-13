use bevy::ecs::{
    entity::EntityHashMap,
    message::Message,
};
use bevy::prelude::*;
use std::collections::VecDeque;

use tilemap_shared::GlobalTilePos;
use tilemap_shared::ChunkPos;

pub const MAX_TRACKED_BEING_NAV_COLUMNS: usize = 4;
pub const MAX_TRACKED_BEING_NAV_LOG_LINES: usize = 256;

#[derive(Debug, Clone, Message)]
pub struct BeingNavDebugLine {
    pub being_ent: Entity,
    pub timestamp_secs: f64,
    pub system: String,
    pub kind: BeingNavDebugKind,
    pub summary: String,
    pub fields: Vec<BeingNavDebugField>,
}

impl BeingNavDebugLine {
    pub fn new(
        being_ent: Entity,
        timestamp_secs: f64,
        system: impl Into<String>,
        kind: BeingNavDebugKind,
        summary: impl Into<String>,
    ) -> Self {
        Self {
            being_ent,
            timestamp_secs,
            system: system.into(),
            kind,
            summary: summary.into(),
            fields: Vec::new(),
        }
    }

    pub fn with_fields(mut self, fields: Vec<BeingNavDebugField>) -> Self {
        self.fields = fields;
        self
    }
}

#[derive(Debug, Clone)]
pub enum BeingNavDebugKind {
    State,
    Decision,
    Repath,
    Path,
    Target,
    Clear,
    Track,
    Info,
    Other(String),
}

#[derive(Debug, Clone)]
pub struct BeingNavDebugField {
    pub key: String,
    pub value: BeingNavDebugValue,
}

impl BeingNavDebugField {
    pub fn new(key: impl Into<String>, value: impl Into<BeingNavDebugValue>) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum BeingNavDebugValue {
    Bool(bool),
    I32(i32),
    I64(i64),
    U32(u32),
    U64(u64),
    F32(f32),
    F64(f64),
    Text(String),
    Entity(Entity),
    GPos(GlobalTilePos),
    Chunk(ChunkPos),
    EntityList(Vec<Entity>),
    GPosList(Vec<GlobalTilePos>),
    MaybeEntity(Option<Entity>),
    MaybeGPos(Option<GlobalTilePos>),
    MaybeI32(Option<i32>),
    MaybeU32(Option<u32>),
    MaybeF32(Option<f32>),
    MaybeText(Option<String>),
}

impl From<bool> for BeingNavDebugValue {
    fn from(value: bool) -> Self {
        Self::Bool(value)
    }
}

impl From<i32> for BeingNavDebugValue {
    fn from(value: i32) -> Self {
        Self::I32(value)
    }
}

impl From<i64> for BeingNavDebugValue {
    fn from(value: i64) -> Self {
        Self::I64(value)
    }
}

impl From<u32> for BeingNavDebugValue {
    fn from(value: u32) -> Self {
        Self::U32(value)
    }
}

impl From<u8> for BeingNavDebugValue {
    fn from(value: u8) -> Self {
        Self::U32(value as u32)
    }
}

impl From<u64> for BeingNavDebugValue {
    fn from(value: u64) -> Self {
        Self::U64(value)
    }
}

impl From<f32> for BeingNavDebugValue {
    fn from(value: f32) -> Self {
        Self::F32(value)
    }
}

impl From<f64> for BeingNavDebugValue {
    fn from(value: f64) -> Self {
        Self::F64(value)
    }
}

impl From<String> for BeingNavDebugValue {
    fn from(value: String) -> Self {
        Self::Text(value)
    }
}

impl From<&str> for BeingNavDebugValue {
    fn from(value: &str) -> Self {
        Self::Text(value.to_string())
    }
}

impl From<Entity> for BeingNavDebugValue {
    fn from(value: Entity) -> Self {
        Self::Entity(value)
    }
}

impl From<GlobalTilePos> for BeingNavDebugValue {
    fn from(value: GlobalTilePos) -> Self {
        Self::GPos(value)
    }
}

impl From<ChunkPos> for BeingNavDebugValue {
    fn from(value: ChunkPos) -> Self {
        Self::Chunk(value)
    }
}

impl From<Vec<Entity>> for BeingNavDebugValue {
    fn from(value: Vec<Entity>) -> Self {
        Self::EntityList(value)
    }
}

impl From<Vec<GlobalTilePos>> for BeingNavDebugValue {
    fn from(value: Vec<GlobalTilePos>) -> Self {
        Self::GPosList(value)
    }
}

impl From<Option<Entity>> for BeingNavDebugValue {
    fn from(value: Option<Entity>) -> Self {
        Self::MaybeEntity(value)
    }
}

impl From<Option<GlobalTilePos>> for BeingNavDebugValue {
    fn from(value: Option<GlobalTilePos>) -> Self {
        Self::MaybeGPos(value)
    }
}

impl From<Option<i32>> for BeingNavDebugValue {
    fn from(value: Option<i32>) -> Self {
        Self::MaybeI32(value)
    }
}

impl From<Option<u32>> for BeingNavDebugValue {
    fn from(value: Option<u32>) -> Self {
        Self::MaybeU32(value)
    }
}

impl From<Option<f32>> for BeingNavDebugValue {
    fn from(value: Option<f32>) -> Self {
        Self::MaybeF32(value)
    }
}

impl From<Option<String>> for BeingNavDebugValue {
    fn from(value: Option<String>) -> Self {
        Self::MaybeText(value)
    }
}

#[derive(Debug, Clone, Default)]
pub struct BeingNavLogColumn {
    pub lines: VecDeque<BeingNavDebugLine>,
    pub paused: bool,
}

#[derive(Resource, Debug, Default)]
pub struct DebuggingBeingNav {
    pub tracked_beings: Vec<Entity>,
    pub columns: EntityHashMap<BeingNavLogColumn>,
    pub pause_all: bool,
}

impl DebuggingBeingNav {
    pub fn is_tracked(&self, being_ent: Entity) -> bool {
        self.tracked_beings.contains(&being_ent)
    }

    pub fn tracked_count(&self) -> usize {
        self.tracked_beings.len()
    }

    pub fn ensure_column(&mut self, being_ent: Entity) -> &mut BeingNavLogColumn {
        self.columns.entry(being_ent).or_default()
    }

    pub fn track_being(&mut self, being_ent: Entity) -> bool {
        if self.is_tracked(being_ent) || self.tracked_beings.len() >= MAX_TRACKED_BEING_NAV_COLUMNS {
            return false;
        }
        self.tracked_beings.push(being_ent);
        self.ensure_column(being_ent);
        true
    }

    pub fn remove_being(&mut self, being_ent: Entity) -> bool {
        let Some(ix) = self.tracked_beings.iter().position(|&tracked| tracked == being_ent) else {
            return false;
        };
        self.tracked_beings.remove(ix);
        self.columns.remove(&being_ent);
        true
    }

    pub fn move_left(&mut self, being_ent: Entity) -> bool {
        let Some(ix) = self.tracked_beings.iter().position(|&tracked| tracked == being_ent) else {
            return false;
        };
        if ix == 0 {
            return false;
        }
        self.tracked_beings.swap(ix, ix - 1);
        true
    }

    pub fn move_right(&mut self, being_ent: Entity) -> bool {
        let Some(ix) = self.tracked_beings.iter().position(|&tracked| tracked == being_ent) else {
            return false;
        };
        if ix + 1 >= self.tracked_beings.len() {
            return false;
        }
        self.tracked_beings.swap(ix, ix + 1);
        true
    }

    pub fn clear_column(&mut self, being_ent: Entity) -> bool {
        let Some(column) = self.columns.get_mut(&being_ent) else {
            return false;
        };
        column.lines.clear();
        true
    }

    pub fn is_column_paused(&self, being_ent: Entity) -> bool {
        self.pause_all || self.columns.get(&being_ent).is_some_and(|column| column.paused)
    }

    pub fn toggle_column_pause(&mut self, being_ent: Entity) -> bool {
        let Some(column) = self.columns.get_mut(&being_ent) else {
            return false;
        };
        column.paused = !column.paused;
        true
    }

    pub fn toggle_pause_all(&mut self) {
        self.pause_all = !self.pause_all;
    }

    pub fn clear_all(&mut self) {
        self.tracked_beings.clear();
        self.columns.clear();
        self.pause_all = false;
    }

    pub fn push_line(&mut self, line: BeingNavDebugLine) {
        if self.is_column_paused(line.being_ent) {
            return;
        }
        let column = self.ensure_column(line.being_ent);
        column.lines.push_back(line);
        while column.lines.len() > MAX_TRACKED_BEING_NAV_LOG_LINES {
            column.lines.pop_front();
        }
    }
}
