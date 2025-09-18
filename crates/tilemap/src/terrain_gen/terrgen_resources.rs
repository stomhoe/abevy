use bevy::{ecs::{entity::EntityHashMap, entity_disabling::Disabled}, platform::collections::{HashMap, HashSet}, prelude::*};
use bevy_asset_loader::asset_collection::AssetCollection;
use bevy_ecs_tilemap::tiles::*;
use bevy_replicon::prelude::*;
use common::common_types::HashIdToEntityMap;

use crate::{terrain_gen::terrgen_events::PendingOp, tile::tile_components::{KeepDistanceFrom, MinDistancesMap, }};
use dimension_shared::DimensionRef;
use crate::tile::tile_components::*;


use bevy::{ecs::{entity::MapEntities, }, prelude::*};
use ::tilemap_shared::*;
use std::mem::take;
use game_common::{game_common_components::*, game_common_components_samplers::EntiWeightedSampler};
use std::hash::Hash;


use serde::{Deserialize, Serialize};



#[derive(Resource, Debug, Reflect, Default, Event, Deserialize, Serialize, Clone)]
#[reflect(Resource, Default)]
pub struct RegisteredPositions(pub EntityHashMap<Vec<(DimensionRef, GlobalTilePos)>>); 
impl RegisteredPositions {
    #[allow(unused_parens, )]
    pub fn check_min_distances(&mut self, cmd: &mut Commands, is_host: bool,
        new: (Entity, EntityZeroRef, DimensionRef, GlobalTilePos, Option<&MinDistancesMap>, Option<&KeepDistanceFrom>), 
        min_dists_query: Query<(&MinDistancesMap), (With<Disabled>)>,
    ) -> bool {


        let (new_tile, new_orig_ref, new_dim, new_pos, new_min_distances, keep_distance) = new;

        if (keep_distance.is_some() || new_min_distances.is_some()) && !is_host {
            return false;
        }
        if keep_distance.is_none() && new_min_distances.is_none() {
            return true;
        }

        if let Some(new_min_distances) = new_min_distances {
            for (&oritile_ent, min_dist) in new_min_distances.0.iter() {
                let Some(previous_positions) = self.0.get(&oritile_ent) else { continue };
                for &(prev_dim, prev_pos) in previous_positions {
                    if prev_dim == new_dim && new_pos.distance_squared(&prev_pos) < min_dist*min_dist {
                        return false;
                    }
                }
            }
        }
        if let Some(keep_distance) = keep_distance {
            for other_ent in &keep_distance.0 {
                let Some(positions) = self.0.get(other_ent) else { continue };
                let Ok(min_dists) = min_dists_query.get(*other_ent) else { continue };
                for &prev_pos in positions {
                    if min_dists.check_min_distances(prev_pos, (new_orig_ref, new_dim, new_pos)) == false {
                        return false;
                    }
                }
            }
        }
        self.0.entry(new_orig_ref.0).or_default().push((new_dim, new_pos));

        cmd.entity(new_tile).try_insert(Replicated);

 
        true
    }
}

#[derive(Resource, Debug, Default, Reflect, )]
#[reflect(Resource, Default)]
pub struct TerrGenEntityMap(pub HashIdToEntityMap);

#[derive(Resource, Debug, Default, Reflect, )]
#[reflect(Resource, Default)]
pub struct OpListEntityMap(pub HashIdToEntityMap);


#[derive(AssetCollection, Resource, Default, Reflect)]
#[reflect(Resource, Default)]
pub struct NoiseSerisHandles {
    #[asset(path = "ron/tilemap/terrgen/noise", collection(typed))]
    pub handles: Vec<Handle<NoiseSerialization>>,
}
#[derive(serde::Deserialize, Asset, Reflect, )]
pub struct NoiseSerialization {
    pub id: String,
    /// Default is 0.01
    pub frequency: Option<f32>,
    /// 0: OpenSimplex2, 1: OpenSimplex2S, 2: Cellular, 3: Perlin, 4: ValueCubic, 5: Value
    pub noise_type: Option<u32>,
    /// 0: None, 1: FBm, 2: Ridged, 3: PingPong, 4: DomainWarpProgressive, 5: DomainWarpIndependent,
    pub fractal_type: Option<u32>,
    /// Default is 3
    pub octaves: Option<u8>,
    /// Default is 2.0
    pub lacunarity: Option<f32>,
    /// Default is 0.5
    pub gain: Option<f32>,
    /// Default is 0.0
    pub weighted_strength: Option<f32>,
    /// Default is 2.0
    pub ping_pong_strength: Option<f32>,
    /// 0: Euclidean, 1: EuclideanSq, 2: Manhattan, 3: Hybrid
    pub cellular_distance_function: Option<u32>,
    /// 0: CellValue, 1: Distance, 2: Distance2, 3: Distance2Add, 4: Distance2Sub, 5: Distance2Mul, 6: Distance2Div
    pub cellular_return_type: Option<u32>,
    /// Default is 1.0
    pub cellular_jitter: Option<f32>,
    /// 0: OpenSimplex2, 1: OpenSimplex2Reduced, 2: BasicGrid
    pub domain_warp_type: Option<u32>,
    /// Default is 1.0
    pub domain_warp_amp: Option<f32>,
}


#[derive(AssetCollection, Resource, Default, Reflect)]
#[reflect(Resource, Default)]
pub struct OpListSerisHandles {
    #[asset(path ="ron/tilemap/terrgen/oplist", collection(typed))]
    pub handles: Vec<Handle<OpListSerialization>>,
}
#[derive(serde::Deserialize, Asset, Reflect, Default)]
pub struct OpListSerialization {
    pub id: String,
    pub root_in_dimensions: Vec<String>,
    /// input variable index, operation name, operands, ouput variable indexs 
    pub operation_operands: Vec<(String, Vec<String>, u8)>,
    /// oplist id, produced tiles
    pub bifs: Vec<(String, Vec<String>)>,
    pub size: Option<[u32; 2]>
}
impl OpListSerialization {
    pub fn is_root(&self) -> bool {
        self.root_in_dimensions.iter().any(|s| !s.is_empty())
    }
}




#[derive(Bundle, Debug, Clone, Reflect)]
pub struct TileHelperStruct{
    pub ezero: EntityZeroRef,
    pub global_pos: GlobalTilePos,
    pub dim_ref: DimensionRef,
    pub oplist_size: OplistSize,
    pub tile_bundle: bevy_ecs_tilemap::prelude::TileBundle,
    pub initial_pos: InitialPos,
}


#[derive(Debug, Clone, Resource, Default, Reflect)]
#[reflect(Resource, Default)]
pub struct MassCollectedTiles  (pub Vec<(Entity, TileHelperStruct)>);
impl MassCollectedTiles {

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
            self.clonespawn_and_push_tile(cmd, ezero, global_pos, dim_ref, oplist_size)
        }));
        spawned
    }
    pub fn clonespawn_and_push_tile(
        &mut self,
        cmd: &mut Commands,
        ezero: EntityZeroRef,
        global_pos: GlobalTilePos,
        dim_ref: DimensionRef,
        oplist_size: OplistSize,
    ) -> Entity {
        let tile_instance = cmd.entity(ezero.0).clone_and_spawn_with(|builder|{
            builder.deny::<ToDenyOnTileClone>();
            //builder.deny::<BundleToDenyOnReleaseBuild>();
        }).id();
        let tile_bundle = TileBundle {
            position: global_pos.to_tilepos(oplist_size), ..Default::default()
        };
        let helper = TileHelperStruct {
            ezero,
            global_pos,
            dim_ref,
            oplist_size,
            tile_bundle,
            initial_pos: InitialPos(global_pos),
        };
        self.0.push((tile_instance, helper));
        tile_instance
    }

    fn collect_tiles_rec(
        &mut self,
        cmd: &mut Commands,
        tiling_ent: Entity,
        global_pos: GlobalTilePos,
        dim_ref: DimensionRef,
        oplist_size: OplistSize,
        weight_maps: &Query<(&EntiWeightedSampler,), ()>,
        gen_settings: &AaGlobalGenSettings,
        depth: u32
    ) {
        if let Ok((wmap, )) = weight_maps.get(tiling_ent) {
            if let Some(tiling_ent) = wmap.sample_with_pos(gen_settings, global_pos) {
                if depth > 6 {
                    warn!("Tile insertion depth exceeded 6, stopping recursion for tile {:?}", tiling_ent);
                    return;
                }
                self.collect_tiles_rec(cmd, tiling_ent, global_pos, dim_ref, oplist_size, weight_maps, gen_settings, depth + 1);
            }
        } else {
            self.clonespawn_and_push_tile(cmd, EntityZeroRef(tiling_ent), global_pos, dim_ref, oplist_size);
        }
    }
    pub fn collect_tiles(&mut self, 
        cmd: &mut Commands,
        bif_tiles: &Vec<Entity>, ev: &PendingOp, oplist_size: OplistSize, weight_maps: &Query<(&EntiWeightedSampler,), ()>, gen_settings: &AaGlobalGenSettings,
    )  {
        for tile in bif_tiles.iter().cloned() {
            self.collect_tiles_rec(cmd, tile, ev.pos, ev.dim_ref, oplist_size, weight_maps, gen_settings, 0);
        }
    }

}
