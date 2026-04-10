#[allow(unused_imports)]
use bevy::prelude::*;
use bevy_ecs_tilemap::tiles::TileColor;
use common::common_components::*;
use ::tilemap_shared::*;

use crate::*;
use crate::load_weighted_colors_seri_defs;

#[allow(unused_parens)]
pub fn init_color_samplers(
    mut cmd: Commands,
    color_map: Res<ColorSamplerEntityMap>,
) {
    if !color_map.0.is_empty() {
        return;
    }

    let mut wmap_to_insert = Vec::new();

    for seri in load_weighted_colors_seri_defs() {

        let str_id = match StrId::new_with_result(seri.id.clone(), WeightedColorsSeri::MIN_ID_LENGTH)
        {
            Ok(id) => id,
            Err(err) => {
                error!(
                    target: "color_sampler_init",
                    "Failed to create StrId for color sampler '{}': {}",
                    seri.id,
                    err
                );
                continue;
            }
        };

        if seri.weights.is_empty() {
            warn!(
                target: "color_sampler_init",
                "Color sampler '{}' has no weights",
                str_id
            );
        }

        let ent = cmd.spawn_empty().id();
        let (wmap, negative_indices) = ColorSampler::new(&seri.weights);
        if !negative_indices.is_empty() {
            tilemap_shared::log_negative_weighted_sampler_indices!("color_sampler_init", &str_id, &seri.weights, negative_indices);
        }
        wmap_to_insert.push((ent, (str_id, wmap)));
    }

    cmd.insert_batch(wmap_to_insert);
}

#[allow(unused_parens)]
pub fn apply_pos_sampled_color(
    mut cmd: Commands,
    gen_settings: Query<&GlobalGenSettings>,
    sampler_map: Res<ColorSamplerEntityMap>,
    samplers: Query<&ColorSampler>,
    mut query: Query<
        (
            Entity,
            &ColorSamplerRef,
            &GlobalTilePos,
            Option<&DimensionRef>,
            AnyOf<(&mut Sprite, &mut TileColor)>,
        ),
        (Or<(Changed<ColorSamplerRef>, Added<Sprite>)>,),
    >,
) {
    if query.is_empty() {
        return;
    }

    let Ok(gen_settings) = gen_settings.single() else {
        error!("Failed to get gen settings");
        return;
    };

    query.iter_mut().for_each(
        |(entity, color_sampler, &global_tile_pos, dimension_ref, (sprite, tile_color))| {
            let Ok(sampler_ent) = sampler_map.0.get_cloned(color_sampler.0) else {
                return;
            };
            let Ok(sampler) = samplers.get(sampler_ent) else {
                return;
            };

            let dimension_hash = dimension_ref
                .map(|dim_ref| dim_ref.0)
                .unwrap_or_default();

            let color = sampler
                .sample_with_pos(global_tile_pos, &gen_settings, dimension_hash)
                .unwrap_or([255, 255, 255, 255]);
            let color: Color = Color::srgba_u8(color[0], color[1], color[2], color[3]);

            if let Some(mut sprite) = sprite {
                sprite.color = color;
            } else if let Some(mut tile_color) = tile_color {
                tile_color.0 = color;
            }

            cmd.entity(entity).try_remove::<ColorSamplerRef>();
        },
    );
}
