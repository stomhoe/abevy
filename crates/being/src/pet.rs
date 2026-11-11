use ::being_shared::*;
use bevy::prelude::*;
use bevy_replicon::prelude::AppRuleExt;
use common::common_states::AssetLoading;

#[path = "pet_init_systems.rs"]
mod pet_init_systems;

common::define_entity_map_systems!(
    main_component: PetSet,
    with_filters: (),
    abbreviation: PetSet,
    target: common::log_targets::ENTITY_MAP_SYSTEM,
    entity_prefix: "",
    despawn_trigger: PetSet,
    id_type: common::common_components::StrId,
);

#[allow(unused_parens, )]
pub fn plugin(app: &mut App) {
    app.add_plugins(plugin_pet_set)
        .replicate::<PetSet>()
        .replicate::<PetSetData>()
        .add_systems(
            OnEnter(AssetLoading::SpawnReplicatedEntities),
            (pet_init_systems::init_pet_sets, map_pet_set_id_to_entity).chain(),
        );
}
