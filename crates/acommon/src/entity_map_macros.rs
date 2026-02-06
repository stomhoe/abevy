
/// Creates an EntityMap struct with systems and plugin.
/// Generates struct with forward mapping (id -> entity).
///
/// # Example
/// ```ignore
/// define_entity_map_systems!(
///     StrId,
///     ColorSampler
/// );
/// ```
///
#[macro_export]
macro_rules! define_entity_map_systems {
    // Simplified version - most common case
    (
        $id_type:ty,
        $main_component:ty
    ) => {
        $crate::define_entity_map_systems!(
            $id_type,
            $main_component,
            (),
            $main_component,
            "",
            ""
        );
    };

    // With additional filters
    (
        $id_type:ty,
        $main_component:ty,
        $with_filters:ty
    ) => {
        $crate::define_entity_map_systems!(
            $id_type,
            $main_component,
            $with_filters,
            $main_component,
            "",
            ""
        );
    };

    // Full version with all parameters
    (
        $id_type:ty,
        $main_component:ty,
        $with_filters:ty,
        $despawn_trigger:ty,
        $target:expr,
        $entity_prefix:expr
    ) => {
        paste::paste! {
            #[derive(bevy::prelude::Resource, std::fmt::Debug, Clone, Reflect)]
            #[reflect(Resource)]
            pub struct [<$main_component EntityMap>](pub common::common_types::HashIdToEntityMap);

            impl Default for [<$main_component EntityMap>] {
                fn default() -> Self {
                    Self(Default::default())
                }
            }

            #[derive(Component, std::fmt::Debug, serde::Deserialize, serde::Serialize, Copy, Clone, std::hash::Hash, PartialEq, Eq, Reflect, bevy::ecs::entity::MapEntities, )]
            pub struct [<$main_component Ref>](#[entities] pub Entity);

            pub fn [<map_ $main_component:snake _id_to_entity>](
                mut cmd: Commands,
                map: Option<ResMut<[<$main_component EntityMap>]>>,
                query: Query<(Entity, Option<&common::common_components::Prefix>, &$id_type), (Changed<$id_type>, With<$main_component>, $with_filters)>,
            ) {
                if let Some(mut map) = map {
                    for (entity, prefix, id) in query.iter() {
                        if let Err(prev_ent) = map.0.insert(id, entity) {
                            if prev_ent.0 == entity {
                                continue;
                            }
                            error!(
                                target: $target,
                                "{} '{}' already in {} with entity {:?}, cannot insert entity {:?}",
                                prefix.cloned().unwrap_or_default(),
                                id,
                                stringify!($main_component),
                                prev_ent,
                                entity
                            );
                            cmd.entity(entity).try_despawn();
                        } else if !$target.is_empty() {
                            trace!(
                                target: $target,
                                "Inserted {} '{}' into {} with entity {:?}",
                                $entity_prefix,
                                id,
                                stringify!($main_component),
                                entity
                            );
                        }
                    }
                } else if !$target.is_empty() {
                    error!(
                        target: $target,
                        "{} resource not found when trying to add {} to it.",
                        stringify!($main_component),
                        $entity_prefix
                    );
                }
            }

            pub fn [<remove_ $main_component:snake _from_ $main_component:snake _on_despawn>](
                trigger: On<bevy::prelude::Despawn, $despawn_trigger>,
                query: Query<(&$id_type), $with_filters>,
                mut map: ResMut<[<$main_component EntityMap>]>,
            ) {
                if let Ok(id) = query.get(trigger.entity) {
                    if let Ok(found_entity) = map.0.get_cloned(id) {
                        if found_entity == trigger.entity {
                            map.0.remove(id.as_str());
                        }
                    }
                }
            }

            pub fn [<plugin_ $main_component:snake>](app: &mut App) {
                use bevy_replicon::prelude::AppRuleExt;
                app.init_resource::<[<$main_component EntityMap>]>()
                    .register_type::<[<$main_component EntityMap>]>()
                    .register_type::<[<$main_component Ref>]>()
                    .add_systems(Update, [<map_ $main_component:snake _id_to_entity>])
                    .add_observer([<remove_ $main_component:snake _from_ $main_component:snake _on_despawn>])
                    .replicate::<$main_component>()
                    .replicate::<[<$main_component Ref>]>()
                    ;
            }
        }
    };
}
