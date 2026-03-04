use bevy::{ecs::entity::{EntityHashMap, EntityHashSet}, prelude::*};
use bevy_ecs_tilemap::{tiles::*};
use bevy_replicon::prelude::Replicated;
use common::{common_components::HashId};

use crate::{terrain::terrgen_messages::PendingOp, tile::tile_bundles::* };
use crate::tile::{tile_components::*, };

use ::tilemap_shared::*;
use game_common::{game_common_components::*, game_common_samplers::EntityWeightedSampler};



#[derive(Resource, Debug, Default, Clone, )]
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

    pub fn get_exempted_tile_ents(&self) -> &EntityHashSet {
        &self.exempted
    }

    pub fn get_registered_ezeros(&self) -> &EntityHashMap<Vec<(DimensionRef, GlobalTilePos)>> {
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

#[derive(Bundle, Debug, Clone, )]
pub struct TileMassSpawnBundle{
    pub ezero_ref: EntityZeroRef,
    pub gpos: GlobalTilePos,
    pub dim_ref: DimensionRef,
    pub tile_bundle: bevy_ecs_tilemap::prelude::TileBundle,
    pub initial_pos: InitialPos,
    pub prev_gpos: PrevGlobalTilePos,
    pub prev_dim_ref: PrevDimensionRef,
}

#[derive(Debug, Clone, Resource, Default, )]
pub struct MassCollectedTiles  (pub Vec<(Entity, TileMassSpawnBundle)>);
impl MassCollectedTiles {

    /// for iterable collections
    pub fn add_tiles_from_ezeros(
        &mut self,
        cmd: &mut Commands,
        ezeros: impl IntoIterator<Item = EntityZeroRef>,
        global_pos: GlobalTilePos,
        dim_ref: DimensionRef,
        _param_set: &CloneSpawnParamSet,
    ) -> Vec<Entity> {
        let ezeros_iter = ezeros.into_iter();
        let mut spawned = Vec::with_capacity(ezeros_iter.size_hint().0);
        spawned.extend(ezeros_iter.map(|ezero| {

            self.clonespawn_and_push_tile(cmd, ezero, global_pos, dim_ref, )
        }));
        spawned
    }
    pub fn clonespawn_and_push_tile(
        &mut self,
        cmd: &mut Commands,
        ezero_ref: EntityZeroRef,
        gpos: GlobalTilePos,
        dim_ref: DimensionRef,
        //param_set: &CloneSpawnParamSet,
    ) -> Entity {
        let tile_instance = cmd.entity(ezero_ref.0).clone_and_spawn_with_opt_out(|builder|{
            builder.deny::<ToDenyOnTileClone>();
            //builder.deny::<BundleToDenyOnReleaseBuild>();
        }).id();
        //let tile_size = param_set.size_in_tiles.get(ezero_ref.0).cloned().unwrap_or_default();

        let tile_bundle = TileBundle {
            position: gpos.to_tilepos(), ..Default::default()
        };
        let helper = TileMassSpawnBundle {
            ezero_ref,
            gpos,
            dim_ref,
            tile_bundle,
            initial_pos: InitialPos(gpos),
            prev_gpos: PrevGlobalTilePos(Some(gpos)),
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
        param_set: &CloneSpawnParamSet,
        depth: u32
    ) {
        if let Ok(wmap) = param_set.weight_maps.get(tiling_ent) {
            let Ok(gen_settings) = param_set.gen_settings.single() else {
                error!("Failed to get gen_settings");
                return;
            };

            if let Some(tiling_ent) = wmap.sample_with_pos(global_pos, gen_settings, dim_hash_id) {
                if depth > 6 {
                    warn!("Tile insertion depth exceeded 6, stopping recursion for tile {:?}", tiling_ent);
                    return;
                }
                self.collect_tiles_rec(cmd, tiling_ent, global_pos, dim_hash_id, dim_ref, param_set, depth + 1);
            }
        } else {
            self.clonespawn_and_push_tile(cmd, EntityZeroRef(tiling_ent), global_pos, dim_ref, );
        }
    }
    pub fn collect_tiles(&mut self,
        cmd: &mut Commands,
        ezero_refs: impl IntoIterator<Item = Entity>,
        ev: &PendingOp,
        param_set: &CloneSpawnParamSet,
        dim_hash_id: HashId,
    )  {
        self.collect_tiles_at_positions(
            cmd,
            ezero_refs.into_iter().map(|tile_ent| (tile_ent, ev.gpos)),
            ev.dimension_ref,
            param_set,
            dim_hash_id,
        );
    }

    pub fn collect_tiles_at_positions(
        &mut self,
        cmd: &mut Commands,
        ezero_refs: impl IntoIterator<Item = (Entity, GlobalTilePos)>,
        dim_ref: DimensionRef,
        param_set: &CloneSpawnParamSet,
        dim_hash_id: HashId,
    ) {
        for (tile_ent, gpos) in ezero_refs {
            self.collect_tiles_rec(cmd, tile_ent, gpos, dim_hash_id, dim_ref, param_set, 0);
        }
    }

}
#[derive(bevy::ecs::system::SystemParam)]
#[allow(unused_parens, )]
pub struct CloneSpawnParamSet<'w, 's> {
    pub weight_maps: Query<'w, 's, &'static EntityWeightedSampler>,
    pub gen_settings: Query<'w, 's, &'static GlobalGenSettings>,
    pub size_in_tiles: Query<'w, 's, &'static SizeInTiles>,
    pub terrgen_offsets: Query<'w, 's, &'static OffsetForTerrgenPlacement, common::AnyDisabling>,
}
