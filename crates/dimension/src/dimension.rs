use bevy::prelude::*;
use bevy_replicon::prelude::*;
use common::common_states::AssetLoading;
use ::tilemap_shared::*;
use ::game_common::*;
use crate::{
    dimension_systems::*, dimension_init_systems::*
//    dimension_events::*,
};

pub fn plugin(app: &mut App) {
    app
        .add_plugins((
            plugin_dimension,
        ))
        .add_systems(OnEnter(AssetLoading::SpawnReplicatedEntities), (
            (init_dimensions, map_dimension_id_to_entity).chain().in_set(DimensionSystems),
        ))
        .add_systems(Update, (
            (replace_multiple_string_refs_by_entity_refs, replace_dim_string_ref_by_entity_ref, replace_portal_tile_string_ref_by_entity_ref).run_if(in_state(ClientState::Disconnected)
            .and(in_state(AssetLoading::SpawnReplicatedEntities))),
            spawn_egui_macro_chunk_holders,
            ensure_childof_for_enti_with_dimension_ref_and_readjust_if_parent_was_dimension,
        ).in_set(StatefulSessionSystems).in_set(DimensionSystems))




        .replicate::<MultipleDimensionRefs>()
        .replicate::<Gravity>()
        .replicate::<DimensionRootOplist>()
        .replicate::<WhitelistedStructureGenTags>()
        .replicate::<BlacklistedStructureGenTags>()
    ;
}
