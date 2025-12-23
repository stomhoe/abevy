use bevy::ecs::entity_disabling::Disabled;
#[allow(unused_imports)] use bevy::prelude::*;
use bevy_ecs_tilemap::tiles::TileColor;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
#[allow(unused_imports)] use bevy_asset_loader::prelude::*;
use common::common_components::{EntityPrefix, StrId};
use tilemap_shared::{AcGlobalGenSettings, GlobalTilePos};

use crate::{color_sampler_resources::*, game_common_components_samplers::{ColorSampler, ColorSamplerRef, }};

#[allow(unused_parens)]
pub fn init_color_samplers(
    mut cmd: Commands,
    mut sampler_handles: ResMut<ColorWeightedSamplerHandles>,
    mut assets: ResMut<Assets<WeightedColorsSeri>>,
    color_map: Option<Res<ColorWeightedSamplersMap>>,
) {
    if color_map.is_some() { return; }
    cmd.insert_resource(ColorWeightedSamplersMap::default());

    for handle in sampler_handles.handles.drain(..) {
        let Some(mut seri) = assets.remove(&handle) else { continue; };

        let str_id = match StrId::new_with_result(seri.id.clone(), WeightedColorsSeri::MIN_ID_LENGTH) {
            Ok(id) => id,
            Err(err) => {
                error!("Failed to create StrId for color sampler '{}': {}", seri.id, err);
                continue;
            }
        };
        if seri.weights.is_empty() {
            warn!("Color sampler '{}' has no weights", str_id);
        }
        let mut i = 0;
        while i < seri.weights.len() {
            if seri.weights[i].1 < 0.0 {
                error!(
                    "Invalid color sampler '{}': negative weight detected at index {} (color value: {:?}, weight: {}). Removing this entry.",
                    str_id, i, seri.weights[i].0, seri.weights[i].1
                );
                seri.weights.swap_remove(i);
                // Do not increment i, as swap_remove puts a new element at i
            } else {
                i += 1;
            }
        }

        let wmap = ColorSampler::new(&seri.weights);

        cmd.spawn((
            str_id.clone(),
            wmap,
        ));
    }
}

#[allow(unused_parens, )]
pub fn add_colorsamplers_to_map(
    mut cmd: Commands,
    map: Option<ResMut<ColorWeightedSamplersMap>>,
    query: Query<(Entity, &EntityPrefix, &StrId), (Added<ColorSampler>, )>,
) {
    let Some(mut map) = map else { return; };
    for (new_ent, prefix, str_id) in query.iter() {
        if let Err(err) = map.0.insert(str_id, new_ent, ) {
            error!("{} {} already in ColorWeightedSamplersMap : {}", prefix, str_id, err);
            cmd.entity(new_ent).try_despawn();
        } else {
            info!("Inserted tile '{}' into ColorWeightedSamplersMap with entity {:?}", str_id, new_ent);
        }
    }
}

#[allow(unused_parens)]
pub fn apply_pos_sampled_color(mut cmd: Commands, 
    gen_settings: Single<&AcGlobalGenSettings>,
    samplers: Query<&ColorSampler>,
    mut query: Query<(Entity, &ColorSamplerRef, &GlobalTilePos, AnyOf<(&mut Sprite, &mut TileColor)>), (Or<(Changed<ColorSamplerRef>, Added<Sprite> )>, )>,
) {
    for (entity, color_sampler, global_tile_pos, (sprite, tile_color)) in query.iter_mut() {
        if let Ok(sampler) = samplers.get(color_sampler.0) {
            let color = sampler.sample_with_pos(&gen_settings, *global_tile_pos).unwrap_or([255, 255, 255, 255]);
            let color: Color = Color::srgba_u8(color[0], color[1], color[2], color[3]);
            if let Some(mut sprite) = sprite {
                sprite.color = color;
            } else if let Some(mut tile_color) = tile_color {
                tile_color.0 = color;
            }
        }
        cmd.entity(entity).try_remove::<ColorSamplerRef>();
    }
}
