use bevy::{ecs::{entity::EntityHashMap, entity_disabling::Disabled}, platform::collections::HashSet};
#[allow(unused_imports)] use bevy::prelude::*;
use bevy_ecs_tilemap::tiles::TileColor;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
#[allow(unused_imports)] use bevy_asset_loader::prelude::*;
use bevy_replicon::shared::server_entity_map::ServerEntityMap;
use bevy_replicon_renet::renet::{RenetClient, RenetServer};
use common::common_components::{DisplayName, EntityPrefix, ImageHolder, ImageHolderMap, StrId};
use game_common::{color_sampler_resources::ColorWeightedSamplersMap, game_common_components::{Category, MyZ, YSortOrigin}, game_common_components_samplers::{ColorSamplerRef, WeightedSamplerRef}};
use bevy_ecs_tilemap::tiles::TilePos;

use crate::{tile::{tile_components::*, tile_resources::*, tile_materials::*}, };
use std::mem::take;

#[allow(unused_parens)]
pub fn init_tiles(
    mut cmd: Commands,  asset_server: Res<AssetServer>,
    mut seris_handles: ResMut<TileSerisHandles>, mut assets: ResMut<Assets<TileSeri>>,
    shader_map: Res<TileShaderEntityMap>,
    tiling_map: Option<Res<TileEntitiesMap>>,
    color_map: Res<ColorWeightedSamplersMap>,

) {
    if tiling_map.is_some() { return; }
    cmd.insert_resource(TileEntitiesMap::default());

    let mut tile_cats = TileCategories::default();

    for handle in seris_handles.handles.iter() {
        //info!("Loading TileSeri from handle: {:?}", handle);
        let Some(seri) = assets.get_mut(handle) else { continue; };

        let str_id = match StrId::new_with_result(seri.id.clone(), Tile::MIN_ID_LENGTH) {
            Ok(id) => id,
            Err(err) => {
                error!("Failed to create StrId for tile '{}': {}", seri.id, err);
                continue;
            }
        };
        let my_z = MyZ(seri.z);
        let enti = cmd.spawn((
            Tile, str_id.clone(), Disabled,
            my_z.clone(),
        )).id();

        let [r, g, b, a] = seri.color.unwrap_or([255, 255, 255, 255]);
        let color = Color::srgba_u8(r, g, b, a);

        if ! seri.name.is_empty() {
            cmd.entity(enti).insert(DisplayName(seri.name.clone()));
        }
        if seri.tmapchild {
            cmd.entity(enti).insert(ChunkOrTilemapChild);
        }


        if seri.img_paths.is_empty() {
            warn!("Tile '{}' has no images", str_id);
        }

        if ! seri.color_map.is_empty() {
            match color_map.0.get(&seri.color_map) {
                Ok(color_sampler_ent) => {
                    cmd.entity(enti).insert(ColorSamplerRef(color_sampler_ent));
                }
                Err(err) => {
                    error!("Tile '{}': Weighted color sampler with id '{}' not found: {}", str_id, seri.color_map, err);
                }
            }
        }

        if seri.randflipx {
            cmd.entity(enti).insert(FlipAlongX);
        }

        //TODO HACER Q LAS TILES PUEDAN TENER MUCHAS IMÁGENES (PARA IR CAMBIANDO ENTRE ELLAS SEGÚN EL ESTADO, USANDO EL INDEX)
        if ! seri.sprite {
            let tile_handles = TileHidsHandles::from_paths(&asset_server, take(&mut seri.img_paths), );

            if let Ok(tile_handles) = tile_handles {
                cmd.entity(enti).insert(tile_handles);
            } else{
                error!("Failed to create TileHandles for tile '{}'", str_id);
            }

            cmd.entity(enti).insert((TileColor::from(color)));
            if seri.shader.len() > 2 {
                match shader_map.0.get(&seri.shader) {
                    Ok(shader_ent) => {
                        cmd.entity(enti).insert(TileShaderRef(shader_ent));
                    }
                    Err(err) => {
                        warn!("Tile '{}' references missing shader '{}': {}", str_id, seri.shader, err);
                    }
                }
            } else if seri.shader.len() > 0 {
                warn!("Tile {} shader {} is too short for a shader", str_id, seri.shader);
            }
        }
        else{
            let map = match ImageHolderMap::from_paths(&asset_server, take(&mut seri.img_paths)) {
                Ok(map) => map,
                Err(err) => {
                    error!("Failed to create ImageHolderMap for tile '{}': {}", str_id, err);
                    continue;
                }
            };
            let offset = Vec2::from_array(seri.offset);

            cmd.entity(enti).insert((
                Sprite{
                    image: map.first_handle(),
                    color,
                    ..Default::default()
                },
                map,
                Transform::from_translation(offset.extend(my_z.as_float())),
            ));
            if ! seri.shader.is_empty() {
                warn!("Tile {} tilemap shaders ('{}') are not compatible with sprite=true, ignoring", str_id, seri.shader);
            }
            if let Some(y_sort_origin) = seri.ysort {
                cmd.entity(enti).insert(YSortOrigin(offset.y + y_sort_origin - 10.0));
            }
        }
        if !seri.cats.is_empty() {
            for cat in seri.cats.iter() {
                if cat.is_empty() { continue; }
                tile_cats.0.entry(Category::new(cat)).or_default().push(enti);
            }
        }
    }
    
    cmd.insert_resource(tile_cats);

} 

pub fn add_tiles_to_map(
    mut cmd: Commands,
    map: Option<ResMut<TileEntitiesMap>>,
    query: Query<(Entity, &EntityPrefix, &StrId), (Added<Tile>, Added<Disabled>, Without<TilePos>)>,
) {
    if let Some(mut map) = map {
        for (ent, prefix, str_id) in query.iter() {
            if let Err(err) = map.0.insert(str_id, ent, ) {
                error!("{} {} already in TilingEntityMap : {}", prefix, str_id, err);
                cmd.entity(ent).despawn();
            } else {
                info!("Inserted tile '{}' into TilingEntityMap with entity {:?}", str_id, ent);
            }
        }
    }
}

#[allow(unused_parens)]
pub fn map_min_dist_tiles(mut cmd: Commands, 
    mut seris_handles: ResMut<TileSerisHandles>, mut assets: ResMut<Assets<TileSeri>>,
    map: ResMut<TileEntitiesMap>,
    tile_cats: Res<TileCategories>,
) {
    let mut keep_away: EntityHashMap<HashSet<Entity>> = EntityHashMap::default();

    for handle in seris_handles.handles.drain(..) {
        let Some(seri) = assets.remove(&handle) else { continue; };

        let Some(min_distances) = seri.min_distances else { continue; };

        if min_distances.is_empty() { continue; }

        let Ok(tile_ent) = map.0.get(&seri.id) else { continue; };

        let mut min_dists = MinDistancesMap::default();

        for (tile_id, min_dist) in min_distances {

            if let Some(cat) = tile_id.strip_prefix("c.") && let Some(cat_entities) = tile_cats.0.get(&Category::new(cat)) {

                for cat_tile_ent in cat_entities {
                    min_dists.0.insert(*cat_tile_ent, min_dist);
                    if cat_tile_ent != &tile_ent {
                        keep_away.entry(*cat_tile_ent).or_default().insert(tile_ent);
                    }
                }
            }
            else if let Ok(other_tile_ent) = map.0.get(&tile_id) {
                min_dists.0.insert(other_tile_ent, min_dist);
                if other_tile_ent != tile_ent {
                    keep_away.entry(other_tile_ent).or_default().insert(tile_ent);
                }
            } else {
                warn!("Tile '{}' min_distances references unknown tile id '{}'", seri.id, tile_id);
                continue;
            };
        }

        if min_dists.0.is_empty() { continue; }
        
        cmd.entity(tile_ent).insert(min_dists);
    }

    for (tile_ent, ents) in keep_away {
        cmd.entity(tile_ent).insert(KeepDistanceFrom(ents.into_iter().collect()));
    }
}


#[allow(unused_parens, )]
pub fn client_map_server_tiling(
    trigger: Trigger<TileEntitiesMap>, 
    server: Option<Res<RenetServer>>,
    mut entis_map: ResMut<ServerEntityMap>, 
    own_map: Res<TileEntitiesMap>,
) {
    if server.is_some() { return; }

    let TileEntitiesMap(received_map) = trigger.event().clone();
    for (hash_id, &server_entity) in received_map.0.iter() {
        if let Ok(client_entity) = own_map.0.get_with_hash(hash_id) {

            debug!("Mapping server entity {:?} to local entity {:?}", server_entity, client_entity);
            entis_map.insert(server_entity, client_entity);
        } else {
            error!("Received entity {:?} with hash id {:?} not found in own map", server_entity, hash_id);
        }
    }
}
