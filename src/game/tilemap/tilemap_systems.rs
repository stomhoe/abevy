
use bevy::{math::U16Vec2, platform::collections::{HashMap}, prelude::*};
use bevy_ecs_tilemap::{map::*, prelude::MaterialTilemapHandle, tiles::*, MaterialTilemapBundle, TilemapBundle};
use debug_unwraps::DebugUnwrapExt;

use crate::game::tilemap::{terrain_gen::{terrain_gen_components::{TileInstantiationData, UsedShader}, terrain_gen_utils::{ UniqueTileDto, Z_DIVISOR}, terrain_materials::TextureOverlayMaterial}, tile_imgs::{ImgIngameCfg, NidImgMap, NidRepeatImgMap, TileImgNid}, tilemap_components::*, tilemap_resources::*};


pub fn tmaptsize_to_uvec2(tile_size: TilemapTileSize) -> UVec2 {
    UVec2::new(tile_size.x as u32, tile_size.y as u32)
}
pub fn uvec2_to_tmaptsize(tile_size: U16Vec2) -> TilemapTileSize {
    TilemapTileSize { x: tile_size.x as f32, y: tile_size.y as f32 }
}

type Map = HashMap<(i32, U16Vec2, UsedShader), LayerDto>;

#[allow(unused_parens)]
pub fn produce_tilemaps(
    mut commands: Commands, 
    chunk_query: Query<(Entity, &TilesReady),>,
    tile_ins_data_query: Query<&TileInstantiationData>,
    nid_img_map: Res<NidImgMap>, 
) {
    let mut layers: Map = HashMap::new();

    for (chunk_ent, tiles_ready) in chunk_query.iter() {
        for uniq_tile_dto in tiles_ready.0.iter() {
            if let Ok(tile_ins_data) = tile_ins_data_query.get(uniq_tile_dto.tile_inst_data_entity()) {unsafe {
                let img_cfg = nid_img_map.get(tile_ins_data.image_nid).debug_expect_unchecked("Image NID not found in nid_img_map");

                if let Some(layer_dto) = layers.get_mut(&(img_cfg.get_z_index(), img_cfg.size, tile_ins_data.used_shader)) {
                    let (tilemap_entity, tile_storage, image_nids) = (
                        layer_dto.tilemap_entity,
                        &mut layer_dto.tile_storage,
                        &mut layer_dto.image_nids,
                    );

                    let texture_index = match image_nids.iter().position(|x| *x == tile_ins_data.image_nid) {
                        Some(index) => TileTextureIndex(index as u32),
                        None => {
                            image_nids.push(tile_ins_data.image_nid);
                            TileTextureIndex((image_nids.len() - 1) as u32)
                        }
                    };

                    add_tile_bundle_and_put_in_storage(&mut commands, &uniq_tile_dto, &tile_ins_data, tilemap_entity, tile_storage, texture_index);

                    layer_dto.needs_y_sort = layer_dto.needs_y_sort || img_cfg.needs_y_sort;
                } else {
                    instantiate_new_layer_dto(&mut commands, &mut layers, img_cfg, &uniq_tile_dto, tile_ins_data, chunk_ent);
                }
            }}
        }

        for (_, layer_dto) in layers.drain() {
            commands.entity(layer_dto.tilemap_entity).insert(layer_dto);
        }
        commands.entity(chunk_ent).insert(LayersReady);
    }

}

#[derive(Component, Debug, )]
pub struct LayerDto {
    pub layer_z: i32, 
    pub tile_size: TilemapTileSize,
    pub used_shader: UsedShader,
    pub tilemap_entity: Entity, 
    pub tile_storage: TileStorage,
    pub image_nids: Vec<TileImgNid>, pub needs_y_sort: bool,
}

fn add_tile_bundle_and_put_in_storage( 
    commands: &mut Commands, 
    unique_tile_dto: &UniqueTileDto,
    tile_inst_data: &TileInstantiationData, 
    tilemap_entity: Entity,  tile_storage: &mut TileStorage,
    texture_index: TileTextureIndex,
)  {
    let tile_bundle = TileBundle {
        position: unique_tile_dto.pos_within_chunk(), tilemap_id: TilemapId(tilemap_entity),
        texture_index,  flip: tile_inst_data.flip, color: tile_inst_data.tile_color(),
        visible: tile_inst_data.tile_visible(), ..Default::default()
    };


    commands.entity(unique_tile_dto.tile_entity()).insert(tile_bundle);
    tile_storage.set( &unique_tile_dto.pos_within_chunk(), unique_tile_dto.tile_entity());
}

fn instantiate_new_layer_dto(
    commands: &mut Commands, 
    layers: &mut Map,
    img_cfg: &ImgIngameCfg,
    unique_tile_dto: &UniqueTileDto,
    tile_inst_data: &TileInstantiationData,
    chunk_ent: Entity
) {
    let tilemap_entity: Entity = commands.spawn(ChildOf(chunk_ent)).id();

    let mut tile_storage = TileStorage::empty(CHUNK_SIZE.as_uvec2().into());

    add_tile_bundle_and_put_in_storage(
        commands, unique_tile_dto, &tile_inst_data, tilemap_entity, &mut tile_storage,
        TileTextureIndex(0),
    );

    let layer_dto = LayerDto {
        tile_size: img_cfg.get_tile_size(),
        layer_z: img_cfg.get_z_index(),
        used_shader: tile_inst_data.used_shader,
        tilemap_entity,
        tile_storage,
        image_nids: vec![tile_inst_data.image_nid],
        needs_y_sort: img_cfg.needs_y_sort,
    };
    layers.insert((img_cfg.get_z_index(), img_cfg.size, tile_inst_data.used_shader), layer_dto);
}


#[allow(unused_parens)]
pub fn fill_tilemaps_data(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut chunk_query: Query<(Entity, &Children), With<LayersReady>>,
    mut layer_query: Query<(&mut LayerDto, &ChildOf)>,
    tile_images_map: Res<NidImgMap>,
    repeat_images_map: Res<NidRepeatImgMap>,
    mut texture_overley_mat: ResMut<Assets<TextureOverlayMaterial>>,
) {unsafe{

    for (mut layer_dto, childof) in layer_query.iter_mut() {
        commands.entity(layer_dto.tilemap_entity).remove::<LayerDto>();

        let grid_size = TilemapGridSize { x: TILE_SIZE_PXS.x as f32, y: TILE_SIZE_PXS.y as f32 };
        let size = CHUNK_SIZE.as_uvec2().into();
        let transform = Transform::from_translation(Vec3::new(0.0, 0.0, (layer_dto.layer_z as f32 / Z_DIVISOR as f32)));
        // let images: Vec<Handle<Image>> = layer_dto
        //     .image_nids
        //     .iter()
        //     .map(|&image_nid| tile_images_map.get(image_nid).debug_expect_unchecked("Image NID not found in tile_images_map").handle.clone())
        //     .collect();
        
        //let texture = TilemapTexture::Vector(images);
        let texture = TilemapTexture::Single(asset_server.load("textures/world/terrain/temperate_grass/grass.png"));
        let storage = std::mem::take(&mut layer_dto.tile_storage);
        let render_settings = TilemapRenderSettings {
            render_chunk_size: UVec2 { x: (CHUNK_SIZE.x * 2) as u32, y: (CHUNK_SIZE.y * 2) as u32 },
            y_sort: layer_dto.needs_y_sort,
        };
        let tile_size = layer_dto.tile_size;

        match layer_dto.used_shader {
            UsedShader::None => {
                commands.entity(layer_dto.tilemap_entity).insert(
                    TilemapBundle {
                        grid_size,
                        size,
                        storage,
                        texture,
                        tile_size,
                        transform,
                        render_settings,
                        ..Default::default()
                    }
                );
            }
            UsedShader::Grass => {
                let material = MaterialTilemapHandle::from(texture_overley_mat.add(TextureOverlayMaterial {
                    texture_overlay: asset_server.load("textures/world/terrain/temperate_grass/grass.png"),
                    scale: 0.001,
                    ..Default::default()
                }));

                commands.entity(layer_dto.tilemap_entity).insert(
                    MaterialTilemapBundle {
                        grid_size,
                        size,
                        storage,
                        texture,
                        tile_size,
                        transform,
                        render_settings,
                        material,
                        ..Default::default()
                    }
                );
            }
        }
        commands.entity(childof.0).insert(InitializedChunk);
    }
       //commands.entity(chunk).insert(InitializedChunk);
}}


