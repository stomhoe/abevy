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

    for mut seri in load_weighted_colors_seri_defs() {

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

        let mut i = 0;
        while i < seri.weights.len() {
            if seri.weights[i].1 < 0.0 {
                error!(
                    target: "color_sampler_init",
                    "Invalid color sampler '{}': negative weight detected at index {} (color value: {:?}, weight: {}). Removing this entry.",
                    str_id,
                    i,
                    seri.weights[i].0,
                    seri.weights[i].1
                );
                seri.weights.swap_remove(i);
            } else {
                i += 1;
            }
        }

        let ent = cmd.spawn_empty().id();
        let wmap = ColorSampler::new(&seri.weights);
        wmap_to_insert.push((ent, (str_id, wmap)));
    }

    cmd.insert_batch(wmap_to_insert);
}

#[allow(unused_parens)]
pub fn apply_pos_sampled_color(
    mut cmd: Commands,
    gen_settings: Query<&GlobalGenSettings>,
    samplers: Query<&ColorSampler>,
    dim_hash_query: Query<&HashId, common::AnyDisabling>,
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
            let Ok(sampler) = samplers.get(color_sampler.0) else {
                return;
            };

            let dimension_hash = dimension_ref
                .and_then(|dim_ref| dim_hash_query.get(dim_ref.0).ok())
                .cloned()
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
