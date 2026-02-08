use bevy::{ecs::entity::{EntityHashMap, EntityHashSet}, math::U16Vec2, platform::collections::HashMap, prelude::*, tasks::Task};
use bevy_ecs_tilemap::{map::TilemapId, tiles::*};
use bevy_replicon::prelude::Replicated;
use common::{AnyDisabling, common_components::HashId};

use crate::{terrain_gen::terrgen_messages::PendingOp };
use dimension_shared::{PrevDimensionRef};
use crate::tile::{tile_components::*, };

use ::tilemap_shared::*;
use game_common::{game_common_components::*, game_common_components_samplers::EntityWeightedSampler};

use bevy_inspector_egui::{egui, inspector_egui_impls::{InspectorPrimitive}, reflect_inspector::InspectorUi};


#[derive(Resource, Debug, Reflect, Default, Clone, )]
#[reflect(Resource, Default)]
pub struct ImportantRegisteredPositions { registered: EntityHashMap<Vec<(DimensionRef, GlobalTilePos)>>, pub exempted: EntityHashSet, }
impl ImportantRegisteredPositions {

    pub fn clear(&mut self) {
        self.registered.clear();
        self.exempted.clear();
    }

    pub fn exempt_entity_from_mindist_checks(&mut self, ent: Entity) {
        self.exempted.insert(ent);
    }
    pub fn register_ezero_at_position(&mut self, ezero: EntityZeroRef, dim: DimensionRef, pos: GlobalTilePos) {
        self.registered.entry(ezero.0).or_default().push((dim, pos));
    }
    pub fn is_pos_registered(&self, ezero: EntityZeroRef, dim: DimensionRef, pos: GlobalTilePos) -> bool {
        self.registered.get(&ezero.0).map_or(false, |positions| {
            positions.iter().any(|(d, p)| *d == dim && *p == pos)
        })
    }

    pub fn get_exempted_entities(&self) -> &EntityHashSet {
        &self.exempted
    }

    pub fn get_registered_entries(&self) -> &EntityHashMap<Vec<(DimensionRef, GlobalTilePos)>> {
        &self.registered
    }

    #[allow(unused_parens, )]
    pub fn check_min_distances(&mut self, cmd: &mut Commands, is_host: bool,
        new: (Entity, EntityZeroRef, DimensionRef, GlobalTilePos, Option<&MinDistancesMap>, Option<&KeepDistanceFrom>),
        min_dists_query: Query<(&MinDistancesMap), (common::AnyDisabling)>,
    ) -> bool {


        let (new_tile, new_tile_ezero, new_dim, new_pos, new_min_distances, keep_distance) = new;

        if (keep_distance.is_some() || new_min_distances.is_some()) && !is_host {
            return false;
        }
        if keep_distance.is_none() && new_min_distances.is_none() {
            return true;
        }

        if ! self.exempted.contains(&new_tile) {
            if let Some(new_min_distances) = new_min_distances {
                for (&ezero_ent, min_dist) in new_min_distances.0.iter() {
                    let Some(previous_positions) = self.registered.get(&ezero_ent) else { continue };
                    for &(prev_dim, prev_pos) in previous_positions {
                        if prev_dim == new_dim && new_pos.distance_squared(&prev_pos) < min_dist*min_dist {
                            return false;
                        }
                    }
                }
            }
            if let Some(keep_distance) = keep_distance {
                for ezero_ent in &keep_distance.0 {
                    let Some(positions) = self.registered.get(ezero_ent) else { continue };
                    let Ok(min_dists) = min_dists_query.get(*ezero_ent) else { continue };
                    for &prev_pos in positions {
                        if min_dists.check_min_distances(prev_pos, (new_tile_ezero, new_dim, new_pos)) == false {
                            return false;
                        }
                    }
                }
            }
        } else if new_min_distances.is_none() && keep_distance.is_none() {
            return true;
        }

        self.registered.entry(new_tile_ezero.0).or_default().push((new_dim, new_pos));

        cmd.entity(new_tile).try_insert(Replicated);


        true
    }
}

#[derive(Bundle, Debug, Clone, Reflect,)]
pub struct TileMassSpawnBundle{
    pub ezero_ref: EntityZeroRef,
    pub gpos: GlobalTilePos,
    pub dim_ref: DimensionRef,
    pub oplist_size: OplistSize,
    pub tile_bundle: bevy_ecs_tilemap::prelude::TileBundle,
    pub initial_pos: InitialPos,
    pub prev_gpos: PrevGlobalTilePos,
    pub prev_dim_ref: PrevDimensionRef,
}

#[derive(Debug, Clone, Resource, Default, Reflect)]
#[reflect(Resource, Default)]
pub struct MassCollectedTiles  (pub Vec<(Entity, TileMassSpawnBundle)>);
impl MassCollectedTiles {

    /// for iterable collections
    pub fn add_tiles_from_ezeros(
        &mut self,
        cmd: &mut Commands,
        ezeros: impl IntoIterator<Item = EntityZeroRef>,
        global_pos: GlobalTilePos,
        dim_ref: DimensionRef,
        oplist_size: OplistSize,
    ) -> Vec<Entity> {
        let ezeros_iter = ezeros.into_iter();
        let mut spawned = Vec::with_capacity(ezeros_iter.size_hint().0);
        spawned.extend(ezeros_iter.map(|ezero| {
            self.clonespawn_and_push_tile(cmd, ezero, global_pos, dim_ref, oplist_size, )
        }));
        spawned
    }
    pub fn clonespawn_and_push_tile(
        &mut self,
        cmd: &mut Commands,
        ezero_ref: EntityZeroRef,
        gpos: GlobalTilePos,
        dim_ref: DimensionRef,

        oplist_size: OplistSize,
    ) -> Entity {
        let tile_instance = cmd.entity(ezero_ref.0).clone_and_spawn_with_opt_out(|builder|{
            builder.deny::<ToDenyOnTileClone>();
            //builder.deny::<BundleToDenyOnReleaseBuild>();
        }).id();
        let tile_bundle = TileBundle {
            position: gpos.to_tilepos(oplist_size), ..Default::default()
        };
        let helper = TileMassSpawnBundle {
            ezero_ref,
            gpos,
            dim_ref,
            oplist_size,
            tile_bundle,
            initial_pos: InitialPos(gpos),
            prev_gpos: PrevGlobalTilePos(gpos),
            prev_dim_ref: PrevDimensionRef(dim_ref.0),
        };
        self.0.push((tile_instance, helper));
        tile_instance
    }

    fn collect_tiles_rec(
        &mut self,
        cmd: &mut Commands,
        tiling_ent: Entity,
        global_pos: GlobalTilePos,
        dim_hash_id: HashId,
        dim_ref: DimensionRef,
        oplist_size: OplistSize,
        weight_maps: &Query<(&EntityWeightedSampler,), ()>,
        gen_settings: &GlobalGenSettings,
        depth: u32
    ) {
        if let Ok((wmap, )) = weight_maps.get(tiling_ent) {
            if let Some(tiling_ent) = wmap.sample_with_pos(global_pos, gen_settings, dim_hash_id) {
                if depth > 6 {
                    warn!("Tile insertion depth exceeded 6, stopping recursion for tile {:?}", tiling_ent);
                    return;
                }
                self.collect_tiles_rec(cmd, tiling_ent, global_pos, dim_hash_id, dim_ref, oplist_size, weight_maps, gen_settings, depth + 1);
            }
        } else {
            self.clonespawn_and_push_tile(cmd, EntityZeroRef(tiling_ent), global_pos, dim_ref, oplist_size, );
        }
    }
    ///used by terr gen
    pub fn collect_tiles(&mut self,
        cmd: &mut Commands,
        bif_tiles: &Vec<Entity>, ev: &PendingOp, oplist_size: OplistSize, weight_maps: &Query<(&EntityWeightedSampler,), ()>, gen_settings: &GlobalGenSettings,
        dim_hash_id: HashId,
    )  {
        for tile in bif_tiles.iter().cloned() {
            self.collect_tiles_rec(cmd, tile, ev.gpos, dim_hash_id, ev.dimension_ref, oplist_size, weight_maps, gen_settings, 0);
        }
    }

}



#[derive(Debug, )]
pub struct ChunkEntityMatrix {
    cells: Box<[Vec<Entity>; ChunkPos::CHUNK_AREA]>,
}
impl ChunkEntityMatrix {
    pub fn new() -> Self {
        let cells = std::array::from_fn(|_| Vec::new());
        Self {
            cells: Box::new(cells),
        }
    }
    fn index(local: UVec2) -> usize {
        let width = ChunkPos::CHUNK_SIZE.x as usize;
        (local.y as usize * width) + local.x as usize
    }
    pub fn get(&self, local: UVec2) -> &[Entity] {
        self.cells.get(Self::index(local)).map_or(&[], |cell| cell.as_slice())
    }
    pub fn push(&mut self, local: UVec2, entity: Entity) {
        if let Some(cell) = self.cells.get_mut(Self::index(local)) {
            cell.push(entity);
        }
    }
    pub fn swap_remove(&mut self, local: UVec2, entity: Entity) -> Option<()> {
        let cell = self.cells.get_mut(Self::index(local))?;
        let idx = cell.iter().position(|&e| e == entity)?;
        cell.swap_remove(idx);
        Some(())
    }
}
impl Default for ChunkEntityMatrix {
    fn default() -> Self {
        let cells = std::array::from_fn(|_| Vec::new());
        Self {
            cells: Box::new(cells),
        }
    }
}
