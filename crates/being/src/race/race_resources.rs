
use bevy::{platform::collections::{HashMap, HashSet}, prelude::*};
use game_common::game_common_seris::NormalDistSeri;
use crate::race::Race;



#[derive(serde::Deserialize, Asset, TypePath, Default, Debug, )]
pub struct RaceSeri {
    pub id: String,
    pub name: String,

    ///can be a body sampler
    pub body_tree_or_sampler: String,
    #[serde(default)]
    pub mass_kg: f32,
    #[serde(default)]
    pub distributed_totals: HashMap<String, f32>,
    pub name_generator: Option<String>,
    #[serde(default)]
    pub icon_path: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub demonym: String,
    #[serde(default)]
    pub singular: String,
    #[serde(default)]
    pub plural: String,
    pub sexes: HashMap<String, RaceSexSeri>,//id -> sex config
    #[serde(default)]
    pub sentient: bool,
    pub fallback_sprites_to_sample: Vec<String>,


    #[serde(default = "default_true")]
    pub scale_hp_and_strength_with_size: bool,



    ///multiplies both hp and sprite size (both vertically and horizontally, multiplies with the other sprite normalvariation multiplier, generated multipliers should be in the range of (0.8, 1.2))
    /// overriden by being inst template's NormalVariation (if present)
    #[serde(default)]
    pub size_variation: NormalDistSeri,

    // only affects sprite width (should generate multipliers close to 1 to avoid weird rsults (0.95, 1.05))
    /// overriden by being inst template's NormalVariation (if present)
    #[serde(default)]
    pub hori_variation: NormalDistSeri,
    // only affects sprite height (should generate multipliers close to 1 to avoid weird rsults (0.95, 1.05))
    /// overriden by being inst template's NormalVariation (if present)
    #[serde(default)]
    pub vert_variation: NormalDistSeri,

    /// Each vec entry is a tuple of (group_name (face, hair, skin color) and the choosable sprites. from the group (HashSet<String>) you are supposed to choose only one). fallbacks to sprites_to_sample when offering sprites to select to player if None
    #[serde(default)]
    pub sets_of_choosable_sprites: Vec<(String, HashSet<String>)>,
    /// global_caloric_burn_rate_multiplier
    #[serde(default)]
    pub caloric_burn_rate_multiplier: f32,
    #[serde(default)]
    pub can_walk_on: HashSet<String>,
    #[serde(default)]
    pub walk_speeds_on_tiles: HashMap<String, f32>,

    #[serde(default)]
    pub whitelisted_tiles_for_spawning: HashSet<String>,
    #[serde(default)]
    pub blacklisted_tiles_for_spawning: HashSet<String>,
    #[serde(default)]
    pub friend_races: HashSet<String>,
    #[serde(default)]
    pub predator_territorialism: f32,
    #[serde(default = "default_pack_size_range")]
    pub predator_pack_size_range: (u32, u32),
    #[serde(default)]
    pub predator_dont_hunt: HashSet<String>,
    #[serde(default = "default_prey_body_size_ratio_tolerance")]
    ///uses current pack's sum of body sizes
    pub predator_prey_body_size_ratio_tolerance: f32,
    #[serde(default = "default_predator_hunt_threshold")]
    pub predator_hunt_threshold: f32,
    #[serde(default)]
    pub wander: WanderSeri,
}

#[derive(serde::Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum RaceSexSeri {
    Legacy((u32, Vec<String>)),
    Extended(RaceSexEntrySeri),
}
impl RaceSexSeri {
    pub fn weight(&self) -> u32 {
        match self {
            Self::Legacy((weight, _)) => *weight,
            Self::Extended(entry) => entry.weight,
        }
    }
    pub fn sprites(&self) -> &Vec<String> {
        match self {
            Self::Legacy((_, sprites)) => sprites,
            Self::Extended(entry) => &entry.sprites,
        }
    }
    pub fn size_variation(&self) -> Option<NormalDistSeri> {
        match self {
            Self::Legacy(_) => None,
            Self::Extended(entry) => entry.size_variation.clone(),
        }
    }
}

#[derive(serde::Deserialize, Debug, Clone, Default)]
pub struct RaceSexEntrySeri {
    #[serde(default)]
    pub weight: u32,
    #[serde(default)]
    pub sprites: Vec<String>,
    pub size_variation: Option<NormalDistSeri>,
}

#[derive(serde::Deserialize, Asset, TypePath, Debug, Clone, Default)]
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
    pub avoid: HashSet<String>,
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
            && self.avoid.is_empty()
    }
}

#[inline]
pub fn normal_dist_is_disabled(nd: &NormalDistSeri) -> bool {
    nd.min == 0.0 && nd.max == 0.0 && nd.mean == 0.0 && nd.std_dev == 0.0
}

fn default_true() -> bool { true }
fn default_predator_hunt_threshold() -> f32 { ::being_shared::PredatorHuntThreshold::SERI_SENTINEL }
fn default_pack_size_range() -> (u32, u32) { (1, 1) }
fn default_prey_body_size_ratio_tolerance() -> f32 { -1.0 }

common::define_entity_map_systems!(
    Race,
    RaceSeri, "seri.being.race", "race.ron",
);
