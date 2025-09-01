use bevy::ecs::entity_disabling::Disabled;
#[allow(unused_imports)] use bevy::prelude::*;
use bevy_ecs_tilemap::tiles::TileColor;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
#[allow(unused_imports)] use bevy_asset_loader::prelude::*;
use bevy_replicon::shared::server_entity_map::ServerEntityMap;
use common::common_components::{DisplayName, EntityPrefix, ImageHolder, ImageHolderMap, StrId};
use common::{common_states::GameSetupType};
use tilemap_shared::{AaGlobalGenSettings, GlobalTilePos};

use crate::{color_sampler_resources::*, game_common_components_samplers::{ColorSampler, ColorSamplerRef, WeightedSampler}};

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
        let Some(seri) = assets.remove(&handle) else { continue; };

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
    mut refs_to_update: Query<(&mut ColorSamplerRef), (Or<(With<Disabled>, Without<Disabled>)>, )>,
    state : Res<State<GameSetupType>>,
) {
    let is_host = state.get() != &GameSetupType::AsJoiner;

    let Some(mut map) = map else { return; };
    for (new_ent, prefix, str_id) in query.iter() {
        if is_host {
            if let Err(err) = map.0.insert(str_id, new_ent, ) {
                error!("{} {} already in ColorWeightedSamplersMap : {}", prefix, str_id, err);
                cmd.entity(new_ent).despawn();
            } else {
                info!("Inserted tile '{}' into ColorWeightedSamplersMap with entity {:?}", str_id, new_ent);
            }
        }
        else if let Some(prev_ent) = map.0.force_insert(str_id, new_ent, ) {
            cmd.entity(prev_ent).try_despawn();

            refs_to_update.iter_mut().for_each(|mut ref_to_upd| {
                if ref_to_upd.0 == prev_ent {
                    ref_to_upd.0 = new_ent;
                }
            });
        }
        
    }
}

#[allow(unused_parens)]
pub fn apply_color(mut cmd: Commands, 
    gen_settings: Option<Res<AaGlobalGenSettings>>,
    samplers: Query<&ColorSampler>,
    mut query: Query<(Entity, &ColorSamplerRef, &GlobalTilePos, AnyOf<(&mut Sprite, &mut TileColor)>)>,
) {
    let Some(gen_settings) = gen_settings else { return; };
    for (entity, color_sampler, global_tile_pos, (sprite, tile_color)) in query.iter_mut() {
        if let Ok(sampler) = samplers.get(color_sampler.0) {
            let color = sampler.0.sample_with_pos(&gen_settings, *global_tile_pos).unwrap_or([255, 255, 255, 255]);
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
