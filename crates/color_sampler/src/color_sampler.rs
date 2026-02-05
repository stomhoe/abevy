use bevy::{prelude::*};
use bevy_common_assets::ron::RonAssetPlugin;
use common::{define_entity_map_systems, common_components::*, common_states::*, };
use bevy_replicon::prelude::*;

use crate::{color_sampler_components::*, color_sampler_resources::*, color_sampler_systems::*};

define_entity_map_systems!(
    ColorWeightedSamplersMap,
    StrId,
    ColorSampler
);

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct ColorSampleSystems;

pub fn plugin(app: &mut App) {
    
    app
    .add_plugins((
        RonAssetPlugin::<WeightedColorsSeri>::new(&["wcolors.ron"]),
        plugin_color_weighted_samplers_map,
    ))
    .add_systems(OnEnter(AssetLoading::SpawnReplicatedEntities), (init_color_samplers, map_color_weighted_samplers_map_id_to_entity).chain().in_set(ColorSampleSystems))
    .add_systems(Update, (
        (apply_pos_sampled_color).in_set(ColorSampleSystems),
    ))

    .register_type::<ColorWeightedSamplerHandles>()
    .register_type::<ColorSamplerRef>()

    .replicate::<ColorSampler>()

;}