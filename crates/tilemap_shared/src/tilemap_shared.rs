use std::{hash::{DefaultHasher, Hash, Hasher}};

#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
use common::common_components::{HashId, Prefix};
use serde::{Deserialize, Serialize};

use crate::tilemap_positioning::{GlobalTilePos, HashablePosVec, OplistSize};

#[derive(Component, Debug, Reflect, Deserialize, Serialize, Clone, )]
#[require(Replicated, Prefix::trunc("GlobalGenSettings"))]
pub struct GlobalGenSettings {

    pub seed: i32,
    pub world_freq: f32,
    /// Timeout in seconds to wait for StructureBuildCompliance before giving up
    pub structure_build_timeout_secs: f64,
}
impl Default for GlobalGenSettings {
    fn default() -> Self {
        Self {
            seed: 3,
            world_freq: 6.
            /100.,//<-DON'T TOUCH
            structure_build_timeout_secs: 4.0,
        }
    }
}

#[derive(Debug, Message, Default)]
pub struct ForceAllChunksDespawn;

type PDiskDistType = i64;
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Reflect, Component, )]
pub struct PoissonDisk { pub mindists_seeds: Vec<(PDiskDistType, u64)>, }
impl PoissonDisk {
    pub fn new(min_distance: u8, seed: u64) -> Result<Self, BevyError> {

        let max = 5;

        if min_distance > max {
            return Err(BevyError::from(format!("min_distance must be <= {}", max)));
        } else if min_distance == 0 {
            return Err(BevyError::from("min_distance must be > 0"));
        }
        Ok(Self { mindists_seeds: vec![(min_distance as PDiskDistType, seed)] })
    }
    pub fn multiple_tagged(mindists_tag: Vec<(Option<u8>, String)>, fallback_mindist: u8, max: u8) -> Result<Self, BevyError> {
        let mut mindists_seeds: Vec<(PDiskDistType, u64)> = Vec::with_capacity(mindists_tag.len());
        for (min_distance, tag) in mindists_tag.iter() {
            let min_distance = min_distance.unwrap_or(fallback_mindist);

            if min_distance > max {
                return Err(BevyError::from(format!("min_distance must be <= {}", max)));
            } else if min_distance == 0 {
                return Err(BevyError::from("min_distance must be > 0"));
            }
            let mut hasher = DefaultHasher::new();
            tag.hash(&mut hasher);
            let seed = hasher.finish();
            mindists_seeds.push((min_distance as PDiskDistType, seed));
        }
        Ok(Self { mindists_seeds })
    }
    pub fn is_allowed_position<T: HashablePosVec>(&self, pos: T, settings: &GlobalGenSettings, dim_hash: HashId, check_within_radius: bool, oplist_size: OplistSize) -> bool {
        self.sample(pos, settings, dim_hash, check_within_radius, oplist_size) > 0.0
    }

    pub fn sample<T: HashablePosVec>(&self, pos: T, settings: &GlobalGenSettings, dim_hash: HashId, check_within_radius: bool, oplist_size: OplistSize) -> f64 {

        let mut sum = 0.0;
        for &(min_distance, seed) in self.mindists_seeds.iter() {
            let val = pos.normalized_hash_value(settings, dim_hash, seed);
            sum += val;
            let added_sample_distance_x = oplist_size.x() as i32 - 1;
            let added_sample_distance_y = oplist_size.y() as i32 - 1;

            for dy in -(min_distance as i32)..=(min_distance as i32) {
                for dx in -(min_distance as i32)..=(min_distance as i32) {
                    if dx == 0 && dy == 0 {
                        continue;
                    }
                    // Only check within circle of radius min_distance
                    if check_within_radius && dx * dx + dy * dy > (min_distance as i32).pow(2) {
                        continue;
                    }
                    // Calculate the neighbor's position by offsetting the current tile position
                    let neighbor_x = pos.x() + dx + added_sample_distance_x;
                    let neighbor_y = pos.y() + dy + added_sample_distance_y;
                    let neighbor_pos = GlobalTilePos(IVec2::new(neighbor_x, neighbor_y));
                    let neighbor_val = neighbor_pos.normalized_hash_value(settings, dim_hash, seed);
                    if neighbor_val > val {
                        return 0.0;
                    }
                }
            }

        }
        sum / (self.mindists_seeds.len() as f64)
        }
}
impl Default for PoissonDisk { fn default() -> Self { Self { mindists_seeds: vec![(1, 0)] } } }
