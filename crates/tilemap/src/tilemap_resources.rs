use bevy::{math::U16Vec2, prelude::*, tasks::Task};
use bevy_ecs_tilemap::tiles::*;
use common::common_components::HashId;

use crate::{terrain_gen::terrgen_messages::PendingOp, };
use dimension_shared::{DimensionRef, PrevDimensionRef};
use crate::tile::{tile_components::*, tile_shader::tile_shader_components::TileShaderRef};
use sprite_shared::AcZ;

use ::tilemap_shared::*;
use game_common::{game_common_components::*, game_common_components_samplers::EntityWeightedSampler};





#[derive(Bundle, Debug, Clone, Reflect)]
pub struct TileMassSpawnBundle{
    pub ezero_ref: EntityZeroRef,
    pub gpos: GlobalTilePos,
    pub prev_gpos: PrevGlobalTilePos,
    pub dim_ref: DimensionRef,
    pub prev_dim_ref: PrevDimensionRef,
    pub oplist_size: OplistSize,
    pub tile_bundle: bevy_ecs_tilemap::prelude::TileBundle,
    pub initial_pos: InitialPos,
    
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
            prev_gpos: PrevGlobalTilePos(gpos),
            dim_ref,
            prev_dim_ref: PrevDimensionRef(dim_ref.0),
            oplist_size,
            tile_bundle,
            initial_pos: InitialPos(gpos),
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

#[derive(Debug, Clone)]
pub struct TilemapPreparedTile {
    pub tile_ent: Entity,
    pub bundle: TileMassSpawnBundle,
    pub tile_strid: TileStrId,
    pub min_dists: Option<MinDistancesMap>,
    pub keep_distance_from: Option<KeepDistanceFrom>,
    pub to_persist: bool,
    pub tile_z_index: AcZ,
    pub tile_handles: Option<TileHidsHandles>,
    pub shader_ref: Option<TileShaderRef>,
    pub transform: Option<Transform>,
    pub color: Option<TileColor>,
    pub tile_size: U16Vec2,
    pub chunk_ent: Option<Entity>,
}

#[derive(Debug, Default)]
pub struct TilemapPrepResult {
    pub prepared_tiles: Vec<TilemapPreparedTile>,
}

#[derive(Resource, Debug, Default)]
pub struct TilemapAsyncTasks {
    pub prep_tasks: Vec<Task<TilemapPrepResult>>,
}