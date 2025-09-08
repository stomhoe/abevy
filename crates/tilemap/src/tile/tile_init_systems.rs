use bevy::{ecs::{entity::{EntityHashMap, EntityHashSet}, entity_disabling::Disabled}, platform::collections::{HashMap, HashSet}, render::sync_world::SyncToRenderWorld};
#[allow(unused_imports)] use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
#[allow(unused_imports)] use bevy_asset_loader::prelude::*;
use bevy_replicon::shared::server_entity_map::ServerEntityMap;
use bevy_replicon_renet::renet::{RenetClient, RenetServer};
use common::common_components::{AssetScoped, DisplayName, EntityPrefix, HashId, ImageHolder, ImageHolderMap, StrId};
use ::dimension_shared::*;
use game_common::{color_sampler_resources::ColorWeightedSamplersMap, game_common_components::{Category, EntityZeroRef, MyZ, SearchingForSuitablePos, YSortOrigin}, game_common_components_samplers::{ColorSamplerRef, WeightedSamplerRef}};
use bevy_ecs_tilemap::tiles::TilePos;
use ::tilemap_shared::*;

use crate::{chunking_resources::LoadedChunks, terrain_gen::{terrgen_events::*, terrgen_resources::RegisteredPositions}, tile::{tile_components::*,  tile_materials::*, tile_resources::*} };
use std::mem::take;

#[derive(Component, Debug, Default, )]
#[require(AssetScoped, EntityPrefix::new("Tiles' Templates"), Name, Transform, Visibility)]
struct EguiTileTemplatesHolder;

#[derive(Component, Debug, Default, )]
#[require(AssetScoped, EntityPrefix::new("Portal tiles"), Name, Transform, Visibility)]
struct EguiPortalTileTemplatesHolder;

#[allow(unused_parens)]
pub fn init_tiles(
    mut cmd: Commands,  asset_server: Res<AssetServer>,
    seris_handles: Res<TileSerisHandles>, mut assets: ResMut<Assets<TileSeri>>,
    shader_map: Res<TileShaderEntityMap>,
    tiling_map: Option<Res<TileEntitiesMap>>,
    color_map: Res<ColorWeightedSamplersMap>,

) {
    if tiling_map.is_some() { return; }
    cmd.insert_resource(TileEntitiesMap::default());
    let holder = cmd.spawn((EguiTileTemplatesHolder, )).id();
    cmd.spawn((TileInstancesHolder, ChildOf(holder)));

    let egui_portal_holder = cmd.spawn((EguiPortalTileTemplatesHolder, ChildOf(holder))).id();

    let mut tile_cats = TileCategories::default();

    for handle in seris_handles.handles.iter() {
        //info!("Loading TileSeri from handle: {:?}", handle);
        let Some(mut seri) = assets.get_mut(handle) else { continue; };

        let str_id = match TileStrId::new_with_result(seri.id.clone(), Tile::MIN_ID_LENGTH) {
            Ok(id) => id,
            Err(err) => {
                error!("Failed to create TileStrId for tile '{}': {}", seri.id, err);
                continue;
            }
        };
        let my_z = MyZ(seri.z);
        let enti = cmd.spawn((
            Tile, str_id.clone(), Disabled,
            EntityPrefix::new("Tile"), Name::default(),
            my_z.clone(),
            ChildOf(holder),
            DimensionRef(Entity::PLACEHOLDER), 
        )).id();

        let [r, g, b, a] = seri.color.unwrap_or([255, 255, 255, 255]);
        let color = Color::srgba_u8(r, g, b, a);

        if ! seri.name.is_empty() {
            cmd.entity(enti).insert(DisplayName(seri.name.clone()));
        }
        if seri.tmapchild && seri.portal.is_none() {
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

        if let Some(portal) = &mut seri.portal { 
            cmd.entity(enti).insert((take(portal), ChildOf(egui_portal_holder))); 
        }

        if ! seri.sprite && seri.tmapchild {
            let tile_handles = TileHidsHandles::from_paths(&asset_server, take(&mut seri.img_paths), );

            if let Ok(tile_handles) = tile_handles {
                cmd.entity(enti).insert(tile_handles);
            } else{
                error!("Failed to create TileHandles for tile '{}'", str_id);
            }

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

            cmd.entity(enti).insert_if_new((TileColor::from(color), ));
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
    query: Query<(Entity, &EntityPrefix, &TileStrId), (Added<Tile>, Added<Disabled>, Without<TilePos>, Without<EntityZeroRef>)>,
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
    tiles_map: Res<TileEntitiesMap>,
    tile_cats: Res<TileCategories>,
) {
    let mut keep_away: EntityHashMap<HashSet<Entity>> = EntityHashMap::default();

    for handle in seris_handles.handles.drain(..) {
        let Some(seri) = assets.remove(&handle) else { continue; };

        let Some(min_distances) = seri.min_distances else { continue; };

        if min_distances.is_empty() { continue; }

        let Ok(tile_ent) = tiles_map.0.get(&seri.id) else { continue; };

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
            else if let Ok(other_tile_ent) = tiles_map.0.get(&tile_id) {
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

#[allow(unused_parens)]
pub fn map_portal_tiles(mut cmd: Commands, 
    query: Query<(Entity, &TileStrId, &PortalSeri, ),(With<Disabled>)>,
    tiles_map: Res<TileEntitiesMap>,
) {
    info!("Mapping portal tiles");
    for (ent, str_id, portal_seri) in query.iter() {
        let Ok(tile_ent) = tiles_map.0.get(&portal_seri.oe_tile) else { 
            error!("Portal tile {} to '{}' references unknown oe_tile '{}'", str_id, portal_seri.dest_dimension, portal_seri.oe_tile);
            continue; 
        };
        info!("Mapping portal tile '{}' to destination dimension '{}'", str_id, portal_seri.dest_dimension);
        cmd.entity(ent).insert(PortalTemplate{
            dest_dimension: Entity::PLACEHOLDER,
            root_oplist: Entity::PLACEHOLDER, //SETEARLO DESPUÉS
            oe_portal_tile: tile_ent,
            checked_oplist: Entity::PLACEHOLDER, //SETEARLO DESPUÉS
            op_i: portal_seri.op_i,
            lim_below: portal_seri.lim_below,
            lim_above: portal_seri.lim_above,
        });
    }
}

#[allow(unused_parens, )]
pub fn client_map_server_tiling(
    trigger: Trigger<TileEntitiesMap>, 
    mut cmd: Commands, 
    server: Option<Res<RenetServer>>,
    mut entis_map: ResMut<ServerEntityMap>, 
    own_map: Res<TileEntitiesMap>,
) {
    if server.is_some() { return; }

    let TileEntitiesMap(received_map) = trigger.event().clone();
    for (hash_id, &server_entity) in received_map.0.iter() {
        
        if let Ok(client_entity) = own_map.0.get_with_hash(hash_id) {
            if let Some(prev_client_entity) = entis_map.server_entry(server_entity).get() 
                && client_entity != prev_client_entity 
            {
                cmd.entity(prev_client_entity).try_despawn();
            }
            debug!("Mapping server entity {:?} to local entity {:?}", server_entity, client_entity);
            entis_map.insert(server_entity, client_entity);
        } else {
            error!("Received entity {:?} with hash id {:?} not found in own map", server_entity, hash_id);
        }
    }
}

#[allow(unused_parens)]
pub fn instantiate_portal(mut cmd: Commands,
    ori_tile_str_id_query: Query<&TileStrId, (With<Disabled>)>,
    new_portals: Query<(Entity, &PortalTemplate, &GlobalTilePos, &DimensionRef, &EntityZeroRef),(Without<SearchingForSuitablePos>, )>,
    pending_search: Query<(Entity, &SearchingForSuitablePos, &PortalTemplate, &GlobalTilePos, &DimensionRef, &EntityZeroRef),()>,
    dimension_query: Query<&HashId, (With<Dimension>, )>,
    mut ew_pos_search: EventWriter<PosSearch>, 
    mut ewriter_tiles: EventWriter<MassCollectedTiles>,
    mut ereader_search_successful: EventReader<SuitablePosFound>,
    mut ereader_search_failed: EventReader<SearchFailed>, 
    mut register_pos: ResMut<RegisteredPositions>

) {
    let mut started_searches: EntityHashMap<Entity> = EntityHashMap::new();
    let mut pos_searches = Vec::new();

    for (portal_ent, portal_template, &global_pos, dim_ref, tile_ref) in new_portals.iter() {

        let studied_op = portal_template.to_studied_op(global_pos);

        let str_id = ori_tile_str_id_query.get(tile_ref.0).map(|id| id.as_str()).unwrap_or_default();

        let Ok(&dimension_hash_id) = dimension_query.get(portal_template.dest_dimension) else {
            error!(
                "PortalTemplate {} (entity: {:?}) references a DestDimension that doesn't exist ({:?}). Entity's own dimension: {:?}, pos: {:?}, ", str_id, portal_ent, portal_template.dest_dimension, dim_ref.0, global_pos,
            );
            cmd.entity(portal_ent).remove::<PortalTemplate>();
            continue;
        };


        let studied_op_ent = cmd.spawn((studied_op.clone(), )).id();

        cmd.entity(portal_ent).try_insert(SearchingForSuitablePos{ studied_op_ent });

        pos_searches.push(PosSearch::portal_pos_search(dimension_hash_id, studied_op_ent, global_pos));
        started_searches.insert(studied_op_ent, portal_ent);
    }

    let mut successful_searches: EntityHashSet = EntityHashSet::new();

    let mut handle_success = |ent: Entity, portal_template: &PortalTemplate, my_pos: GlobalTilePos, 
        found_pos: GlobalTilePos, my_dim_ref: DimensionRef, my_orig_tile_ref: EntityZeroRef| 
    {
        cmd.entity(ent).remove::<(SearchingForSuitablePos, PortalTemplate)>();
        register_pos.0.entry(portal_template.oe_portal_tile)
            .or_default()
            .push((DimensionRef(portal_template.dest_dimension), found_pos));
        cmd.entity(ent).insert(PortalInstance::new(portal_template.dest_dimension, found_pos));

        let oe_portal_tileref = if portal_template.oe_portal_tile == ent {
            my_orig_tile_ref
        } else {
            EntityZeroRef(portal_template.oe_portal_tile)
        };

        let oe_portal = Tile::spawn_from_ref(&mut cmd, oe_portal_tileref, found_pos, OplistSize::default());
        info!("Instantiated portal tile '{}' at position {:?} in dimension {:?}", oe_portal, found_pos, portal_template.dest_dimension);

        cmd.entity(oe_portal).remove::<PortalTemplate>().insert(PortalInstance::new(my_dim_ref.0, my_pos));
        let mut collected = MassCollectedTiles::new(1);
        // collected.collect_tiles(&mut cmd, oe_portal, ) ;

        // ewriter_tiles.write(MassCollectedTiles(oe_portal, portal_template.dest_dimension, ));
    };

    'successful_searches: for search_successful_ev in ereader_search_successful.read() {
        let studied_op_ent = search_successful_ev.studied_op_ent;
        if successful_searches.contains(&studied_op_ent) {
            continue 'successful_searches;
        }

        if let Some(portal_ent) = started_searches.remove(&studied_op_ent) {
            let Ok((_, portal_template, &my_pos, &dim_ref, &orig_tile_ref)) = new_portals.get(portal_ent) else {
                continue 'successful_searches;
            };
            successful_searches.insert(studied_op_ent.clone());
            handle_success(portal_ent, portal_template, my_pos, search_successful_ev.found_pos.clone(), dim_ref, orig_tile_ref);
            continue 'successful_searches;
        }

        for (ent, searching_for, portal_template, &my_pos, &dim_ref, &orig_tile_ref) in pending_search.iter() {
            if studied_op_ent == searching_for.studied_op_ent {
                let str_id = ori_tile_str_id_query.get(orig_tile_ref.0).map(|id| id.as_str()).unwrap_or_default();
                info!(
                    "Found suitable pos for portal tile {} (entity: {:?}) self's dimension and pos: ({:?}, {:?}), DestDimension: {:?}, found pos: {:?}", str_id, ent, dim_ref.0, my_pos, portal_template.dest_dimension, search_successful_ev.found_pos
                );
                successful_searches.insert(studied_op_ent.clone());
                handle_success(ent, portal_template, my_pos, search_successful_ev.found_pos.clone(), dim_ref, orig_tile_ref);
                continue 'successful_searches;
            }
        }
    }

    for ev in ereader_search_failed.read() {
        if successful_searches.contains(&ev.0) { continue; }

        if started_searches.remove(&ev.0).is_some() {
            error!("Failed to find suitable pos for a portal tile, {:?}", ev.0);
            continue;
        }

        for (ent, searching_for, portal_template, &global_pos, dim_ref, tile_ref) in pending_search.iter() {
            let str_id = ori_tile_str_id_query.get(tile_ref.0).map(|id| id.as_str()).unwrap_or_default();
            if ev.0 == searching_for.studied_op_ent {
                error!(
                    "Failed to find suitable pos for portal tile {} (entity: {:?}) self's dimension and pos: ({:?}, {:?}), DestDimension: {:?}", str_id, ent, dim_ref.0, global_pos, portal_template.dest_dimension
                );
                cmd.entity(ent).remove::<SearchingForSuitablePos>();
            }
        }
    }
    ew_pos_search.write_batch(pos_searches);
}

#[allow(unused_parens)]
pub fn client_sync_tile(
    mut cmd: Commands, 
    query: Query<(Entity, &EntityZeroRef, &GlobalTilePos, &DimensionRef, ), (Added<Replicated>, With<Tile>)>,
    ori_query: Query<(&TileStrId, Has<ChunkOrTilemapChild>, Option<&Sprite>), (With<Disabled>)>,
    loaded_chunks: Res<LoadedChunks>,
    mut ewriter_tmap_process: EventWriter<MassCollectedTiles>

) {
    let mut tiles_to_tmap_process = Vec::new();
    for (tile_ent, &orig_ref, &global_pos, &dim_ref, ) in query.iter() {
        let Ok((tile_strid, is_child, sprite)) = ori_query.get(orig_ref.0) else{
            error!("Original tile entity {} is despawned", orig_ref.0);
            continue;
        };

        if let Some(sprite) = sprite {
            cmd.entity(tile_ent).try_insert(sprite.clone());
        }

        let chunk_pos: ChunkPos = global_pos.into();

        
        if is_child && let Some(&chunk) = loaded_chunks.0.get(&(dim_ref, chunk_pos)) {
            //let tiles = OplistCollectedTiles::new(tile_ent, );
            //tiles_to_tmap_process.push(Tiles2TmapProcess{tiles, chunk,});
            cmd.entity(tile_ent).try_insert((ChildOf(chunk), SyncToRenderWorld));
            
        } else{
            cmd.entity(tile_ent).try_insert(ChildOf(dim_ref.0));
        }
    }
    ewriter_tmap_process.write_batch(tiles_to_tmap_process);

}
