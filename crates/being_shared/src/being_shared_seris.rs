use bevy::{platform::collections::{HashMap, HashSet}, prelude::*};

#[derive(serde::Deserialize, Asset, TypePath, Clone, Default, Debug)]
pub struct WanderSeri {
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
    pub avoid_race_tags: HashSet<String>,
    #[serde(default)]
    pub avoid_bit_tags: HashSet<String>,
    #[serde(default)]
    pub avoid_pack_tags: HashSet<String>,//avoid pack instances which reference packtempl with these tags
    #[serde(default = "default_nan")]
    pub max_drift: f32, //use NaN as sentinel for None (if None it should deactivate)
    #[serde(default)]
    pub pack_orbit_radius: f32,
    #[serde(default)]
    pub pack_orbit_retarget_secs_min: f32,
    #[serde(default)]
    pub pack_orbit_retarget_secs_max: f32,
    #[serde(default)]
    pub wander_around_leader: bool, //if true, orbit around the pack center rather than directly homing to it
    #[serde(default)]
    pub avoid_entity_radius: HashMap<String, f32>,
    #[serde(default)]
    pub avoid_entity_strength: HashMap<String, f32>,
    #[serde(default)]
    pub avoid_blacklisted_spawn_tiles: bool,
}
impl WanderSeri {
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
            && self.avoid_race_tags.is_empty()
            && self.avoid_bit_tags.is_empty()
            && self.avoid_pack_tags.is_empty()
            && self.avoid_entity_radius.is_empty()
            && self.avoid_entity_strength.is_empty()
            && self.max_drift.is_nan()
            && self.pack_orbit_radius == 0.0
            && self.pack_orbit_retarget_secs_min == 0.0
            && self.pack_orbit_retarget_secs_max == 0.0
            && !self.wander_around_leader
            && !self.avoid_blacklisted_spawn_tiles
    }
}

pub const DEFAULT_AVOID_ENTITY_RADIUS: f32 = 18.0;
pub const DEFAULT_AVOID_ENTITY_STRENGTH: f32 = 0.85;

pub fn default_nan() -> f32 {
    f32::NAN
}
