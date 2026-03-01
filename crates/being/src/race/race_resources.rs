
use bevy::{platform::collections::{HashMap, HashSet}, prelude::*};
use game_common::game_common_seris::NormalDistSeri;
use crate::race::Race;
pub use crate::race::race_seris::*;


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


#[inline]
pub fn normal_dist_is_disabled(nd: &NormalDistSeri) -> bool {
    nd.min == 0.0 && nd.max == 0.0 && nd.mean == 0.0 && nd.std_dev == 0.0
}

common::define_entity_map_systems!(
    Race,
    RaceSeri, "seri.being.race", "race.ron",
);
