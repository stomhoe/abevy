use bevy::ecs::entity::{EntityHashMap, EntityHashSet};
use bevy::prelude::*;
use common::common_components::*;
use common::log_targets::TILE_INIT;
use bevy_replicon::prelude::*;

use serde::{Deserialize, Serialize};
use std::hash::{DefaultHasher, Hash, Hasher};

use crate::{tilemap_components::GlobalGenSettings, DimensionRef, GlobalTilePos};

#[derive(Component, Clone, Deserialize, Serialize, Hash, PartialEq, Eq, Copy, Debug)]
pub struct InitialPos {
    pub pos: GlobalTilePos,
    pub dim: DimensionRef,
}
impl Default for InitialPos {
    fn default() -> Self {
        Self {
            pos: GlobalTilePos::default(),
            dim: DimensionRef(Entity::PLACEHOLDER),
        }
    }
}

pub fn tile_pos_hash_rand(initial_pos: InitialPos, settings: &GlobalGenSettings) -> f32 {
    let mut hasher = DefaultHasher::new();
    initial_pos.hash(&mut hasher);
    settings.seed.hash(&mut hasher);
    (hasher.finish() as f64 / u64::MAX as f64).abs() as f32
}

#[derive(Component, Debug, Default, Clone)]
/// Holds the mapping between tile image HashIds and the image handles they are mapped to
pub struct TileHashIdsHandles {
    ids: Vec<HashId>,
    handles: Vec<Handle<Image>>,
}
impl TileHashIdsHandles {
    pub fn from_paths(
        asset_server: &AssetServer,
        img_paths: Vec<(String, String)>,
    ) -> Result<Self, BevyError> {
        if img_paths.is_empty() {
            return Err(BevyError::from("TileImgsMap cannot be created with an empty image paths map"));
        }
        let mut ids = Vec::with_capacity(img_paths.len());
        let mut handles = Vec::with_capacity(img_paths.len());
        for (key, path) in img_paths {
            let Ok(image_holder) = ImagePathHolder::new(path.clone()) else {
                error!(target: TILE_INIT, "Failed to find image file for key {} at path: {}", key, path);
                continue;
            };
            ids.push(HashId::from(key));
            handles.push(asset_server.load(image_holder.path().clone()));
        }
        if ids.is_empty() {
            return Err(BevyError::from("No valid entries"));
        }
        Ok(Self { ids, handles })
    }
    pub fn len(&self) -> usize {
        self.handles.len()
    }
    pub fn first_handle(&self) -> Handle<Image> {
        self.handles.first().cloned().unwrap_or_default()
    }
    /// NO HACER take() porque lo necesitan multiples isntancias de tiles
    pub fn handles(&self) -> &Vec<Handle<Image>> {
        &self.handles
    }
    pub fn iter(&self) -> impl Iterator<Item = (HashId, &Handle<Image>)> {
        self.ids.iter().cloned().zip(self.handles.iter())
    }
}

#[derive(Component, Debug, Default, Deserialize, Serialize, Copy, Clone, PartialEq, Eq)]
/// applied on tile's start gpos if placed via terrgen
pub struct OffsetForTerrgenPlacement(pub GlobalTilePos);
impl OffsetForTerrgenPlacement {
    pub fn from_i32s(offset: (i32, i32)) -> Self {
        Self(GlobalTilePos::new(offset.0, offset.1))
    }
}

#[derive(Component, Debug, Clone, PartialEq, Eq, Default)]
pub struct MinDistancesMap(pub EntityHashMap<u64>);

impl MinDistancesMap {
    #[allow(unused_parens)]
    pub fn check_min_distances(
        &self,
        my_pos: (DimensionRef, GlobalTilePos),
        new: (Entity, DimensionRef, GlobalTilePos),
    ) -> bool {
        self.0.get(&new.0).map_or(true, |&min_dist| {
            my_pos.0 != new.1 || my_pos.1.distance_squared(&new.2) > (min_dist * min_dist)
        })
    }
}

#[derive(Component, Debug, Default, Clone, )]
pub struct KeepDistanceFrom(#[entities] pub Vec<Entity>);

#[derive(Resource, Debug, Default, Clone)]
pub struct ImportantRegisteredPositions {
    registered: EntityHashMap<Vec<(DimensionRef, GlobalTilePos)>>,
    exempted: EntityHashSet,
}
impl ImportantRegisteredPositions {
    pub fn clear(&mut self) {
        self.registered.clear();
        self.exempted.clear();
    }
    pub fn exempt_entity_from_mindist_checks(&mut self, ent: Entity) {
        self.exempted.insert(ent);
    }
    pub fn register_templ_at_position(&mut self, templ: Entity, dim: DimensionRef, pos: GlobalTilePos) {
        self.registered.entry(templ).or_default().push((dim, pos));
    }
    pub fn is_pos_registered(&self, templ: Entity, dim: DimensionRef, pos: GlobalTilePos) -> bool {
        self.registered.get(&templ).map_or(false, |positions| {
            positions.iter().any(|(d, p)| *d == dim && *p == pos)
        })
    }

    pub fn is_any_occupied_pos_registered(
        &self,
        templ: Entity,
        dim: DimensionRef,
        anchor_gpos: GlobalTilePos,
        size: IVec2,
    ) -> bool {
        for y in anchor_gpos.0.y..(anchor_gpos.0.y + size.y) {
            for x in anchor_gpos.0.x..(anchor_gpos.0.x + size.x) {
                if self.is_pos_registered(templ, dim, GlobalTilePos::new(x, y)) {
                    return true;
                }
            }
        }
        false
    }

    #[allow(unused_parens, )]
    pub fn check_min_distances(&mut self, cmd: &mut Commands, is_host: bool,
        new: (Entity, Entity, DimensionRef, GlobalTilePos, Option<&MinDistancesMap>, Option<&KeepDistanceFrom>),
        min_dists_query: Query<(&MinDistancesMap), (common::AnyDisabling)>,
    ) -> bool {
        let (new_tile, new_tile_templ, new_dim, new_pos, new_min_distances, keep_distance) = new;

        if (keep_distance.is_some() || new_min_distances.is_some()) && !is_host {
            return false;
        }
        if keep_distance.is_none() && new_min_distances.is_none() {
            return true;
        }
        if ! self.exempted.contains(&new_tile) {
            if let Some(new_min_distances) = new_min_distances {
                for (&templ_ent, min_dist) in new_min_distances.0.iter() {
                    let Some(previous_positions) = self.registered.get(&templ_ent) else { continue };
                    for &(prev_dim, prev_pos) in previous_positions {
                        if prev_dim == new_dim && new_pos.distance_squared(&prev_pos) < min_dist*min_dist {
                            return false;
                        }
                    }
                }
            }
            if let Some(keep_distance) = keep_distance {
                for templ_ent in &keep_distance.0 {
                    let Some(positions) = self.registered.get(templ_ent) else { continue };
                    let Ok(min_dists) = min_dists_query.get(*templ_ent) else { continue };
                    for &prev_pos in positions {
                        if min_dists.check_min_distances(prev_pos, (new_tile_templ, new_dim, new_pos)) == false {
                            return false;
                        }
                    }
                }
            }
        } else if new_min_distances.is_none() && keep_distance.is_none() {
            return true;
        }
        self.registered.entry(new_tile_templ).or_default().push((new_dim, new_pos));
        cmd.entity(new_tile).try_insert(Replicated);
        true
    }

    pub fn get_exempted_tile_ents(&self) -> &EntityHashSet {
        &self.exempted
    }
    pub fn get_registered_templs(&self) -> &EntityHashMap<Vec<(DimensionRef, GlobalTilePos)>> {
        &self.registered
    }
}
