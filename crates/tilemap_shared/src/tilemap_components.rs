use std::hash::{DefaultHasher, Hash, Hasher};

use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use bevy_replicon::prelude::*;
use common::common_components::*;
use serde::{Deserialize, Serialize};

use crate::{GlobalTilePos, HashablePosVec, OplistSize};

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct PreChunkDespawnSystems;

#[derive(Component, Debug, Deserialize, Serialize, Clone, )]
#[require(Replicated, Prefix::trunc("GlobalGenSettings"), AssetScoped, HotReload)]
pub struct GlobalGenSettings {

    pub seed: i32,
    pub world_freq: f32,
    pub tectonic_frequency: f32,
    pub hot_reload_window_open_on_start: bool,
    /// Timeout in seconds to wait for StructureBuildCompliance before giving up
    pub structure_build_timeout_secs: f64,
}
const DONT_TOUCH: f32 = 1000.;
impl Default for GlobalGenSettings {
    fn default() -> Self {
        Self {
            seed: 0,
            world_freq: 20.
            /DONT_TOUCH,
            tectonic_frequency: 20.
            /DONT_TOUCH,
            hot_reload_window_open_on_start: false,
            structure_build_timeout_secs: 4.0,
        }
    }
}

type PDiskDistType = i64;
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Component, )]
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
    pub fn is_allowed_position<T: HashablePosVec>(&self, pos: T, settings: &GlobalGenSettings, dim_hash: HashId, check_within_radius: bool, size_in_tiles: OplistSize) -> bool {
        self.sample(pos, settings, dim_hash, check_within_radius, size_in_tiles) > 0.0
    }

    pub fn sample<T: HashablePosVec>(&self, pos: T, settings: &GlobalGenSettings, dim_hash: HashId, check_within_radius: bool, size_in_tiles: OplistSize) -> f64 {

        let mut sum = 0.0;
        for &(min_distance, seed) in self.mindists_seeds.iter() {
            let val = pos.normalized_hash_value(settings, dim_hash, seed);
            sum += val;
            let added_sample_distance_x = size_in_tiles.x() as i32 - 1;
            let added_sample_distance_y = size_in_tiles.y() as i32 - 1;

            for dy in -(min_distance as i32)..=(min_distance as i32) {
                for dx in -(min_distance as i32)..=(min_distance as i32) {
                    if dx == 0 && dy == 0 {
                        continue;
                    }
                    if check_within_radius && dx * dx + dy * dy > (min_distance as i32).pow(2) {
                        continue;
                    }
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


#[derive(Component, Debug, Copy, Clone, Hash, PartialEq, Eq, )]
#[relationship(relationship_target = Tilemaps)]
pub struct TilemapOf {
    #[relationship]
    pub chunk: Entity,
}
impl TilemapOf {
    pub fn new(chunk: Entity) -> Self {
        Self { chunk }
    }
}

#[derive(Component, Debug, Clone)]
#[relationship_target(relationship = TilemapOf)]
pub struct Tilemaps(Vec<Entity>);
impl Tilemaps { pub fn entities(&self) -> &[Entity] { &self.0 } }
