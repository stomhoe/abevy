
use bevy::{math::U16Vec2, platform::collections::{HashMap}, prelude::*};
use bevy_ecs_tilemap::{anchor::TilemapAnchor, map::*, prelude::MaterialTilemapHandle, tiles::*, MaterialTilemapBundle, TilemapBundle};
use debug_unwraps::DebugUnwrapExt;

use crate::{common::common_components::MyZ, game::tilemap::{chunking_components::*, chunking_resources::*, terrain_gen::terrain_materials::MonoRepeatTextureOverlayMat, tile::{tile_components::{AppliedShader, Tileimg}, tile_constants::TILE_SIZE_PXS, tile_resources::{HandleConfigMap, TileimgConfig}},}};


pub fn tmaptsize_to_uvec2(tile_size: TilemapTileSize) -> UVec2 {
    UVec2::new(tile_size.x as u32, tile_size.y as u32)
}
pub fn uvec2_to_tmaptsize(tile_size: U16Vec2) -> TilemapTileSize {
    TilemapTileSize { x: tile_size.x as f32, y: tile_size.y as f32 }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct MapKey {z_index: MyZ, tile_size: U16Vec2, shader: Option<AppliedShader>,}
impl MapKey {
    pub fn new(z_index: MyZ, size: U16Vec2, shader: Option<AppliedShader>) -> Self {
        Self { z_index, tile_size: size, shader }
    }
    pub fn take_shader(&mut self) -> Option<AppliedShader> { std::mem::take(&mut self.shader) }
}

type Map = HashMap<MapKey, LayerDto>;

#[allow(unused_parens)]
pub fn produce_tilemaps(
    mut cmd: Commands, 
    mut chunk_query: Query<(Entity, &mut ProducedTiles), (Without<Children>, Added<TilesReady>)>,
    mut tile_comps: Query<(Entity, &TilePos, Option<&Tileimg>, Option<&MyZ>, Option<&AppliedShader>, Option<&mut Transform>)>,
    handle_config_map: Res<HandleConfigMap>, 
) -> Result {
    let mut layers: Map = HashMap::new();

    #[allow(unused_mut)]
    for (chunk_ent, mut produced_tiles) in chunk_query.iter_mut() {
        for &tile_ent in produced_tiles.produced_tiles().iter() {unsafe{

            let (tile_ent, tile_pos, tile_img, tile_z_index, mut shader, transf) = tile_comps.get_mut(tile_ent)?;

            let tile_z_index = tile_z_index.cloned().unwrap_or_default();

            if let Some(mut transform) = transf {
                transform.translation.x += (tile_pos.x as f32 * TILE_SIZE_PXS.x as f32); 
                transform.translation.y += (tile_pos.y as f32 * TILE_SIZE_PXS.y as f32);
            }

            let shader: Option<AppliedShader> = shader.take().cloned();
            cmd.entity(tile_ent).remove::<Tileimg>().remove::<AppliedShader>();
                

            let tile_size = if let Some(img) = tile_img {
                handle_config_map.get(img).debug_expect_unchecked("img not found in handle_config_map").tile_size_u16vec2()
            } else {
                U16Vec2::ONE
            };

            let map_key = MapKey::new(tile_z_index, tile_size, shader);

            if let Some(layer_dto) = layers.get_mut(&map_key) {
                let (tilemap_entity, tile_storage, tile_imgs) = (
                    layer_dto.tilemap,
                    &mut layer_dto.tile_storage,
                    &mut layer_dto.images,
                );

                let texture_index: Option<TileTextureIndex> = if let Some(Tileimg(handle)) = tile_img {
                    match tile_imgs.iter().position(|x| *x == *handle) {
                        Some(i) => Some(TileTextureIndex(i as u32)),
                        None => {
                            tile_imgs.push(handle.clone());
                            Some(TileTextureIndex((tile_imgs.len() - 1) as u32))
                        }
                    }
                } else { None };
                add_tile_bundle_and_put_in_storage(&mut cmd, tile_ent, tile_pos, tilemap_entity, tile_storage, texture_index);

            } else {
                instantiate_new_layer_dto(&mut cmd, &mut layers, map_key, tile_size, tile_ent, tile_pos, tile_img.clone(), chunk_ent);
            }
        }}
        if layers.is_empty() {
            warn!("No tiles produced for chunk {:?}", chunk_ent);
            cmd.entity(chunk_ent).insert(InitializedChunk);

        } else{
            cmd.entity(chunk_ent).insert(LayersReady);
            for (mut key, mut layer_dto) in layers.drain() {
                layer_dto.used_shader = key.take_shader();
                cmd.entity(layer_dto.tilemap.entity()).insert(layer_dto);
            }
        }
    }
    Ok(())
}

#[derive(Component, Debug, )]
pub struct LayerDto {
    pub layer_z: MyZ, 
    pub tile_size: TilemapTileSize,
    pub used_shader: Option<AppliedShader>,
    pub tilemap: TilemapId, 
    pub tile_storage: TileStorage,
    pub images: Vec<Handle<Image>>, 
}
impl LayerDto { pub fn take_images(&mut self) -> Vec<Handle<Image>> { std::mem::take(&mut self.images) } }

fn add_tile_bundle_and_put_in_storage( 
    commands: &mut Commands, 
    tile_entity: Entity,
    tile_pos: &TilePos,
    tilemap: TilemapId,  
    tile_storage: &mut TileStorage,
    texture_index: Option<TileTextureIndex>,
)  {
    let tile_bundle = TileBundle {
        tilemap_id: tilemap, 
        texture_index: texture_index.unwrap_or_default(),
        ..Default::default()
    };

    commands.entity(tile_entity).insert_if_new((ChildOf(tilemap.entity()), tile_bundle));
    if texture_index.is_none() {
        commands.entity(tile_entity).insert_if_new((
            tile_bundle,
        )).insert((TileVisible(false), Visibility::Inherited));
    }
    tile_storage.set(tile_pos, tile_entity);

}

fn instantiate_new_layer_dto( commands: &mut Commands, 
    layers: &mut Map, map_key: MapKey,
    tile_size: U16Vec2, tile_ent: Entity, tile_pos: &TilePos,
    tileimg: Option<&Tileimg>, chunk_ent: Entity,
) {
    let tilemap_entity = commands.spawn((Name::new("Tilemap"), ChildOf(chunk_ent))).id();
    let tilemap = TilemapId(tilemap_entity);

    let mut tile_storage = TileStorage::empty(CHUNK_SIZE.as_uvec2().into());

    let texture_index = tileimg.map(|_| TileTextureIndex(0));

    add_tile_bundle_and_put_in_storage(
        commands,
        tile_ent,
        tile_pos,
        tilemap,
        &mut tile_storage,
        texture_index,
    );
    let images = tileimg
        .map(|img| vec![img.0.clone()])
        .unwrap_or_else(|| vec![Handle::<Image>::default()]);

    let layer_dto = LayerDto {
        tile_size: uvec2_to_tmaptsize(tile_size),
        layer_z: map_key.z_index,
        tilemap,
        tile_storage,
        images,
        used_shader: None,
    };

    layers.insert(map_key, layer_dto);
}


#[allow(unused_parens)]
pub fn fill_tilemaps_data(
    mut commands: Commands,
    mut chunk_query: Query<(Entity, &Children), (With<LayersReady>)>,
    mut layer_query: Query<(&mut LayerDto), (With<ChildOf>)>,
    mut texture_overley_mat: ResMut<Assets<MonoRepeatTextureOverlayMat>>,
) {unsafe{
    
     for (chunk, children) in chunk_query.iter_mut() {
        commands.entity(chunk).remove::<LayersReady>();


        for child in children.iter() {
            if let Ok(mut layer_dto) = layer_query.get_mut(child) {//DEJAR CON IF LET
                let grid_size = TilemapGridSize { x: TILE_SIZE_PXS.x as f32, y: TILE_SIZE_PXS.y as f32 };//NO TOCAR
                let size = CHUNK_SIZE.as_uvec2().into();
                let transform = Transform::from_translation(Vec3::new(0.0, 0.0, (layer_dto.layer_z.div_1e9())));
                let texture = TilemapTexture::Vector(layer_dto.take_images());
                let storage = std::mem::take(&mut layer_dto.tile_storage);
                let render_settings = TilemapRenderSettings {render_chunk_size: UVec2 {x: (CHUNK_SIZE.x * 2) as u32, y: (CHUNK_SIZE.y * 2) as u32,}, y_sort: false};
                let tile_size = layer_dto.tile_size.clone();
                let mut tmap_commands = commands.entity(layer_dto.tilemap.entity());


                if let Some(shader) = &layer_dto.used_shader {
                    match shader{
                        AppliedShader::MonoRepeating(rep_texture) => {
                            let material = MaterialTilemapHandle::from(texture_overley_mat.add(
                                MonoRepeatTextureOverlayMat {
                                    texture_overlay: rep_texture.cloned_handle(),
                                    scale: rep_texture.scale_div_1M(),
                                    mask_color: rep_texture.mask_color(),
                                }
                            ));

                            tmap_commands.insert((
                                MaterialTilemapBundle {grid_size, size, storage, texture, tile_size, transform, render_settings, 
                                    material, ..Default::default()
                                },
                            ));
                        },
                        AppliedShader::BiRepeating(first_texture, second) => todo!(),
                    }
                } else {
                    tmap_commands.insert(
                        TilemapBundle {grid_size, size, storage, texture, tile_size, transform, render_settings, ..Default::default()}
                    );
                }
                    tmap_commands.remove::<LayerDto>();
            }
        }

        commands.entity(chunk).insert(InitializedChunk);
        
    }
}}


