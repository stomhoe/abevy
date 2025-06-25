use std::mem;

use bevy::{platform::collections::{HashMap, HashSet}, prelude::*};
use bevy_ecs_tilemap::{map::*, prelude::MaterialTilemap, tiles::*, TilemapBundle};

use crate::game::{beings::beings_components::Being, factions::factions_components::SelfFaction, tilemap::{terrain_gen::{terrain_gen_components::FnlComp, terrain_gen_resources::WorldGenSettings, terrain_gen_utils::TileDto, terrain_materials::TextureOverlayMaterial}, tilemap_components::*, tilemap_resources::*}};


pub fn visit_chunks_around_activators(
    mut commands: Commands, 
    mut query: Query<(&Transform, &mut ActivatesChunks), (With<SelfFaction>)>,
    mut loaded_chunks: ResMut<LoadedChunks>,
    tilemap_settings: Res<ChunkRangeSettings>,
) {
    let cnt = tilemap_settings.chunk_show_range as i32;   
    for (transform, mut activates_chunks) in query.iter_mut() {
        let center_chunk_pos = contpos_to_chunkpos(transform.translation.xy());

        for y in (center_chunk_pos.y - cnt)..(center_chunk_pos.y + cnt+1) {
            for x in (center_chunk_pos.x - cnt)..(center_chunk_pos.x + cnt+1) {

                let adjacent_chunk_pos = IVec2::new(x, y);

                if ! loaded_chunks.0.contains_key(&adjacent_chunk_pos) {
                    let chunk_ent = commands.spawn((UninitializedChunk(adjacent_chunk_pos), Visibility::Hidden, )).id();
                    loaded_chunks.0.insert(adjacent_chunk_pos, chunk_ent);
                    activates_chunks.0.insert(chunk_ent);
                }
                else {
                    activates_chunks.0.insert(loaded_chunks.0[&adjacent_chunk_pos]);
                }
            }
        }
    }
}

pub fn rem_outofrange_chunks_from_activators(
    mut activator_query: Query<(&Transform, &mut ActivatesChunks), (With<SelfFaction>)>,
    mut chunks_query: Query<(Entity, &Transform), With<Chunk>>,
    tilemap_settings: Res<ChunkRangeSettings>,
) {
    for (transform, mut activate_chunks) in activator_query.iter_mut() {
        for (entity, chunk_transform) in chunks_query.iter_mut() {
            let chunk_cont_pos = chunk_transform.translation.xy();
            let distance = transform.translation.xy().distance(chunk_cont_pos);
            
            if distance > tilemap_settings.chunk_active_max_dist {
              //  println!("Removing chunk at position: {:?}", contpos_to_chunk_pos(&chunk_cont_pos));
                
                activate_chunks.0.remove(&entity);
            }
        }
    }
}

pub fn despawn_unreferenced_chunks(
    mut commands: Commands,
    activator_query: Query<(&ActivatesChunks), (With<SelfFaction>)>,
    mut chunks_query: Query<(Entity, &Transform), With<Chunk>>,
    mut loaded_chunks: ResMut<LoadedChunks>,
) {

    let mut referenced_chunks: HashSet<Entity> = HashSet::new();

    for activates_chunks in activator_query.iter() {
        for chunk_entity in activates_chunks.0.iter() {
            referenced_chunks.insert(*chunk_entity);
        }
    }

    for (entity, chunk_transform) in chunks_query.iter_mut() {
        if !referenced_chunks.contains(&entity) {
            let chunk_cont_pos = chunk_transform.translation.xy();
            let chunk_pos = contpos_to_chunkpos(chunk_cont_pos);

            loaded_chunks.0.remove(&chunk_pos);
            commands.entity(entity).despawn();
        }
    }
}

#[allow(unused_parens)]
pub fn show_chunks_around_camera(
    camera_query: Query<&Transform, With<Camera>>,
    mut chunks_query: Query<&mut Visibility, (With<Chunk>)>,
    loaded_chunks: Res<LoadedChunks>,
    tilemap_settings: Res<ChunkRangeSettings>,
) {
    let cnt = tilemap_settings.chunk_show_range as i32;   
    for transform in camera_query.iter() {
        let camera_chunk_pos = contpos_to_chunkpos(transform.translation.xy());
        for y in (camera_chunk_pos.y - cnt)..(camera_chunk_pos.y + cnt+1) {
            for x in (camera_chunk_pos.x - cnt)..(camera_chunk_pos.x + cnt+1) {
                
                let adj_chunk_pos = IVec2::new(x, y);

                loaded_chunks.0.get(&adj_chunk_pos).map(|ent| {
                    if let Ok(mut visibility) = chunks_query.get_mut(*ent) {
                        *visibility = Visibility::Visible;
                    }
                });
            }
        }
    }
}

pub fn hide_outofrange_chunks(
    camera_query: Query<&Transform, With<Camera>>,
    mut chunks_query: Query<(&Transform, &mut Visibility), With<Chunk>>,
    tilemap_settings: Res<ChunkRangeSettings>,
) {
    for camera_transform in camera_query.iter() {
        for (chunk_transform, mut visibility) in chunks_query.iter_mut() {
            let chunk_cont_pos = chunk_transform.translation.xy();

            let distance = camera_transform.translation.xy().distance(chunk_cont_pos);
            
            if distance > tilemap_settings.chunk_visib_max_dist {
                *visibility = Visibility::Hidden;
            }
        }
    }
}

#[allow(unused_parens)]
pub fn add_tilemaps_to_chunk(
    mut commands: Commands, 
    asset_server: Res<AssetServer>, 
    //texturess: Res<Textures>,
    gen_settings: Res<WorldGenSettings>, 
    chunks_query: Query<(Entity, &UninitializedChunk)>, 
    noise_query: Query<&FnlComp>, 
    mut materials: ResMut<Assets<TextureOverlayMaterial>>
) {
    for (chunk_ent, chunk) in chunks_query.iter() {
       
        commands.entity(chunk_ent).remove::<UninitializedChunk>();
        commands.entity(chunk_ent).insert((Chunk(chunk.0), Transform::from_translation(chunkpos_to_pixelpos(chunk.0).extend(0.0)) ));

        produce_tilemaps(&mut commands, &asset_server, &gen_settings, chunk_ent, chunk.0, noise_query, &mut materials);
    }
}


type Map = HashMap<(i32, UVec2), LayerDto>;

pub fn tmaptsize_to_uvec2(tile_size: TilemapTileSize) -> UVec2 {
    UVec2::new(tile_size.x as u32, tile_size.y as u32)
}
pub fn uvec2_to_tmaptsize(tile_size: UVec2) -> TilemapTileSize {
    TilemapTileSize { x: tile_size.x as f32, y: tile_size.y as f32 }
}

#[allow(unused_parens)]
fn produce_tilemaps(
    commands: &mut Commands, 
    asset_server: &AssetServer, 
    gen_settings: &WorldGenSettings,
    chunk_ent: Entity, 
    chunk_pos: IVec2, 
    noise_query: Query<&FnlComp>,
    mut materials: &mut Assets<TextureOverlayMaterial>
) {
    let mut layers: Map = HashMap::new();

    for mut tile_dto in super::terrain_gen::gather_all_tiles2spawn_within_chunk(commands, &asset_server, noise_query, gen_settings, chunk_pos) {
        let tilepos_within_chunk = TilePos::from(tile_dto.pos_within_chunk);

        if let Some(layer_dto) = layers.get_mut(&(tile_dto.layer_z, tmaptsize_to_uvec2(tile_dto.tile_size))) 
        {
            let (&mut tilemap_entity, tile_storage, handles) = (
                &mut layer_dto.tilemap_entity,
                &mut layer_dto.tile_storage,
                &mut layer_dto.handles,
            );


            let texture_index = match handles.iter().position(|x| *x == tile_dto.used_handle) {
                Some(index) => TileTextureIndex(index as u32),
                None => {
                    handles.push(mem::take(&mut tile_dto.used_handle));
                    TileTextureIndex((handles.len() - 1) as u32)
                }
            };

            insert_tile_into_storage(commands, &tile_dto, tilemap_entity, tile_storage, texture_index, tilepos_within_chunk,);

            layer_dto.needs_y_sort = layer_dto.needs_y_sort || tile_dto.needs_y_sort;
        } else{
            instantiate_new_layer_dto(commands, &mut layers, tile_dto);
        }
    }
    
    for (&(layer_z, tile_size), layer_dto) in layers.iter_mut() {
        super::terrain_gen::fill_tilemap_data(
            commands, asset_server, layer_dto.tilemap_entity,
            TilemapTexture::Vector(mem::take(&mut layer_dto.handles)),
            chunk_ent, chunk_pos, mem::take(&mut layer_dto.tile_storage),
            layer_z, layer_dto.needs_y_sort,
            uvec2_to_tmaptsize(tile_size),
            &mut materials,
        );
    }
}







struct LayerDto {
    pub tilemap_entity: Entity, pub tile_storage: TileStorage,
    pub handles: Vec<Handle<Image>>, pub needs_y_sort: bool,
}

fn insert_tile_into_storage( 
    commands: &mut Commands, tileinfo: &TileDto, 
    tilemap_entity: Entity,  tile_storage: &mut TileStorage,
    texture_index: TileTextureIndex, pos_within_chunk: TilePos,
)  {
    let tile_bundle = TileBundle {
        position: pos_within_chunk, tilemap_id: TilemapId(tilemap_entity),
        texture_index,  flip: tileinfo.flip, color: tileinfo.color,
        visible: tileinfo.visible, ..Default::default()
    };
    let tile_entity: Entity = if let Some(preexisting_tile_entity) = tileinfo.entity {
        commands.entity(preexisting_tile_entity).insert(tile_bundle);
        preexisting_tile_entity
    } else {
        commands
            .spawn(tile_bundle)
            .id()
    };
    tile_storage.set(&pos_within_chunk, tile_entity);
}

fn instantiate_new_layer_dto(
    commands: &mut Commands, 
    layers: &mut Map,
    tile_dto: TileDto,
) {
    let tilemap_entity: Entity = commands.spawn_empty().id();
    let mut tile_storage = TileStorage::empty(CHUNK_SIZE.into());

    insert_tile_into_storage(
        commands, &tile_dto, tilemap_entity, &mut tile_storage,
        TileTextureIndex(0), TilePos::from(tile_dto.pos_within_chunk),
    );

    let layer_dto = LayerDto {
        tilemap_entity,
        tile_storage,
        handles: vec![tile_dto.used_handle],
        needs_y_sort: tile_dto.needs_y_sort,
    };
    layers.insert((tile_dto.layer_z, tmaptsize_to_uvec2(tile_dto.tile_size)), layer_dto);
}
