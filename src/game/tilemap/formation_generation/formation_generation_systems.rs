use std::mem;

use bevy::{ecs::query, log::tracing_subscriber::layer, platform::collections::{HashMap, HashSet}, prelude::*};
use bevy_ecs_tilemap::{map::*, prelude::{MaterialTilemap, MaterialTilemapHandle}, tiles::*, MaterialTilemapBundle, TilemapBundle};

use crate::game::tilemap::{formation_generation::{formation_generation_components::*, formation_generation_resources::*}, tilemap_components::{Chunk, TilesInstantiated}, tilemap_resources::{*}};


// NO OLVIDARSE DE AGREGARLO AL Plugin DEL MÓDULO
#[allow(unused_parens)]
pub fn setup(mut commands: Commands, query: Query<(),()>, world_settings: Res<WorldGenSettings>) {

}

const RENDER_CHUNK_SIZE: UVec2 = UVec2 {x: CHUNK_SIZE.x * 2, y: CHUNK_SIZE.y * 2,};


#[allow(unused_parens)]
pub fn add_tilemaps_to_chunk(
    mut commands: Commands, 
    textures: Res<Textures>,
    gen_settings: Res<WorldGenSettings>, 
    chunks_query: Query<(Entity, &Chunk), (Without<TilesInstantiated>)>, 
    noise_query: Query<&NoiseComp>, 
) {
    for (chunk_ent, chunk) in chunks_query.iter() {
       
        produce_tilemaps(&mut commands, &gen_settings, chunk_ent, chunk.0, noise_query);
        
        commands.entity(chunk_ent).insert(TilesInstantiated);
    }
}

struct TileInfo{
    pub layer_z: i16,
    pub pos_within_chunk: UVec2,
    pub shader_id: u16,//dejar pa después. darle su propio tilemap si usa shader, o cambiarle a propósito levemente layer_z para no sobrecomplicarlo y así tiene de forma simple su layer
    pub used_handle: Handle<Image>,//NO SÉ SI USAR ESTO O UNA ID O ALGO ASÍ EN VEZ DE ESTE SHARED POINTER
    pub flip: TileFlip,
    pub color: TileColor,
    pub needs_y_sort: bool,
}

// NO OLVIDARSE DE AGREGARLO AL Plugin DEL MÓDULO
#[allow(unused_parens)]

fn gather_all_tiles2spawn_within_chunk (mut commands: &mut Commands, noise_query: Query<&NoiseComp>, chunk_pos: IVec2,) -> Vec<TileInfo> {
    
    let mut tiles2spawn: Vec<TileInfo> = Vec::new();

    for x in 0..CHUNK_SIZE.x {
        for y in 0..CHUNK_SIZE.y {
            let pos_within_chunk = UVec2::new(x, y);
            
            let x = x as i32 + chunk_pos.x; let y = y as i32;
            let worldgrid_pos = chunk_pos + IVec2::new(x,y);

            //el super match o lo q sea
            
            let tileinfo = TileInfo {
                layer_z: 0,
                pos_within_chunk,
                shader_id: 0,
                used_handle: Handle::<Image>::default(),
                flip: TileFlip::default(),
                color: TileColor::default(),
                needs_y_sort: false,
            };
            
            tiles2spawn.push(tileinfo);
        }
    }
    tiles2spawn
}


struct LayerInfo {
    pub tilemap_entity: Entity,
    pub tile_storage: TileStorage,
    pub handles: Vec<Handle<Image>>,
    pub needs_y_sort: bool,
}

#[allow(unused_parens)]
fn produce_tilemaps(
    commands: &mut Commands, 
    gen_settings: &WorldGenSettings,
    chunk_ent: Entity, 
    chunk_pos: IVec2, 
    noise_query: Query<&NoiseComp>,
) {
    let mut layers: HashMap<i16, LayerInfo> = HashMap::new();

    // Collect all tiles to spawn, taking ownership of the TileInfo values
    for tileinfo in gather_all_tiles2spawn_within_chunk(commands, noise_query, chunk_pos) {
        if let Some(layer_info) = layers.get_mut(&tileinfo.layer_z) {
            let (tilemap_entity, tile_storage, handles) = (
                &mut layer_info.tilemap_entity,
                &mut layer_info.tile_storage,
                &mut layer_info.handles,
            );

            let tilepos_within_chunk = TilePos {
                x: tileinfo.pos_within_chunk.x,
                y: tileinfo.pos_within_chunk.y,
            };

            let texture_index = match handles.iter().position(|x| *x == tileinfo.used_handle) {
                Some(index) => TileTextureIndex(index as u32),
                None => {
                    handles.push(tileinfo.used_handle.clone());
                    TileTextureIndex((handles.len() - 1) as u32)
                }
            };

            let tile_entity: Entity = commands
                .spawn((
                    TileBundle {
                        position: tilepos_within_chunk,
                        tilemap_id: TilemapId(*tilemap_entity),
                        texture_index,
                        flip: tileinfo.flip,
                        color: tileinfo.color,
                        ..Default::default()
                    },
                    ChildOf(*tilemap_entity),
                ))
                .id();

            tile_storage.set(&tilepos_within_chunk, tile_entity);

            layer_info.needs_y_sort = layer_info.needs_y_sort || tileinfo.needs_y_sort;
        } else {
            let mut tile_storage = TileStorage::empty(CHUNK_SIZE.into());
            let tilemap_entity: Entity = commands.spawn_empty().id();

            let tilepos_within_chunk = TilePos {
                x: tileinfo.pos_within_chunk.x,
                y: tileinfo.pos_within_chunk.y,
            };

            let tile_entity: Entity = commands
                .spawn((
                    TileBundle {
                        position: tilepos_within_chunk,
                        tilemap_id: TilemapId(tilemap_entity),
                        texture_index: TileTextureIndex(0),
                        flip: tileinfo.flip,
                        color: tileinfo.color,
                        ..Default::default()
                    },
                    ChildOf(tilemap_entity),
                ))
                .id();

            tile_storage.set(&tilepos_within_chunk, tile_entity);

            let handles = vec![tileinfo.used_handle.clone()];

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
            commands,
            layer_info.tilemap_entity,
            TilemapTexture::Vector(mem::take(&mut layer_info.handles)),
            chunk_ent,
            chunk_pos,
            mem::take(&mut layer_info.tile_storage),
            layer_z,
            layer_info.needs_y_sort,
        );
    }
}


// NO OLVIDARSE DE AGREGARLO AL Plugin DEL MÓDULO
#[allow(unused_parens)]
fn make_tilemap(commands: &mut Commands, 
    tmap_entity: Entity,
    texture: TilemapTexture, 
    chunk_ent: Entity, 
    chunk_pos: IVec2,
    storage: TileStorage,
    layer_z_level: i16, y_sort: bool, 
    //mut materials: ResMut<Assets<MyMaterial>>,//buscar formas alternativas de agarrar shaders
)
{
    commands.entity(tmap_entity).insert(ChildOf(chunk_ent));
    let mut ent_commands = commands.entity(tmap_entity);
    let grid_size = BASE_TILE_SIZE.into(); let size = CHUNK_SIZE.into();  
    let tile_size = BASE_TILE_SIZE;
    let transform = Transform::from_translation(chunkpos_to_contpos(chunk_pos).extend(layer_z_level.into()));
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
            ent_commands.insert((
                TilemapBundle {grid_size, size, storage, texture, tile_size, transform, render_settings, ..Default::default()}
            ));
        }
    }
}


use bevy::{reflect::TypePath, render::render_resource::AsBindGroup};

#[derive(AsBindGroup, TypePath, Debug, Clone, Default, Asset)]
pub struct MyMaterial {
    #[uniform(0)]
    brightness: f32,
    // webgl2 requires 16 byte alignment
    #[uniform(0)]
    _padding: Vec3,
}

impl MaterialTilemap for MyMaterial {
    fn fragment_shader() -> bevy::render::render_resource::ShaderRef {
        "custom_shader.wgsl".into()
    }
}




