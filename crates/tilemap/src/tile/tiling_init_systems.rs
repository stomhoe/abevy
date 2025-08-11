use bevy::ecs::entity_disabling::Disabled;
#[allow(unused_imports)] use bevy::prelude::*;
use bevy_ecs_tilemap::tiles::TileColor;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
#[allow(unused_imports)] use bevy_asset_loader::prelude::*;
use common::common_components::{DisplayName, EntityPrefix, ImageHolder, ImageHolderMap, StrId};
use game_common::{game_common_components::MyZ, };

use crate::{tile::{tile_components::*, tile_resources::*}, };


#[allow(unused_parens)]
pub fn init_shaders(
    mut cmd: Commands, asset_server: Res<AssetServer>, 
    repeat_tex_handles: Res<ShaderRepeatTexSerisHandles>,
    mut assets: ResMut<Assets<ShaderRepeatTexSeri>>,
) -> Result {
    let mut result: Result = Ok(());
    for handle in repeat_tex_handles.handles.iter() {
        //info!(target: "tiling_loading", "Loading Shader from handle: {:?}", handle);
        if let Some(seri) = assets.remove(handle) {

            let str_id = StrId::new(seri.id)?;

            match ImageHolder::new(&asset_server, seri.img_path) {
                Ok(img_holder) => {
                    cmd.spawn((
                        str_id,
                        TileShader::TexRepeat(RepeatingTexture::new(
                            img_holder, seri.scale, seri.mask_color.into()
                        )),
                    ));
                },
                Err(err) => {
                    error!(target: "tiling_loading", "Failed to create ImagePathHolder for shader '{}': {}", str_id, err);
                    result = Err(err);
                }
            }
        }
    }
    // FORS PARA OTROS TIPOS DE SHADERS...
    result
}

#[allow(unused_parens)]
pub fn add_shaders_to_map(
    terrgen_map: Option<ResMut<TileShaderEntityMap>>,
    query: Query<(Entity, &EntityPrefix, &StrId), (Added<TileShader>,)>,
) -> Result {
    let Some(mut terrgen_map) = terrgen_map else {
        return Err(BevyError::from("Failed to get TileShaderEntityMap"));
    };
    let mut result: Result = Ok(());
    for (ent, prefix, str_id) in query.iter() {
        if let Err(err) = terrgen_map.0.insert(str_id, ent, ) {
            error!(target: "tiling_loading", "{} {} already in TileShaderEntityMap : {}", prefix, str_id, err);
            result = Err(err);
        }
    }
    result
}

#[allow(unused_parens)]
pub fn init_tiles(
    mut cmd: Commands,  asset_server: Res<AssetServer>,
    seris_handles: Res<TileSerisHandles>, mut assets: ResMut<Assets<TileSeri>>,
    shader_map: Res<TileShaderEntityMap>,
) -> Result {
    let mut result: Result = Ok(());
    for handle in seris_handles.handles.iter() {
        //info!(target: "tiling_loading", "Loading TileSeri from handle: {:?}", handle);
        if let Some(seri) = assets.remove(handle) {

            let str_id = StrId::new(seri.id)?;
            let my_z = MyZ(seri.z);
            let enti = cmd.spawn((
                Tile, str_id.clone(), Disabled,
                my_z.clone(),
            )).id();

            let [r, g, b, a] = seri.color.unwrap_or([255, 255, 255, 255]);
            let color = Color::srgba_u8(r, g, b, a);

           

            if seri.img_paths.is_empty() {
                error!(target: "tiling_loading", "Tile '{}' has no images", str_id);
                continue;
            }

            //TODO HACER Q LAS TILES PUEDAN TENER MUCHAS IMÁGENES (PARA IR CAMBIANDO ENTRE ELLAS SEGÚN EL ESTADO, USANDO EL INDEX)
            if ! seri.sprite {
                let tile_handles = TileIdsHandles::from_paths(&asset_server, seri.img_paths, );

                let tile_handles = match tile_handles {
                    Ok(tile_handles) => tile_handles,
                    Err(err) => {
                        error!(target: "tiling_loading", "Failed to create TileHandles for tile '{}': {}", str_id, err);
                        result = Err(err);
                        continue;
                    }
                };

                cmd.entity(enti).insert((
                    TileColor::from(color),
                    tile_handles,
                ));
                if seri.shader.len() > 2 {
                    match shader_map.0.get(&seri.shader) {
                        Ok(shader_ent) => {
                            cmd.entity(enti).insert(TileShaderRef(shader_ent));
                        }
                        Err(err) => {
                            warn!(target: "tiling_loading", "Tile '{}' references missing shader '{}': {}", str_id, seri.shader, err);
                            result = Err(err);
                        }
                    }
                } else if seri.shader.len() > 0 {
                    warn!(target: "tiling_loading", "Tile {} shader {} is too short for a shader", str_id, seri.shader);
                }
            }
            else{
                let map = match ImageHolderMap::from_paths(&asset_server, seri.img_paths) {
                    Ok(map) => map,
                    Err(err) => {
                        error!(target: "tiling_loading", "Failed to create ImageHolderMap for tile '{}': {}", str_id, err);
                        result = Err(err);
                        continue;
                    }
                };

                cmd.entity(enti).insert((
                    Sprite{
                        image: map.first_handle(),
                        color,
                        ..Default::default()
                    },
                    map,
                    Transform::from_translation(Vec2::from_array(seri.offset).extend(my_z.as_float())),
                ));
                if ! seri.shader.is_empty() {
                    warn!(target: "tiling_loading", "Tile {} tilemap shaders ('{}') are not compatible with sprite=true, ignoring", str_id, seri.shader);
                }
            }

            if ! seri.name.is_empty() {
                cmd.entity(enti).insert(DisplayName(seri.name.clone()));
            }
        }
    }
    result
} 

pub fn add_tiles_to_map(
    terrgen_map: Option<ResMut<TilingEntityMap>>,
    query: Query<(Entity, &EntityPrefix, &StrId), (Added<Tile>, With<Disabled>)>,
) -> Result {
    let mut result: Result = Ok(());
    if let Some(mut terrgen_map) = terrgen_map {
        for (ent, prefix, str_id) in query.iter() {
                if let Err(err) = terrgen_map.0.insert(str_id, ent, ) {
                    error!(target: "tiling_loading", "{} {} already in TilingEntityMap : {}", prefix, str_id, err);
                    result = Err(err);
                } else {
                    //info!(target: "tiling_loading", "Inserted tile '{}' into TilingEntityMap with entity {:?}", str_id, ent);
                }
        }
    }
    result
}

#[allow(unused_parens)]
pub fn init_tile_weighted_samplers(
    mut cmd: Commands, 
    seris_handles: Res<TileWeightedSamplerSerisHandles>,
    mut assets: ResMut<Assets<TileWeightedSamplerSeri>>,
    map: Res<TilingEntityMap>,
) -> Result {
    let mut result: Result = Ok(());
    for handle in seris_handles.handles.iter() {
        if let Some(mut seri) = assets.remove(&*handle) {
            //info!(target: "tiling_loading", "Loading TileWeightedSamplerSeri from handle: {:?}", handle);

            let str_id = StrId::new(seri.id)?;

            let mut weights: Vec<(Entity, f32)> = Vec::new();

            for (tile_id, weight) in seri.weights.drain() {
                if weight <= 0.0 {
                    let err = BevyError::from(format!("TileWeightedSampler {:?} has non-positive weight {}, skipping", str_id, weight));
                    error!(target: "tiling_loading", "{}", err);
                    result = Err(err);
                    continue;
                }
                if let Ok(ent) = map.0.get(&tile_id) {
                    weights.push((ent.clone(), weight));
                } else {
                    let err = BevyError::from(format!("TileWeightedSampler {:?} references non-existent tile id {:?}, skipping", str_id, tile_id));
                    error!(target: "tiling_loading", "{}", err);
                    result = Err(err);
                    continue;
                }
            }
            if weights.is_empty() {
                let err = BevyError::from(format!("TileWeightedSampler {:?} has no valid tiles, skipping", str_id));
                error!(target: "tiling_loading", "{}", err);
                result = Err(err);
                continue;
            }
            cmd.spawn((str_id, HashPosEntiWeightedSampler::new(&weights), ));
        }
    }
    result
} 
#[allow(unused_parens, )]
pub fn add_tile_weighted_samplers_to_map(
    terrgen_map: Option<ResMut<TilingEntityMap>>,
    query: Query<(Entity, &EntityPrefix, &StrId), (Added<HashPosEntiWeightedSampler>)>,
) -> Result {
    let mut result: Result = Ok(());
    if let Some(mut terrgen_map) = terrgen_map {
        for (ent, prefix, str_id) in query.iter() {
            if let Err(err) = terrgen_map.0.insert(str_id, ent, ) {
                error!(target: "tiling_loading", "{} {} already in TilingEntityMap : {}", prefix, str_id, err);
                result = Err(err);
            } else {
                //info!(target: "tiling_loading", "Inserted tile weighted sampler '{}' into TilingEntityMap with entity {:?}", str_id, ent);
            }
        }
    }
    result
}

