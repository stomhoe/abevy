use bevy::{ecs::entity_disabling::Disabled, prelude::*};
use bevy_common_assets::ron::RonAssetPlugin;
use common::{common_components::{AnyDisabling, ImagePathHolder}, common_states::*, common_types::*};
use bevy_replicon::prelude::*;

use crate::{color_sampler_components::*, color_sampler_resources::*, color_sampler_systems::*};


#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct ColorSampleSystems;

pub fn plugin(app: &mut App) {
    app
    .add_plugins((
        RonAssetPlugin::<WeightedColorsSeri>::new(&["wcolors.ron"]),
        
    ))
    .add_systems(OnEnter(AssetLoading::SpawnReplicatedEntities), (init_color_samplers, map_colorsampler_id_to_entity).chain().in_set(ColorSampleSystems))
    .add_systems(Update, (
        (apply_pos_sampled_color).in_set(ColorSampleSystems),
        map_colorsampler_id_to_entity,
    ))
    .add_observer(remove_color_sampler_from_map_on_despawn)

    .register_type::<ColorWeightedSamplerHandles>()
    .register_type::<ColorSamplerRef>()

    .replicate::<ColorSampler>()

    .init_resource::<ColorWeightedSamplersMap>()
;}