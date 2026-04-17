use std::hash::{DefaultHasher, Hash, Hasher};

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{GlobalGenSettings, GlobalTilePos, HashablePosVec};
use common::common_components::HashId;

type PDiskDistType = i64;

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Component)]
pub struct PoissonDisk {
    pub mindists_seeds: Vec<(PDiskDistType, u64)>,
}
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
    fn coarse_stride(min_distance: PDiskDistType) -> i32 {
        (min_distance as i32).saturating_add(1).max(1)
    }
    fn coarse_phase(stride: i32, settings: &GlobalGenSettings, dim_hash: HashId, seed: u64, axis: u64) -> i32 {
        let mut hasher = DefaultHasher::new();
        stride.hash(&mut hasher);
        settings.seed.hash(&mut hasher);
        dim_hash.hash(&mut hasher);
        seed.hash(&mut hasher);
        axis.hash(&mut hasher);
        (hasher.finish() % stride as u64) as i32
    }
    fn coarse_cell_hash(cell_x: i32, cell_y: i32, settings: &GlobalGenSettings, dim_hash: HashId, seed: u64) -> u64 {
        GlobalTilePos(IVec2::new(cell_x, cell_y)).hash_value(settings, dim_hash, seed)
    }
    #[allow(dead_code)]
    fn exact_allows_position(
        pos_x: i32,
        pos_y: i32,
        min_distance: PDiskDistType,
        settings: &GlobalGenSettings,
        dim_hash: HashId,
        seed: u64,
    ) -> bool {
        let val = GlobalTilePos(IVec2::new(pos_x, pos_y)).normalized_hash_value(settings, dim_hash, seed);
        for dy in -(min_distance as i32)..=(min_distance as i32) {
            for dx in -(min_distance as i32)..=(min_distance as i32) {
                if dx == 0 && dy == 0 {
                    continue;
                }
                if dx * dx + dy * dy > (min_distance as i32).pow(2) {
                    continue;
                }
                let neighbor_pos = GlobalTilePos(IVec2::new(pos_x + dx, pos_y + dy));
                let neighbor_val = neighbor_pos.normalized_hash_value(settings, dim_hash, seed);
                if neighbor_val > val {
                    return false;
                }
            }
        }
        true
    }
    pub fn allows_position<T: HashablePosVec>(&self, pos: T, settings: &GlobalGenSettings, dim_hash: HashId) -> bool {
        for &(min_distance, seed) in self.mindists_seeds.iter() {
            if min_distance <= 2 {
                if !Self::exact_allows_position(pos.x(), pos.y(), min_distance, settings, dim_hash, seed) {
                    return false;
                }
                continue;
            }

            let stride = Self::coarse_stride(min_distance);
            let phase_x = Self::coarse_phase(stride, settings, dim_hash, seed, 0);
            let phase_y = Self::coarse_phase(stride, settings, dim_hash, seed, 1);

            if pos.x().rem_euclid(stride) != phase_x || pos.y().rem_euclid(stride) != phase_y {
                return false;
            }

            let cell_x = (pos.x() - phase_x).div_euclid(stride);
            let cell_y = (pos.y() - phase_y).div_euclid(stride);
            let cell_hash = Self::coarse_cell_hash(cell_x, cell_y, settings, dim_hash, seed);

            for neighbor_y in (cell_y - 1)..=(cell_y + 1) {
                for neighbor_x in (cell_x - 1)..=(cell_x + 1) {
                    if neighbor_x == cell_x && neighbor_y == cell_y {
                        continue;
                    }
                    let neighbor_hash = Self::coarse_cell_hash(neighbor_x, neighbor_y, settings, dim_hash, seed);
                    if neighbor_hash > cell_hash {
                        return false;
                    }
                }
            }
        }
        true
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
}
impl Default for PoissonDisk {
    fn default() -> Self {
        Self { mindists_seeds: vec![(1, 0)] }
    }
}
