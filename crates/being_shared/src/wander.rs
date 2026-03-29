use bevy::{platform::collections::{HashMap, HashSet}, prelude::*};
use serde::{Deserialize, Serialize};

use crate::being_shared_seris::{DEFAULT_AVOID_ENTITY_RADIUS, DEFAULT_AVOID_ENTITY_STRENGTH, WanderSeri};

#[derive(Component, Debug, Clone, Deserialize, Serialize)]
pub struct WanderConfig {
    pub dir_secs_min: f32,
    pub dir_secs_max: f32,
    pub move_secs_min: f32,
    pub move_secs_max: f32,
    pub halt_secs_min: f32,
    pub halt_secs_max: f32,
    pub speed_min: f32,
    pub speed_max: f32,
    #[serde(default)]
    pub avoid_tile_tags: HashSet<String>,
    #[serde(default)]
    pub avoid_bit_tags: HashSet<String>,
    #[serde(default)]
    pub avoid_race_tags: HashSet<String>,
    #[serde(default)]
    pub avoid_pack_tags: HashSet<String>,
    #[serde(default = "crate::being_shared_seris::default_nan")]
    pub max_drift: f32,
    #[serde(default)]
    pub pack_orbit_radius: f32,
    #[serde(default)]
    pub pack_orbit_retarget_secs_min: f32,
    #[serde(default)]
    pub pack_orbit_retarget_secs_max: f32,
    #[serde(default)]
    pub wander_around_leader: bool,
    #[serde(default)]
    pub avoid_entity_radius: HashMap<String, f32>,
    #[serde(default)]
    pub avoid_entity_strength: HashMap<String, f32>,
    #[serde(default)]
    pub avoid_blacklisted_spawn_tiles: bool,
}

impl Default for WanderConfig {
    fn default() -> Self {
        Self {
            dir_secs_min: 0.8,
            dir_secs_max: 2.4,
            move_secs_min: 0.9,
            move_secs_max: 2.4,
            halt_secs_min: 0.25,
            halt_secs_max: 1.4,
            speed_min: 0.2,
            speed_max: 0.6,
            avoid_tile_tags: HashSet::default(),
            avoid_bit_tags: HashSet::default(),
            avoid_race_tags: HashSet::default(),
            avoid_pack_tags: HashSet::default(),
            max_drift: f32::NAN,
            pack_orbit_radius: 0.0,
            pack_orbit_retarget_secs_min: 0.0,
            pack_orbit_retarget_secs_max: 0.0,
            wander_around_leader: false,
            avoid_entity_radius: HashMap::default(),
            avoid_entity_strength: HashMap::default(),
            avoid_blacklisted_spawn_tiles: false,
        }
    }
}

impl WanderConfig {
    pub fn from_seri(seri: &WanderSeri) -> Self {
        Self {
            dir_secs_min: seri.dir_secs_min,
            dir_secs_max: seri.dir_secs_max,
            move_secs_min: seri.move_secs_min,
            move_secs_max: seri.move_secs_max,
            halt_secs_min: seri.halt_secs_min,
            halt_secs_max: seri.halt_secs_max,
            speed_min: seri.speed_min,
            speed_max: seri.speed_max,
            avoid_tile_tags: seri.avoid_tile_tags.clone(),
            avoid_bit_tags: seri.avoid_bit_tags.clone(),
            avoid_race_tags: seri.avoid_race_tags.clone(),
            avoid_pack_tags: seri.avoid_pack_tags.clone(),
            max_drift: seri.max_drift,
            pack_orbit_radius: seri.pack_orbit_radius,
            pack_orbit_retarget_secs_min: seri.pack_orbit_retarget_secs_min,
            pack_orbit_retarget_secs_max: seri.pack_orbit_retarget_secs_max,
            wander_around_leader: seri.wander_around_leader,
            avoid_entity_radius: seri
                .avoid_entity_radius
                .iter()
                .map(|(tag, radius)| (tag.clone(), radius.max(0.0)))
                .collect(),
            avoid_entity_strength: seri
                .avoid_entity_strength
                .iter()
                .map(|(tag, strength)| (tag.clone(), strength.max(0.0)))
                .collect(),
            avoid_blacklisted_spawn_tiles: seri.avoid_blacklisted_spawn_tiles,
        }
        .sanitized()
    }

    pub fn sanitized(mut self) -> Self {
        self.dir_secs_min = self.dir_secs_min.max(0.01);
        self.dir_secs_max = self.dir_secs_max.max(self.dir_secs_min);
        self.move_secs_min = self.move_secs_min.max(0.01);
        self.move_secs_max = self.move_secs_max.max(self.move_secs_min);
        self.halt_secs_min = self.halt_secs_min.max(0.01);
        self.halt_secs_max = self.halt_secs_max.max(self.halt_secs_min);
        self.speed_min = self.speed_min.max(0.0);
        self.speed_max = self.speed_max.max(self.speed_min);
        self.pack_orbit_radius = self.pack_orbit_radius.max(0.0);
        self.pack_orbit_retarget_secs_min = self.pack_orbit_retarget_secs_min.max(0.0);
        self.pack_orbit_retarget_secs_max = self.pack_orbit_retarget_secs_max.max(self.pack_orbit_retarget_secs_min);
        self.avoid_entity_radius = self
            .avoid_entity_radius
            .into_iter()
            .map(|(tag, radius)| (tag, radius.max(0.0)))
            .collect();
        self.avoid_entity_strength = self
            .avoid_entity_strength
            .into_iter()
            .map(|(tag, strength)| (tag, strength.max(0.0)))
            .collect();
        self
    }

    pub fn is_disabled(&self) -> bool {
        self.dir_secs_min == 0.0
            && self.dir_secs_max == 0.0
            && self.move_secs_min == 0.0
            && self.move_secs_max == 0.0
            && self.halt_secs_min == 0.0
            && self.halt_secs_max == 0.0
            && self.speed_min == 0.0
            && self.speed_max == 0.0
            && self.avoid_tile_tags.is_empty()
            && self.avoid_bit_tags.is_empty()
            && self.avoid_race_tags.is_empty()
            && self.avoid_pack_tags.is_empty()
            && self.max_drift.is_nan()
            && self.pack_orbit_radius == 0.0
            && self.pack_orbit_retarget_secs_min == 0.0
            && self.pack_orbit_retarget_secs_max == 0.0
            && !self.wander_around_leader
            && self.avoid_entity_radius.is_empty()
            && self.avoid_entity_strength.is_empty()
            && !self.avoid_blacklisted_spawn_tiles
    }

    pub fn avoid_entity_radius_for(&self, tag: &str) -> f32 {
        self.avoid_entity_radius.get(tag).copied().unwrap_or(DEFAULT_AVOID_ENTITY_RADIUS)
    }

    pub fn avoid_entity_strength_for(&self, tag: &str) -> f32 {
        self.avoid_entity_strength.get(tag).copied().unwrap_or(DEFAULT_AVOID_ENTITY_STRENGTH)
    }

    pub fn max_avoid_entity_radius(&self) -> f32 {
        self.avoid_entity_radius.values().copied().fold(DEFAULT_AVOID_ENTITY_RADIUS, f32::max)
    }
}

#[derive(Component, Debug, Default, Deserialize, Serialize, Copy, Clone)]
pub struct AvoidBlacklistedSpawnTilesForWander;
