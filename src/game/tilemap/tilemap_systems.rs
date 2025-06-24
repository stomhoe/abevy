use std::mem;

use bevy::{platform::collections::{HashMap, HashSet}, prelude::*};
use bevy_ecs_tilemap::{map::*, prelude::MaterialTilemap, tiles::*, TilemapBundle};

use crate::game::{beings::beings_components::Being, factions::factions_components::SelfFaction, tilemap::{formation_generation::{formation_generation_components::FnlComp, formation_generation_resources::WorldGenSettings, formation_generation_utils::{gather_all_tiles2spawn_within_chunk, TileInfo, }}, tilemap_components::*, tilemap_resources::*}};


pub fn visit_chunks_around_activators(
    mut commands: Commands, 
    mut query: Query<(&Transform, &mut ActivatesChunks), (With<SelfFaction>)>,
    mut loaded_chunks: ResMut<LoadedChunks>,
    tilemap_settings: Res<ChunkRangeSettings>,
    asset_server: Res<AssetServer>, 
    gen_settings: Res<WorldGenSettings>,
    noise_query: Query<&FnlComp>,
) {
    let cnt = tilemap_settings.chunk_show_range as i32;   
    for (transform, mut activates_chunks) in query.iter_mut() {
        let center_chunk_pos = contpos_to_chunkpos(transform.translation.xy());

        for y in (center_chunk_pos.y - cnt)..(center_chunk_pos.y + cnt+1) {
            for x in (center_chunk_pos.x - cnt)..(center_chunk_pos.x + cnt+1) {

                let adjacent_chunk_pos = IVec2::new(x, y);

                if ! loaded_chunks.0.contains_key(&adjacent_chunk_pos) {
                    let chunk_ent = commands.spawn((Chunk(adjacent_chunk_pos), Visibility::Hidden, )).id();
                    loaded_chunks.0.insert(adjacent_chunk_pos, chunk_ent);
                    activates_chunks.0.insert(chunk_ent);
                    commands.entity(chunk_ent).insert(Transform::from_translation(chunkpos_to_pixelpos(adjacent_chunk_pos).extend(0.0)));
                    produce_tilemaps(&mut commands, &asset_server, &gen_settings, chunk_ent, adjacent_chunk_pos, noise_query);
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
struct LayerInfo {
    pub tilemap_entity: Entity, pub tile_storage: TileStorage,
    pub handles: Vec<Handle<Image>>, pub needs_y_sort: bool,
}

fn instantiate_tile_bundle(
    tileinfo: &TileInfo, tilemap_entity: Entity, 
    texture_index: TileTextureIndex, pos_within_chunk: TilePos,
) -> TileBundle {
    TileBundle {
        position: pos_within_chunk,
        tilemap_id: TilemapId(tilemap_entity),
        texture_index, 
        flip: tileinfo.flip,
        color: tileinfo.color,
        visible: tileinfo.visible,
        ..Default::default()
    }
}

fn do_stuff_in_common( 
    commands: &mut Commands, tileinfo: &TileInfo, 
    tilemap_entity: Entity,  tile_storage: &mut TileStorage,
    texture_index: TileTextureIndex, pos_within_chunk: TilePos,
)  {
    let tile_bundle = instantiate_tile_bundle(tileinfo, tilemap_entity, texture_index, pos_within_chunk);
    let tile_entity: Entity = if let Some(existing_entity) = tileinfo.entity {
        commands.entity(existing_entity).insert(tile_bundle);
        existing_entity
    } else {
        commands
            .spawn(tile_bundle)
            .id()
    };
    tile_storage.set(&pos_within_chunk, tile_entity);
}

#[allow(unused_parens)]
fn produce_tilemaps(
    commands: &mut Commands, 
    asset_server: &AssetServer, 
    gen_settings: &WorldGenSettings,
    chunk_ent: Entity, 
    chunk_pos: IVec2, 
    noise_query: Query<&FnlComp>,
) {
    let mut layers: HashMap<i16, LayerInfo> = HashMap::new();

    for mut tileinfo in gather_all_tiles2spawn_within_chunk(commands, &asset_server, noise_query, gen_settings, chunk_pos) {
        let tilepos_within_chunk = TilePos {x: tileinfo.pos_within_chunk.x, y: tileinfo.pos_within_chunk.y, };
        if let Some(layer_info) = layers.get_mut(&tileinfo.layer_z)
        {
            let (&mut tilemap_entity, tile_storage, handles) = (
                &mut layer_info.tilemap_entity,
                &mut layer_info.tile_storage,
                &mut layer_info.handles,
            );


            let texture_index = match handles.iter().position(|x| *x == tileinfo.used_handle) {
                Some(index) => TileTextureIndex(index as u32),
                None => {
                    handles.push(mem::take(&mut tileinfo.used_handle));
                    TileTextureIndex((handles.len() - 1) as u32)
                }
            };

            do_stuff_in_common(commands, &tileinfo, tilemap_entity, tile_storage, texture_index, tilepos_within_chunk,);

            layer_info.needs_y_sort = layer_info.needs_y_sort || tileinfo.needs_y_sort;
        } else{
            let tilemap_entity: Entity = commands.spawn_empty().id();
            let mut tile_storage = TileStorage::empty(CHUNK_SIZE.into());

            do_stuff_in_common(
                commands, &tileinfo, tilemap_entity, &mut tile_storage,
                TileTextureIndex(0), tilepos_within_chunk,
            );

            let handles = vec![mem::take(&mut tileinfo.used_handle)];
            let layer_info = LayerInfo {
                tilemap_entity,
                tile_storage,
                handles,
                needs_y_sort: tileinfo.needs_y_sort,
            };
            
            layers.insert(tileinfo.layer_z, layer_info);
        }
    }
    
    for (&layer_z, layer_info) in layers.iter_mut() {
        make_tilemap(
            commands, layer_info.tilemap_entity,
            TilemapTexture::Vector(mem::take(&mut layer_info.handles)),
            chunk_ent, chunk_pos, mem::take(&mut layer_info.tile_storage),
            layer_z, layer_info.needs_y_sort,
        );
    }
}

const RENDER_CHUNK_SIZE: UVec2 = UVec2 {x: CHUNK_SIZE.x * 2, y: CHUNK_SIZE.y * 2,};
const TILEMAP_GRID_SIZE: TilemapGridSize = TilemapGridSize { x: TILE_SIZE_PXS.x as f32, y: TILE_SIZE_PXS.y as f32 };
pub const TILEMAP_TILE_SIZE: TilemapTileSize = TilemapTileSize { x: TILE_SIZE_PXS.x as f32, y: TILE_SIZE_PXS.y as f32 };


#[allow(unused_parens)]
fn make_tilemap(commands: &mut Commands, 
    tmap_entity: Entity,
    texture: TilemapTexture, 
    chunk_ent: Entity, 
    _chunk_pos: IVec2,//lo dejo por si lo necesita algún shader
    storage: TileStorage,
    layer_z_level: i16, y_sort: bool, 
    //mut materials: ResMut<Assets<MyMaterial>>,//buscar formas alternativas de agarrar shaders
)
{
    commands.entity(tmap_entity).insert(ChildOf(chunk_ent));
    let mut ent_commands = commands.entity(tmap_entity);
    let grid_size = TILEMAP_GRID_SIZE; let size = CHUNK_SIZE.into();  
    let tile_size = TILEMAP_TILE_SIZE;
    let transform = Transform::from_translation(Vec3::new(0.0, 0.0, layer_z_level as f32));
    let render_settings = TilemapRenderSettings {render_chunk_size: RENDER_CHUNK_SIZE, y_sort,};

    match layer_z_level {
        20/*z level del agua x ejemplo  */ => {

            // let material = MaterialTilemapHandle::from(materials.add(MyMaterial {
            //     brightness: 0.5,
            //     ..default()
            // }));

            // ent_commands.insert((
            //     MaterialTilemapBundle {grid_size, size, storage, texture, tile_size, transform, render_settings, 
            //         material, ..Default::default()
            //     },
            // ));
        },
        _ => {
            ent_commands.insert(
                TilemapBundle {grid_size, size, storage, texture, tile_size, transform, render_settings, ..Default::default()}
            );
        }
    }
}



use bevy::{reflect::TypePath, render::render_resource::AsBindGroup};

#[derive(AsBindGroup, TypePath, Debug, Clone, Default, Asset)]
pub struct MyMaterial {
    #[uniform(0)]
    brightness: f32,
    #[uniform(0)]
    _padding: Vec3,
}

impl MaterialTilemap for MyMaterial {
    fn fragment_shader() -> bevy::render::render_resource::ShaderRef {
        "custom_shader.wgsl".into()
    }
}
