use bevy::prelude::*;
use bevy_common_assets::ron::RonAssetPlugin;
use bevy_replicon::prelude::*;
use common::common_states::AssetLoading;
use ::dimension_shared::*;
use game_common::{GameplaySystems, StatefulSessionSystems};
use crate::{
    dimension_resources::*, dimension_systems::*, dimension_init_systems::*
//    dimension_events::*,
};

pub fn plugin(app: &mut App) {
    app
        .add_plugins((
            RonAssetPlugin::<DimensionSeri>::new(&["dimension.ron"]),
        ))
        .add_systems(OnEnter(AssetLoading::SpawnReplicatedEntities), (
            (init_dimensions, add_dimensions_to_map).chain().in_set(DimensionSystems),
        ))
        .add_systems(Update, (
            (replace_multiple_string_refs_by_entity_refs, replace_dim_string_ref_by_entity_ref).run_if(in_state(ClientState::Disconnected)
            .and(in_state(AssetLoading::SpawnReplicatedEntities))),

            readjust_childof_to_new_dim_if_parent_was_dimension,
        ).in_set(StatefulSessionSystems).in_set(DimensionSystems))

        .register_type::<DimensionRef>()
        .register_type::<MultipleDimensionRefs>()
        .register_type::<DimensionRootOplist>()
        .register_type::<RootInDimensions>()

        .replicate_once_filtered::<Transform, With<Dimension>>()

        .replicate::<Dimension>()
        .replicate::<DimensionRef>()
        .replicate::<MultipleDimensionRefs>()
        .replicate::<DimensionRootOplist>()
        .replicate::<WhitelistedStructureGenTags>()
        .replicate::<BlacklistedStructureGenTags>()
    ;
}