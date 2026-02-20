
use bevy::{platform::collections::{HashMap, HashSet}, prelude::*};
use game_common::game_common_seris::NormalDistSeri;
use crate::race::Race;



#[derive(serde::Deserialize, Asset, TypePath, Default, Debug, )]
pub struct RaceSeri {
    pub id: String,
    pub name: String,

    ///can be a body sampler
    pub body_tree: String,
    pub name_generator: Option<String>,
    pub icon_path: Option<String>,
    pub description: Option<String>,
    pub demonym: Option<String>,
    pub singular: Option<String>,
    pub plural: Option<String>,
    pub sexes: HashMap<String, (u32, Vec<String>)>,//id, (weight, spriteents_to_sampl (may be empty, in that case fallback is used)
    #[serde(default)]
    pub sentient: bool,
    pub fallback_sprites_to_sample: Vec<String>,


    #[serde(default = "default_true")]
    pub scale_hp_and_strength_with_size: bool,



    ///multiplies both hp and sprite size (both vertically and horizontally, multiplies with the other sprite normalvariation multiplier, generated multipliers should be in the range of (0.8, 1.2))
    /// overriden by being inst template's NormalVariation (if present)
    pub size_variation: Option<NormalDistSeri>,

    // only affects sprite width (should generate multipliers close to 1 to avoid weird rsults (0.95, 1.05))
    /// overriden by being inst template's NormalVariation (if present)
    pub hori_variation: Option<NormalDistSeri>,
    // only affects sprite height (should generate multipliers close to 1 to avoid weird rsults (0.95, 1.05))
    /// overriden by being inst template's NormalVariation (if present)
    pub vert_variation: Option<NormalDistSeri>,

    /// Each vec entry is a tuple of (group_name (face, hair, skin color) and the choosable sprites. from the group (HashSet<String>) you are supposed to choose only one). fallbacks to sprites_to_sample when offering sprites to select to player if None
    pub sets_of_choosable_sprites: Option<Vec<(String, HashSet<String>)>>,
    /// global_caloric_burn_rate_multiplier
    pub caloric_burn_rate_multiplier: Option<f32>,
    pub can_walk_on: Option<HashSet<String>>,
    pub walk_speeds_on_tiles: Option<HashMap<String, f32>>,

    pub whitelisted_tiles_for_spawning: Option<HashSet<String>>,
    pub blacklisted_tiles_for_spawning: Option<HashSet<String>>,
    #[serde(default = "default_predator_hunt_threshold")]
    pub predator_hunt_threshold: f32,
}

fn default_true() -> bool { true }
fn default_predator_hunt_threshold() -> f32 { ::being_shared::PredatorHuntThreshold::SERI_SENTINEL }

common::define_entity_map_systems!(
    Race,
    RaceSeri, "seri.being.race", "race.ron",
);
