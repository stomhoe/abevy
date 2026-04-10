#[allow(unused_imports)]
use bevy::prelude::*;
use bevy::platform::collections::HashMap;
use tilemap_shared::InteractionZoneSeri;

fn default_one_f32() -> f32 {
    1.0
}
fn default_starvation_kcal_per_sec() -> f32 {
    9999.0
}

#[derive(serde::Deserialize, Asset, TypePath, Default, Debug, Clone)]
pub struct BodypartNodeSeri {
    pub part_id: String,
    #[serde(default)]
    pub label_override: String,
    pub children: Vec<BodypartNodeSeri>,
}

/// Template data for a full body. This is where species-level energy tuning lives.
#[derive(serde::Deserialize, Asset, TypePath, Default, Debug)]
#[serde(default)]
pub struct BodySeri {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub distributed_totals: HashMap<String, f32>,
    #[serde(default)]
    pub sexes: HashMap<String, being_shared::RaceSexEntrySeri>,
    #[serde(default = "default_one_f32")]
    pub caloric_burn_rate_multiplier: f32,
    #[serde(default = "default_one_f32")]
    pub wasting_rate_multiplier: f32,
    #[serde(default = "default_one_f32")]
    pub healthy_fat_capacity_multiplier: f32,
    #[serde(default = "default_starvation_kcal_per_sec")]
    pub max_fat_mobilization_kcal_per_sec: f32,
    #[serde(default = "default_starvation_kcal_per_sec")]
    pub max_lean_catabolism_kcal_per_sec: f32,
    #[serde(default = "default_one_f32")]
    pub damage_per_sec_at_zero_lean: f32,
    pub melee_interaction_zone: InteractionZoneSeri,
    pub collision_zone: InteractionZoneSeri,
    pub bodytree_id: String,
}
