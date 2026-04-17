use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub enum FightOrFlightReaction {
    #[serde(alias = "counterattack", alias = "counter_attack", alias = "counter-attack")]
    Counterattack,
    #[serde(alias = "flee")]
    Flee,
}

impl Default for FightOrFlightReaction {
    fn default() -> Self {
        Self::Counterattack
    }
}

#[derive(Component, Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct FightOrFlightConfig {
    pub reaction: FightOrFlightReaction,
    pub min_melee_strength_ratio_to_counterattack: f32,
    pub curr_hp_ratio_over_my_max_hp_to_start_fleeing: Option<f32>,
    pub entire_nearby_squad_counterattacks: bool,
    pub retaliation_chase_stop_distance_tiles: f32,
}

impl Default for FightOrFlightConfig {
    fn default() -> Self {
        Self {
            reaction: FightOrFlightReaction::Counterattack,
            min_melee_strength_ratio_to_counterattack: 0.6,
            curr_hp_ratio_over_my_max_hp_to_start_fleeing: None,
            entire_nearby_squad_counterattacks: false,
            retaliation_chase_stop_distance_tiles: 300.0,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct RangedFightingStyle {
    #[serde(default)]
    pub min_speed_ratio_over_enemy_to_bother_keep_distance: f32,
}

impl Default for RangedFightingStyle {
    fn default() -> Self {
        Self {
            min_speed_ratio_over_enemy_to_bother_keep_distance: 1.3,
        }
    }
}

#[derive(Component, Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum FightingStyle {
    Melee,
    Ranged(RangedFightingStyle),
}

impl Default for FightingStyle {
    fn default() -> Self {
        Self::Melee
    }
}

impl FightingStyle {
    pub fn ranged_keep_distance_threshold(&self) -> Option<f32> {
        match self {
            Self::Melee => None,
            Self::Ranged(ranged) => Some(ranged.min_speed_ratio_over_enemy_to_bother_keep_distance.max(0.0)),
        }
    }
}
