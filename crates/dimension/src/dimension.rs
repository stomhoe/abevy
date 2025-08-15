use bevy::prelude::*;
use bevy_common_assets::ron::RonAssetPlugin;
use bevy_replicon::prelude::*;
use common::common_states::AssetsLoadingState;
use game_common::{GameplaySystems, StatefulSessionSystems};
use crate::{
    dimension_components::*, dimension_resources::*, dimension_systems::*, dimension_init_systems::*
//    dimension_events::*,
};

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct DimensionSystems;

pub fn plugin(app: &mut App) {
    app
        .add_plugins((
            RonAssetPlugin::<DimensionSeri>::new(&["dimension.ron"]),
        ))
        .add_systems(OnEnter(AssetsLoadingState::ReplicatedFinished), (
            (init_dimensions, add_dimensions_to_map).chain(),
        ))
        .add_systems(Update, (
            add_dimensions_to_map.run_if(not(server_or_singleplayer)),
            (replace_multiple_string_refs_by_entity_refs, dim_replace_string_ref_by_entity_ref).run_if(server_or_singleplayer.and(in_state(AssetsLoadingState::ReplicatedFinished))),

        ).in_set(StatefulSessionSystems).in_set(DimensionSystems))

        .replicate_bundle::<(Dimension, Transform, GlobalTransform)>()
        .replicate_with((
            RuleFns::<Dimension>::default(),
            RuleFns::<Transform>::default(),
            (RuleFns::<GlobalTransform>::default(), SendRate::Once),
        ))
        .replicate::<DimensionRef>()
        .replicate::<MultipleDimensionRefs>()
        .register_type::<DimensionRef>()
        .register_type::<MultipleDimensionRefs>()
    ;
}