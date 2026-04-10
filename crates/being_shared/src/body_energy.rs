use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use common::log_targets::BODY_ENERGY_SYSTEM;

/// Static body-wide tuning values loaded from the `.body` template.
#[derive(Component, Debug, Deserialize, Serialize, Clone, Copy)]
pub struct BodyEnergyProfile {
    pub burn_rate_multiplier: f32,
    pub wasting_rate_multiplier: f32,
    pub healthy_fat_capacity_multiplier: f32,
}
impl Default for BodyEnergyProfile {
    fn default() -> Self {
        Self {
            burn_rate_multiplier: 1.0,
            wasting_rate_multiplier: 1.0,
            healthy_fat_capacity_multiplier: 1.0,
        }
    }
}

/// Runtime energy storage for one body.
/// `baseline_mass_kg` is the starting lean reference mass, while the other fields track current reserves.
#[derive(Component, Debug, Deserialize, Serialize, Clone, Copy)]
pub struct BodyEnergyStore {
    pub baseline_mass_kg: f32,
    pub lean_mass_kg: f32,
    pub fat_mass_kg: f32,
    pub stomach_kcal: f32,
    pub burn_kcal_per_sec: f32,
    pub activity_multiplier: f32,
    pub thermal_multiplier: f32,
}
impl Default for BodyEnergyStore {
    fn default() -> Self {
        Self {
            baseline_mass_kg: 0.0,
            lean_mass_kg: 0.0,
            fat_mass_kg: 0.0,
            stomach_kcal: 0.0,
            burn_kcal_per_sec: 0.0,
            activity_multiplier: 1.0,
            thermal_multiplier: 1.0,
        }
    }
}

/// Cached energy balance from the last energy tick.
#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, Copy)]
pub struct BodyEnergyBalance {
    pub last_tick_net_kcal: f32,
    pub unresolved_deficit_kcal: f32,
}

/// Starvation tuning values that control how quickly the body burns fat, then lean mass, and how much damage it takes when reserves are gone.
#[derive(Component, Debug, Deserialize, Serialize, Clone, Copy)]
pub struct StarvationConfig {
    pub max_fat_mobilization_kcal_per_sec: f32,
    pub max_lean_catabolism_kcal_per_sec: f32,
    pub damage_per_sec_at_zero_lean: f32,
}
impl Default for StarvationConfig {
    fn default() -> Self {
        Self {
            max_fat_mobilization_kcal_per_sec: 9999.0,
            max_lean_catabolism_kcal_per_sec: 9999.0,
            damage_per_sec_at_zero_lean: 1.0,
        }
    }
}

/// Message used by food or digestion systems to add usable calories to a being's body storage.
#[derive(Debug, Copy, Clone, Message)]
pub struct AddCaloriesToBeing {
    pub being: Entity,
    pub kcal: f32,
}
impl AddCaloriesToBeing {
    pub fn new(being: Entity, kcal: f32, ) -> Self {
        if kcal < 0.0 {
            trace!(target: BODY_ENERGY_SYSTEM, "Ignored negative AddCaloriesToBeing kcal={} for {:?}", kcal, being);
        }
        Self {
            being,
            kcal: kcal.max(0.0),
        }
    }
}

/// Per-being hunger and body-composition state derived from the global energy model.
#[derive(Component, Debug, Deserialize, Serialize, Copy, Clone, Reflect)]
pub struct BodyCondition {
    pub hunger_ratio: f32,
    pub wasting: f32,
    pub obesity: f32,
}
impl Default for BodyCondition {
    fn default() -> Self {
        Self {
            hunger_ratio: 0.0,
            wasting: 0.0,
            obesity: 0.0,
        }
    }
}

/// Per-being multiplier used to scale strength- and movement-related effects from wasting or obesity.
#[derive(Component, Debug, Deserialize, Serialize, Copy, Clone, Reflect)]
pub struct BodyStrengthScale(pub f32);
impl Default for BodyStrengthScale {
    fn default() -> Self {
        Self(1.0)
    }
}
