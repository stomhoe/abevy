
/// Creates an EntityMap struct with systems and plugin.
/// Generates struct with forward mapping (id -> entity).
///
/// # Example
/// ```ignore
/// define_entity_map_systems!(
///     ColorWeightedSamplersMap,
///     StrId,
///     ColorSampler
/// );
/// ```
///
#[macro_export]
macro_rules! define_entity_map_systems {
    // Simplified version - most common case  
    (
        $map_name:ident,
        $id_type:ty,
        $watch_component:ty
    ) => {
        $crate::define_entity_map_systems!(
            $map_name,
            $id_type,
            $watch_component,
            (),
            $watch_component,
            "",
            ""
        );
    };

    // With additional filters
    (
        $map_name:ident,
        $id_type:ty,
        $watch_component:ty,
        $with_filters:ty
    ) => {
        $crate::define_entity_map_systems!(
            $map_name,
            $id_type,
            $watch_component,
            $with_filters,
            $watch_component,
            "",
            ""
        );
    };

    // Full version with all parameters
    (
        $map_name:ident,
        $id_type:ty,
        $watch_component:ty,
        $with_filters:ty,
        $despawn_trigger:ty,
        $target:expr,
        $entity_type:expr
    ) => {
        paste::paste! {
            #[derive(bevy::prelude::Resource, Debug, Clone, Reflect)]
            #[reflect(Resource)]
            pub struct $map_name(pub common::common_types::HashIdToEntityMap);

            impl Default for $map_name {
                fn default() -> Self {
                    Self(Default::default())
                }
            }

            pub fn [<map_ $map_name:snake _id_to_entity>](
                mut cmd: Commands,
                map: Option<ResMut<$map_name>>,
                query: Query<(Entity, Option<&common::common_components::Prefix>, &$id_type), (Changed<$id_type>, With<$watch_component>, $with_filters)>,
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
                                stringify!($map_name),
                                prev_ent,
                                entity
                            );
                            cmd.entity(entity).try_despawn();
                        } else if !$target.is_empty() {
                            trace!(
                                target: $target,
                                "Inserted {} '{}' into {} with entity {:?}",
                                $entity_type,
                                id,
                                stringify!($map_name),
                                entity
                            );
                        }
                    }
                } else if !$target.is_empty() {
                    error!(
                        target: $target,
                        "{} resource not found when trying to add {} to it.",
                        stringify!($map_name),
                        $entity_type
                    );
                }
            }

            pub fn [<remove_ $watch_component:snake _from_ $map_name:snake _on_despawn>](
                trigger: On<bevy::prelude::Despawn, $despawn_trigger>,
                query: Query<(&$id_type), $with_filters>,
                mut map: ResMut<$map_name>,
            ) {
                if let Ok(id) = query.get(trigger.entity) {
                    if let Ok(found_entity) = map.0.get_cloned(id) {
                        if found_entity == trigger.entity {
                            map.0.remove(id.as_str());
                        }
                    }
                }
            }

            pub fn [<plugin_ $map_name:snake>](app: &mut App) {
                use bevy_replicon::prelude::AppRuleExt;
                app.init_resource::<$map_name>()
                    .register_type::<$map_name>()
                    .add_systems(Update, [<map_ $map_name:snake _id_to_entity>])
                    .add_observer([<remove_ $watch_component:snake _from_ $map_name:snake _on_despawn>])
                    .replicate::<$watch_component>()
                    ;
            }
        }
    };
}