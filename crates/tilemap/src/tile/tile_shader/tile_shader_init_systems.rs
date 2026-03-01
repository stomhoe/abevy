#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
#[allow(unused_imports)] use bevy_asset_loader::prelude::*;
use common::common_components::*;
use common::log_targets::TILE_SHADER_INIT;
use crate::tile::tile_shader::{TileShaderEntityMap, tile_material::prelude::*, tile_shader_components::*, tile_shader_resources::*};


#[allow(unused_parens)]
pub fn init_shaders(
    mut cmd: Commands,
    tileshader_map: Option<Res<TileShaderEntityMap>>,
) {
    if let Some(tileshader_map) = &tileshader_map
        && !tileshader_map.0.is_empty() {
        return;
    }
    let mut shader_comps_to_insert = Vec::new();
    let mut path_holders_to_insert = Vec::new();

    for seri in load_shader_terrbl_seri_defs() {
            //trace!(target: TILE_SHADER_INIT, "Loading Shader from handle: {:?}", handle);

            let str_id = match StrId::new_with_result(seri.id.clone(), 1) {
                Ok(id) => id,
                Err(err) => {
                    error!(target: TILE_SHADER_INIT, "Failed to create StrId for shader '{}': {}", seri.id, err);
                    continue;
                }
            };

            match ImagePathHolder::new(seri.img_path) {
                Ok(path_holder) => {

                    let ent = cmd.spawn_empty().id();

                    shader_comps_to_insert.push((ent, (
                        str_id.clone(),
                        TileShader::TerrBlend(TerrBlendMat::new(
                            Handle::default(),
                            seri.mask_color.into(),
                            seri.scale,
                            seri.speed,
                            seri.wavy_strength,
                        )),
                    )));
                    path_holders_to_insert.push((ent, path_holder));

                },
                Err(err) => {
                    error!(target: TILE_SHADER_INIT, "Failed to create ImagePathHolder for shader '{}': {}", str_id, err);
                }
            }
    }
    cmd.insert_batch(path_holders_to_insert);
    cmd.insert_batch(shader_comps_to_insert);
}
