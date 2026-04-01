use bevy::{prelude::*};
use common::{define_entity_map_systems_no_replicate, common_states::*, };

use crate::{WeightedColorsSeri, color_sampler_components::*, color_sampler_systems::*};

define_entity_map_systems_no_replicate!(
    main_component: ColorSampler,
    with_filters: (),
    abbreviation: ColorSampler,
    target: common::log_targets::ENTITY_MAP_SYSTEM,
    entity_prefix: "ColorSampler",
    despawn_trigger: ColorSampler,
    id_type: common::common_components::StrId,
    assets: [(WeightedColorsSeri, "seri.color_sampler", "wcolors.ron")],
);

pub type ColorWeightedSamplerHandles = WeightedColorsSerisHandles;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct ColorSampleSystems;

pub fn plugin(app: &mut App) {

    app
    .add_plugins((
        plugin_color_sampler,
    ))
    .add_systems(OnEnter(AssetLoading::SpawnReplicatedEntities), (init_color_samplers, map_color_sampler_id_to_entity).chain().in_set(ColorSampleSystems))
    .add_systems(Update, (
        (apply_pos_sampled_color).in_set(ColorSampleSystems),
    ))
;}
