use ::being_shared::*;
use bevy::prelude::*;
use bevy_replicon::prelude::AppRuleExt;
use common::common_states::AssetLoading;
use common::Tags;

#[path = "class_init_systems.rs"]
mod class_init_systems;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct ClassSystems;

common::define_entity_map_systems!(
    main_component: Class,
    with_filters: (),
    abbreviation: Class,
    target: common::log_targets::ENTITY_MAP_SYSTEM,
    entity_prefix: "",
    despawn_trigger: Class,
    id_type: common::common_components::StrId,
);

common::define_entity_map_systems!(
    main_component: Ability,
    with_filters: (),
    abbreviation: Ability,
    target: common::log_targets::ENTITY_MAP_SYSTEM,
    entity_prefix: "",
    despawn_trigger: Ability,
    id_type: common::common_components::StrId,
);

#[allow(unused_parens, )]
pub fn plugin(app: &mut App) {
    app
        .add_plugins((plugin_class, plugin_ability))
        .replicate::<Class>()
      .replicate::<PlayerSelectableClass>()
      .replicate::<ClassData>()
      .replicate::<StatsDescription>()
      .replicate::<Tags>()
      .replicate::<ClassRequirements>()
        .replicate::<ClassRef>()
        .replicate::<Ability>()
        .replicate::<AbilityRef>()
      .replicate::<AbilityEffects>()
        .add_systems(OnEnter(AssetLoading::SpawnReplicatedEntities), (
            class_init_systems::init_classes,
            class_init_systems::init_abilities,
        ).chain().in_set(ClassSystems));
}
