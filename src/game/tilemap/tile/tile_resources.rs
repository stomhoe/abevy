use bevy::{ecs::entity_disabling::Disabled, math::U16Vec2, platform::collections::HashMap};
#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
#[allow(unused_imports)] use bevy_asset_loader::prelude::*;
use bevy_ecs_tilemap::{map::TilemapTileSize, tiles::*};

use crate::{common::common_components::{DisplayName, MyZ}, game::tilemap::tile::{
    tile_components::*, 
    tile_constants::*,
}};


#[derive(Resource, Debug, Default )]
pub struct TileShaderEntityMap(pub HashMap<String, Entity>);
impl TileShaderEntityMap {
    pub fn new_repeat_tex_shader(
        &mut self, cmd: &mut Commands, asset_server: &AssetServer,
        handle: Handle<ShaderRepeatTexSeri>, assets: &mut Assets<ShaderRepeatTexSeri>,
    ) {
        if let Some(mut seri) = assets.remove(&handle) {
            use std::mem::take;
            if self.0.contains_key(&seri.id) {
                error!(target: "tiling_loading", "Shader with id {:?} already exists in shader map, skipping", seri.id);
                return;
            }
            let img_path = format!("assets/{}", seri.img_path);
            if !std::path::Path::new(&img_path).exists() {
                error!(target: "tiling_loading", "Image path does not exist: {}", img_path);
                return
            }
            let img_path = take(&mut seri.img_path);
            if seri.id.len() <= 2 {
                error!(target: "tiling_loading", "Shader id '{}' is too short or empty, skipping", seri.id);
                return;
            }
            if seri.scale == 0 {
                error!(target: "tiling_loading", "Shader id '{}' scale is zero, skipping", seri.id);
                return;
            }

            let enti = cmd.spawn((
                Name::new(format!("TileShaderRepeatTex {}", seri.id.clone())),
                TileShader::TexRepeat(RepeatingTexture::new(
                    &asset_server, img_path, seri.scale, seri.mask_color.into()
                )),
            )).id();

            self.0.insert(seri.id, enti);
        }
    }

    //HACER métodos new para otros tipos de shaders con otros parámetros (y sus respectivos structs de _Seri)

    pub fn get_entity<S: Into<String>>(&self, id: S) -> Option<Entity> { self.0.get(&id.into()).copied() }

    #[allow(dead_code)]
    pub fn get_entities<I, S>(&self, ids: I) -> Vec<Entity> where I: IntoIterator<Item = S>, S: AsRef<str>, {
        ids.into_iter().filter_map(|id| self.0.get(id.as_ref()).copied()).collect()
    }
}

#[derive(Resource, Debug, Default )]
pub struct TilingEntityMap(pub HashMap<String, Entity>);

#[allow(unused_parens)]
impl TilingEntityMap {
    pub fn new_tile_ent_from_seri(
        &mut self, cmd: &mut Commands, asset_server: &AssetServer, 
        handle: Handle<TileSeri>, assets: &mut Assets<TileSeri>, shader_map: &TileShaderEntityMap
    ) {
        if let Some(mut seri) = assets.remove(&handle) {
            use std::mem::take;
            if self.0.contains_key(&seri.id) {
                error!(target: "tiling_loading", "TileSeri with id {:?} already exists in map, skipping", seri.id);
                return;
            }
            let img_path = format!("assets/{}", seri.img_path);
            if !std::path::Path::new(&img_path).exists() {
                error!(target: "tiling_loading", "Image path does not exist: {}", img_path);
                return
            }
            let img_path = take(&mut seri.img_path);
            if seri.id.len() <= 2 {
                error!(target: "tiling_loading", "TileSeri id is too short or empty, skipping");
                return;
            }

            let my_z = MyZ::new(seri.z);
            let enti = cmd.spawn((
                Tile,
                Disabled,
                Name::new(format!("Tile id:{}", seri.id.clone())),
                my_z.clone(),
            )).id();

            let [r, g, b, a] = seri.color.unwrap_or([255, 255, 255, 255]);
            let color = Color::srgba_u8(r, g, b, a);
            if ! seri.sprite {
                cmd.entity(enti).insert(TileColor::from(color));
                if seri.shader.len() > 2 {
                    match shader_map.get_entity(&seri.shader) {
                        Some(shader_ent) => {
                            cmd.entity(enti).insert(ShaderRef(shader_ent));
                        }
                        None => {
                            warn!(target: "tiling_loading", "TileSeri id '{}' references missing shader '{}'", seri.id, seri.shader);
                        }
                    }
                } else if seri.shader.len() > 0 {
                    warn!(target: "tiling_loading", "TileSeri id:'{}' shader id:'{}' is too short for a shader", seri.id, seri.shader);
                }
            }
            else{
                cmd.entity(enti).insert((
                    Sprite{
                        image: asset_server.load(img_path),
                        color,
                        ..Default::default()
                    },
                    Transform::from_translation(Vec2::from_array(seri.offset).extend(my_z.div_1e9())),
                ));
                if ! seri.shader.is_empty() {
                    warn!(target: "tiling_loading", "TileSeri id:'{}' tilemap shaders ('{}') are not compatible with sprite=true, ignoring", seri.id, seri.shader);
                }
            }

            if ! seri.name.is_empty() {
                seri.name = seri.id.clone();
                cmd.entity(enti).insert(DisplayName(seri.name.clone()));
            }

            self.0.insert(seri.id.clone(), enti);
        }
    }

    pub fn new_weighted_tilesampler_ent_from_seri(
        &mut self, cmd: &mut Commands, handle: Handle<TileWeightedSamplerSeri>, assets: &mut Assets<TileWeightedSamplerSeri>,
    ) {
        if let Some(mut seri) = assets.remove(&handle) {
            if self.0.contains_key(&seri.id) {
                error!(target: "tiling_loading", "TileWeightedSampler with id {:?} already exists in map, skipping", seri.id);
                return;
            }
            if seri.id.len() <= 2 {
                error!(target: "tiling_loading", "TileWeightedSampler id is too short or empty, skipping");
                return;
            }

            let mut weights: Vec<(Entity, f32)> = Vec::new();

            for (tile_id, weight) in seri.weights.drain() {
                if weight <= 0.0 {
                    error!(target: "tiling_loading", "TileWeightedSampler id {:?} has non-positive weight {}, skipping", tile_id, weight);
                    continue;
                }

                if let Some(ent) = self.0.get(&tile_id) {
                    weights.push((ent.clone(), weight));
                } else {
                    error!(target: "tiling_loading", "TileWeightedSampler id {:?} references non-existent tile id, skipping", tile_id);
                    continue;
                }
            }

            if weights.is_empty() {
                error!(target: "tiling_loading", "TileWeightedSampler id {:?} has no valid tiles, skipping", seri.id);
                return;
            }
            let sampler = cmd.spawn((
                Name::new(format!("WeightedTileSampler {}", seri.id.clone())),
                TileWeightedSampler::new(&weights),
            )).id();

            self.0.insert(seri.id, sampler);
        }
    }

    pub fn get_entity<S: Into<String>>(&self, id: S) -> Option<Entity> { self.0.get(&id.into()).copied() }

    pub fn get_entities<I, S>(&self, ids: I) -> Vec<Entity> where I: IntoIterator<Item = S>, S: AsRef<str>, {
        ids.into_iter().filter_map(|id| self.0.get(id.as_ref()).copied()).collect()
    }
}

#[derive(AssetCollection, Resource)] pub struct TileSerisHandles {
#[asset(path = "ron/tilemap/tiling/tile", collection(typed))] pub handles: Vec<Handle<TileSeri>>,
}
#[derive(AssetCollection, Resource)] pub struct TileWeightedSamplerSerisHandles {
#[asset(path = "ron/tilemap/tiling/weighted_sampler", collection(typed))] pub handles: Vec<Handle<TileWeightedSamplerSeri>>,
}
#[derive(AssetCollection, Resource)] pub struct ShaderRepeatTexSerisHandles {
#[asset(path = "ron/tilemap/tiling/shader", collection(typed))] pub handles: Vec<Handle<ShaderRepeatTexSeri>>,
}


#[derive(serde::Deserialize, Asset, TypePath, Default)]
pub struct ShaderRepeatTexSeri {
    pub id: String,
    pub img_path: String,
    pub scale: u32,
    pub mask_color: [u8; 4],
}

#[derive(serde::Deserialize, Asset, TypePath, Default)]
pub struct TileSeri {
    pub id: String,
    pub name: String,
    pub img_path: String,
    pub shader: String,
    pub sprite: bool,
    pub offset: [f32; 2],
    pub z: i32,
    pub color: Option<[u8; 4]>,
    pub color_map: String,
    pub spawns: Vec<String>,
    pub spawns_children: Vec<String>,
    pub somecomp_present: Option<bool>,
}

#[derive(serde::Deserialize, Asset, TypePath, Default)]
pub struct TileWeightedSamplerSeri {
    pub id: String,
    pub weights: HashMap<String, f32>,
}