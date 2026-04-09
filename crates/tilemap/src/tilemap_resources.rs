use bevy::prelude::*;
use bevy_ecs_tilemap::{tiles::*};
use common::{common_components::HashId};

use crate::{terrain::terrgen_messages::PendingOp, tile::tile_bundles::* };

use ::tilemap_shared::*;
use game_common::game_common_components::*;
use tilemap_shared::tilemap_shared_samplers::HashIdWeightedSampler;
use crate::tile::{TileEntityMap, TileWeightedSamplerEntityMap};

#[derive(Debug, Clone, Resource, Default, )]
pub struct MassCollectedTiles  (pub Vec<(Entity, TileMassSpawnBundle)>);
impl MassCollectedTiles {

    pub fn clonespawn_and_push_tile(
        &mut self,
        cmd: &mut Commands,
        templ_ref: TemplEntiRef,
        gpos: GlobalTilePos,
        dim_ref: DimensionRef,
        //param_set: &CloneSpawnParamSet,
    ) -> Entity {
        let tile_instance = cmd.entity(templ_ref.0).clone_and_spawn_with_opt_out(|builder|{
            builder.deny::<ToDenyOnTileClone>();
            //builder.deny::<BundleToDenyOnReleaseBuild>();
        }).id();

        //let tile_size = param_set.size_in_tiles.get(templ_ref.0).cloned().unwrap_or_default();

        let tile_bundle = TileBundle {
            position: gpos.to_tilepos(), ..Default::default()
        };
        let helper = TileMassSpawnBundle {
            templ_ref,
            gpos,
            snap_to_gpos: SnapTransformToGpos::OnChange,
            dim_ref,
            tile_bundle,
            initial_pos: InitialPos { pos: gpos, dim: dim_ref },
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

            if let Some(tiling_hash_id) = wmap.sample_with_pos(global_pos, gen_settings, dim_hash_id) {
                if depth > 6 {
                    warn!("Tile insertion depth exceeded 6, stopping recursion for tile {:?}", tiling_hash_id);
                    return;
                }
                let Some(tiling_ent) = param_set
                    .tile_map
                    .0
                    .get_opt(tiling_hash_id)
                    .copied()
                    .or_else(|| param_set.sampler_map.0.get_opt(tiling_hash_id).copied()) else {
                    warn!("Tile insertion sampled unknown hash {:?} at depth {}", tiling_hash_id, depth);
                    return;
                };
                self.collect_tiles_rec(cmd, tiling_ent, global_pos, dim_hash_id, dim_ref, param_set, depth + 1);
            }
        } else {
            self.clonespawn_and_push_tile(cmd, TemplEntiRef(tiling_ent), global_pos, dim_ref, );
        }
    }
    pub fn collect_tiles(&mut self,
        cmd: &mut Commands,
        templ_refs: impl IntoIterator<Item = Entity>,
        ev: &PendingOp,
        param_set: &CloneSpawnParamSet,
        dim_hash_id: HashId,
    )  {
        self.collect_tiles_at_positions(
            cmd,
            templ_refs.into_iter().map(|tile_ent| (tile_ent, ev.gpos())),
            ev.dimension_ref(),
            param_set,
            dim_hash_id,
        );
    }

    pub fn collect_tiles_at_positions(
        &mut self,
        cmd: &mut Commands,
        templ_refs: impl IntoIterator<Item = (Entity, GlobalTilePos)>,
        dim_ref: DimensionRef,
        param_set: &CloneSpawnParamSet,
        dim_hash_id: HashId,
    ) {
        for (tile_ent, gpos) in templ_refs {
            self.collect_tiles_rec(cmd, tile_ent, gpos, dim_hash_id, dim_ref, param_set, 0);
        }
    }

}
#[derive(bevy::ecs::system::SystemParam)]
#[allow(unused_parens, )]
pub struct CloneSpawnParamSet<'w, 's> {
    pub weight_maps: Query<'w, 's, &'static HashIdWeightedSampler>,
    pub gen_settings: Query<'w, 's, &'static GlobalGenSettings>,
    pub size_in_tiles: Query<'w, 's, &'static SizeInTiles>,
    pub terrgen_offsets: Query<'w, 's, &'static OffsetForTerrgenPlacement, common::AnyDisabling>,
    pub tile_map: Res<'w, TileEntityMap>,
    pub sampler_map: Res<'w, TileWeightedSamplerEntityMap>,
}
