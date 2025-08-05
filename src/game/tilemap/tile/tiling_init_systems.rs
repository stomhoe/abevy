use bevy::ecs::entity_disabling::Disabled;
#[allow(unused_imports)] use bevy::prelude::*;
use bevy_ecs_tilemap::tiles::TileColor;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
#[allow(unused_imports)] use bevy_asset_loader::prelude::*;
use bevy_replicon::shared::server_entity_map::ServerEntityMap;
use bevy_replicon_renet::renet::RenetServer;
use crate::{common::common_components::{DisplayName, EntityPrefix, MyZ, StrId}, game::{game_components::{ImageHolder, ImagePathHolder}, tilemap::tile::{
    tile_components::*,
    tile_resources::*,
}}};


#[allow(unused_parens)]
pub fn init_shaders(
    mut cmd: Commands, asset_server: Res<AssetServer>, 
    repeat_tex_handles: Res<ShaderRepeatTexSerisHandles>,
    mut assets: ResMut<Assets<ShaderRepeatTexSeri>>,
) -> Result {
    let mut result: Result = Ok(());
    for handle in repeat_tex_handles.handles.iter() {
        info!(target: "tiling_loading", "Loading TileSeri from handle: {:?}", handle);
        if let Some(mut seri) = assets.remove(handle) {

            let str_id = StrId::new_take(&mut seri.id)?;

            match ImagePathHolder::new(seri.img_path) {
                Ok(img_path_holder) => {
                    cmd.spawn((
                        str_id,
                        TileShader::TexRepeat(RepeatingTexture::new(
                            &asset_server, img_path_holder, seri.scale, seri.mask_color.into()
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
    result
}

#[allow(unused_parens)]
pub fn add_shaders_to_map(
    mut terrgen_map: Option<ResMut<TileShaderEntityMap>>,
    query: Query<(Entity, &EntityPrefix, &StrId), (Added<StrId>, With<TileShader>)>,
) -> Result {
    let mut result: Result = Ok(());
    if let Some(mut terrgen_map) = terrgen_map {
        for (ent, prefix, str_id) in query.iter() {
            if let Err(err) = terrgen_map.0.insert(str_id, ent, prefix) {
                error!(target: "tiling_loading", "{}", err);
                result = Err(err);
            }
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
        if let Some(mut seri) = assets.remove(handle) {

            let str_id = StrId::new_take(&mut seri.id)?;
            let my_z = MyZ::new(seri.z);
            let enti = cmd.spawn((
                str_id.clone(),
                Tile,
                Disabled,
                my_z.clone(),
            )).id();

            let [r, g, b, a] = seri.color.unwrap_or([255, 255, 255, 255]);
            let color = Color::srgba_u8(r, g, b, a);
            let img_holder = ImageHolder::new(&asset_server, seri.img_path)?;
            if ! seri.sprite {
                cmd.entity(enti).insert((
                    TileColor::from(color),
                    img_holder,
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
                cmd.entity(enti).insert((
                    Sprite{
                        image: img_holder.0.clone(),
                        color,
                        ..Default::default()
                    },
                    Transform::from_translation(Vec2::from_array(seri.offset).extend(my_z.div_1e9())),
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
    query: Query<(Entity, &EntityPrefix, &StrId), (Added<StrId>, With<Tile>, With<Disabled>)>,
) -> Result {
    let mut result: Result = Ok(());
    if let Some(mut terrgen_map) = terrgen_map {
        for (ent, prefix, str_id) in query.iter() {
                if let Err(err) = terrgen_map.0.insert(str_id, ent, prefix) {
                    error!(target: "tiling_loading", "{}", err);
                    result = Err(err);
                } else {
                    info!(target: "tiling_loading", "Inserted tile '{}' into TilingEntityMap with entity {:?}", str_id, ent);
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
            info!(target: "tiling_loading", "Loading TileWeightedSamplerSeri from handle: {:?}", handle);

            let str_id = StrId::new_take(&mut seri.id)?;

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
            cmd.spawn((
                str_id,
                TileWeightedSampler::new(&weights),
            ));
        }
    }
    result
} 

pub fn add_tile_weighted_samplers_to_map(
    terrgen_map: Option<ResMut<TilingEntityMap>>,
    query: Query<(Entity, &EntityPrefix, &StrId), (Added<StrId>, With<TileWeightedSampler>)>,
) -> Result {
    let mut result: Result = Ok(());
    if let Some(mut terrgen_map) = terrgen_map {
        for (ent, prefix, str_id) in query.iter() {
            if let Err(err) = terrgen_map.0.insert(str_id, ent, prefix) {
                error!(target: "tiling_loading", "{}", err);
                result = Err(err);
            } else {
                info!(target: "tiling_loading", "Inserted tile weighted sampler '{}' into TilingEntityMap with entity {:?}", str_id, ent);
            }
        }
    }
    result
}


pub fn client_map_server_tiling(
    trigger: Trigger<TilingEntityMap>,
    server: Option<Res<RenetServer>>,
    mut entis_map: ResMut<ServerEntityMap>,
    own_map: Res<TilingEntityMap>,
) {
    if server.is_some() { return; }

    let TilingEntityMap(received_map) = trigger.event().clone();
    for (hash_id, &server_entity) in received_map.0.iter() {
        if let Ok(client_entity) = own_map.0.get_with_hash(hash_id) {
            // Map the server entity to the local entity
            info!(target: "tiling_loading", "Mapping server entity {:?} to local entity {:?}", server_entity, client_entity);
            entis_map.insert(server_entity, client_entity);
        } else {
            error!(target: "tiling_loading", "Received entity {:?} with hash id {:?} not found in own map", server_entity, hash_id);
        }
    }
}
// pub fn server_map_client_tiles(
//     trigger: Trigger<FromClient<TilingEntityMap>>,
//     mut entis_map: ResMut<ServerEntityMap>,
//     own_map: Res<TilingEntityMap>,
// ) {

//     let TilingEntityMap(received_map) = trigger.event.clone();
//     for (hash_id, &client_entity) in received_map.0.iter() {
//         if let Ok(server_entity) = own_map.0.get_with_hash(hash_id) {
//             info!(target: "tiling_loading", "Mapping client entity {:?} to server entity {:?}", client_entity, server_entity);
//             entis_map.insert(client_entity, server_entity);
//         } else {
//             error!(target: "tiling_loading", "Received entity {:?} with hash id {:?} not found in own map", client_entity, hash_id);
//         }
//     }
// }