use bevy::platform::collections::HashMap;
#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
use serde::{Deserialize, Serialize};


use rand_distr::weighted::WeightedAliasIndex;
use rand::prelude::*;

#[derive(Debug, )]
pub struct WeightedMap<K> {
    weights: Vec<u32>,
    choices: Vec<K>,
    dist: WeightedAliasIndex<u32>,
}

impl<K: Eq + std::hash::Hash + Clone + Serialize + for<'de> Deserialize<'de>> WeightedMap<K> {
    pub fn new(weights_map: HashMap<K, u32>) -> Self {
        let weights: Vec<u32> = weights_map.values().cloned().collect();
        let choices: Vec<K> = weights_map.keys().cloned().collect();
        let dist = WeightedAliasIndex::new(weights.clone()).unwrap();
        Self {
            weights, choices, dist,
        }
    }

    pub fn choose<R: Rng>(&self, rng: &mut R) -> Option<&K> {
        let index = self.dist.sample(rng) as usize;
        self.choices.get(index)
    }

    pub fn choices(&self) -> &Vec<K> {
        &self.choices
    }

    fn default_dist() -> WeightedAliasIndex<u32> {
        // This will never be used, as dist is always rebuilt in deserialize_with
        WeightedAliasIndex::new(vec![1]).unwrap()
    }
}

// Custom implementation to rebuild dist after deserialization
impl<'de, K: Eq + std::hash::Hash + Clone + Serialize + Deserialize<'de>> Deserialize<'de> for WeightedMap<K> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Helper<K> {
            weights: Vec<u32>,
            choices: Vec<K>,
        }
        let Helper { weights, choices } = Helper::deserialize(deserializer)?;
        let dist = WeightedAliasIndex::new(weights.clone()).map_err(serde::de::Error::custom)?;
        Ok(WeightedMap {
            weights,
            choices,
            dist,
        })
    }
}

impl<K: Eq + std::hash::Hash + Clone + Serialize> Serialize for WeightedMap<K> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        #[derive(Serialize)]
        struct Helper<'a, K> {
            weights: &'a Vec<u32>,
            choices: &'a Vec<K>,
        }
        let helper = Helper {
            weights: &self.weights,
            choices: &self.choices,
        };
        helper.serialize(serializer)
    }
}


pub type StrId = String;
