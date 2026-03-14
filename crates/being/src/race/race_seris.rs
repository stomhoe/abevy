use bevy::{platform::collections::{HashMap, HashSet}, prelude::*};
use game_common::game_common_seris::NormalDistSeri;
use tilemap_shared::tilemap_seris::InteractionZoneSeri;


#[derive(serde::Deserialize, Asset, TypePath, Default, Debug)]
pub struct RaceSeri {
    pub id: String,
    pub name: String,
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
    pub sexes: HashMap<String, RaceSexEntrySeri>,
    #[serde(default)]
    pub sentient: bool,
    pub fallback_sprites_to_sample: Vec<String>,
    #[serde(default = "default_true")]
    pub scale_hp_and_strength_with_size: bool,
    #[serde(default)]
    pub size_variation: NormalDistSeri,
    #[serde(default)]
    pub hori_variation: NormalDistSeri,
    #[serde(default)]
    pub vert_variation: NormalDistSeri,
    #[serde(default)]
    pub sets_of_choosable_sprites: Vec<(String, HashSet<String>)>,
    #[serde(default)]
    pub caloric_burn_rate_multiplier: f32,
    #[serde(default)]
    pub can_walk_on: HashSet<String>,
    #[serde(default = "default_true")]
    pub produces_step_sfx: bool,
    #[serde(default)]
    pub footstep_sfx: RaceFootstepSfxSeri,
    #[serde(default)]
    pub walk_speeds_on_tiles: HashMap<String, f32>,
    #[serde(default)]
    pub whitelisted_tiles_for_spawning: HashSet<String>,
    #[serde(default)]
    pub blacklisted_tiles_for_spawning: HashSet<String>,
    #[serde(default)]// these are not hunted/attacked even if hungry and carnivore
    pub friend_races: HashSet<String>,
    #[serde(default)]
    pub predator_territorialism: f32,
    #[serde(default = "default_pack_size_range")]
    pub predator_pack_size_range: (u32, u32),
    #[serde(default)]
    pub predator_dont_hunt: HashSet<String>,
    #[serde(default = "default_prey_body_size_ratio_tolerance")]
    pub predator_prey_body_size_ratio_tolerance: f32,
    #[serde(default = "default_predator_hunt_threshold")]
    pub predator_hunt_threshold: f32,
    #[serde(default)]
    pub wander: WanderSeri,
    #[serde(default = "default_melee_interaction_zone")]
    pub melee_interaction_zone: InteractionZoneSeri,
    #[serde(default)]
    pub hitbox_hashid: String,

    #[serde(default)]//targets for already-spawned packs
    pub pack_size_min_max: (u32, u32),

    #[serde(default)]//if this race wins sampling, it will spawn with a pack size drawn from this distribution
    pub spawn_pack_size_normal_dist: NormalDistSeri,
    #[serde(default)]//additive membership into already-defined packs
    pub belongs_to_packs: Vec<String>,

    // IMPLEMENT THIS in race_init_systems.rs
    #[serde(default)]
    pub biome_affinity: HashMap<String, f32>,//f32: multiplier for own weight, when there are multiple candidate races/bitrefs to spawn for a given biome.
}

#[derive(serde::Deserialize, Debug, Clone, Default)]
pub struct RaceFootstepSfxSeri {
    #[serde(default)]
    pub paths: Vec<String>,
    #[serde(default)]
    pub disable_tile_step_sfx: bool,
}

#[derive(serde::Deserialize, Debug, Clone, Default)]
pub struct RaceSexEntrySeri {//TODO fix usage
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

fn default_true() -> bool { true }
fn default_melee_interaction_zone() -> InteractionZoneSeri {
    InteractionZoneSeri {
        offset_positions: vec![(0, 1)],
        radius_offset: Vec::new(),
    }
}
fn default_predator_hunt_threshold() -> f32 { ::being_shared::PredatorHuntThreshold::SERI_SENTINEL }
fn default_pack_size_range() -> (u32, u32) { (1, 1) }
fn default_prey_body_size_ratio_tolerance() -> f32 { -1.0 }
