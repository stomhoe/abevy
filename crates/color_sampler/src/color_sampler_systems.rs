#[allow(unused_imports)] use bevy::prelude::*;
use bevy_ecs_tilemap::tiles::TileColor;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
use common::common_components::{AnyDisabling, HashId, Prefix, StrId};
use ::tilemap_shared::*;
use dimension_shared::DimensionRef;
use crate::{color_sampler_components::*, color_sampler_resources::* };

#[allow(unused_parens)]
pub fn init_color_samplers(
    mut cmd: Commands,
    mut sampler_handles: ResMut<ColorWeightedSamplerHandles>,
    mut assets: ResMut<Assets<WeightedColorsSeri>>,
    color_map: Res<ColorWeightedSamplersMap>,
) {
    if ! color_map.0.is_empty() { return; }

    let mut wmap_to_insert = Vec::new();

    for handle in sampler_handles.handles.drain(..) {
        let Some(mut seri) = assets.remove(&handle) else { continue; };

        let str_id = match StrId::new_with_result(seri.id.clone(), WeightedColorsSeri::MIN_ID_LENGTH) {
            Ok(id) => id,
            Err(err) => {
                error!(target: "color_sampler_init", "Failed to create StrId for color sampler '{}': {}", seri.id, err);
                continue;
            }
        };
        if seri.weights.is_empty() {
            warn!(target: "color_sampler_init", "Color sampler '{}' has no weights", str_id);
        }
        let mut i = 0;
        while i < seri.weights.len() {
            if seri.weights[i].1 < 0.0 {
                error!(
                    target: "color_sampler_init",
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

        let ent = cmd.spawn_empty().id(); 

        wmap_to_insert.push((ent, (str_id, wmap.clone())));

    }
    cmd.insert_batch(wmap_to_insert);
}

#[allow(unused_parens, )]
pub fn map_colorsampler_id_to_entity(
    mut cmd: Commands,
    map: Option<ResMut<ColorWeightedSamplersMap>>,
    query: Query<(Entity, Option<&Prefix>, &StrId), (Changed<StrId>, With<ColorSampler>)>,
) {
    let Some(mut map) = map else { return; };
    for (new_ent, prefix, str_id) in query.iter() {
        if let Err(err) = map.0.insert(str_id, new_ent, ) {
            if err.0 == new_ent {
                continue;
            }
            error!(target: "color_sampler_init","{} '{}' already in ColorWeightedSamplersMap with entity {:?}, cannot insert entity {:?}", prefix.cloned().unwrap_or_default(), str_id, err, new_ent);
            cmd.entity(new_ent).try_despawn();
        } else {
            info!(target: "color_sampler_init", "Inserted tile '{}' into ColorWeightedSamplersMap with entity {:?}", str_id, new_ent);
        }
    }
}

#[allow(unused_parens)]
pub fn apply_pos_sampled_color(mut cmd: Commands, 
    gen_settings: Single<&GlobalGenSettings>,
    samplers: Query<&ColorSampler>,
    dim_hash_query: Query<&HashId, AnyDisabling>,
    mut query: Query<(Entity, &ColorSamplerRef, &GlobalTilePos, Option<&DimensionRef>, AnyOf<(&mut Sprite, &mut TileColor)>), (Or<(Changed<ColorSamplerRef>, Added<Sprite> )>, )>,
) {
    query.iter_mut().for_each(|(entity, color_sampler, &global_tile_pos, dimension_ref, (sprite, tile_color))| {
        let Ok(sampler) = samplers.get(color_sampler.0) 
        else {return;};

        let dimension_hash = dimension_ref
            .and_then(|dim_ref| dim_hash_query.get(dim_ref.0).ok())
            .cloned()
            .unwrap_or_default();

        let color = sampler.sample_with_pos(global_tile_pos, &gen_settings, dimension_hash).unwrap_or([255, 255, 255, 255]);
        let color: Color = Color::srgba_u8(color[0], color[1], color[2], color[3]);
        if let Some(mut sprite) = sprite {
            sprite.color = color;
        } else if let Some(mut tile_color) = tile_color {
            tile_color.0 = color;
        }
        cmd.entity(entity).try_remove::<ColorSamplerRef>();
    });
}

#[allow(unused_parens)]
pub fn remove_color_sampler_from_map_on_despawn(
    trigger: On<Despawn, (ColorSampler )>,
    query: Query<(&StrId),(AnyDisabling)>,
    mut map: ResMut<ColorWeightedSamplersMap>,
) {
    if let Ok(str_id) = query.get(trigger.entity) {
        if let Ok(found_entity) = map.0.get_cloned(str_id) {
            if found_entity == trigger.entity {
                map.0.remove(str_id.as_str());
            }
        }
    }
}