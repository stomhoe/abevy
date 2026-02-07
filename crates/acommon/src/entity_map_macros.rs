#[macro_export]
macro_rules! define_entity_map_systems {
    // Simplified version - most common case
    (
        $main_component:ident
        $(, $seri_type:ty, $ron_dir:literal, $ron_suffix:literal )*
        $(,)?
    ) => {
        $crate::define_entity_map_systems!($main_component, (), $main_component, stringify!($main_component:snake), "", $main_component, common::common_components::StrId $(, $seri_type, $ron_dir, $ron_suffix )*);
    };

    // With additional filters (using StrId by default)
    (
        $main_component:ident,
        $with_filters:ty
        $(, $seri_type:ty, $ron_dir:literal, $ron_suffix:literal )*
        $(,)?
    ) => {
        $crate::define_entity_map_systems!($main_component, $with_filters, $main_component, stringify!($main_component:snake), "", $main_component, common::common_components::StrId $(, $seri_type, $ron_dir, $ron_suffix )*);
    };

    // With additional filters and custom id type
    (
        $main_component:ident,
        $with_filters:ty,
        $despawn_trigger:ty
        $(, $seri_type:ty, $ron_dir:literal, $ron_suffix:literal )*
        $(,)?
    ) => {
        $crate::define_entity_map_systems!(
            $main_component,
            $with_filters,
            $main_component,
            stringify!($main_component:snake),
            "",
            $despawn_trigger,
            common::common_components::StrId
            $(, $seri_type, $ron_dir, $ron_suffix )*
        );
    };

    // Full version with all parameters (with asset variadics)
    (
        $main_component:ident,
        $with_filters:ty,
        $abbreviation: ident,
        $target:expr,
        $entity_prefix:expr,
        $despawn_trigger:ty,
        $id_type:ty
        $(,)?
    ) => {
        paste::paste! {
            #[derive(Component, Debug, Default, serde::Deserialize, serde::Serialize, Clone, Reflect)]
            #[require(common::common_components::SparedFromHotReloading, common::common_components::AssetScoped, common::common_id_components::Prefix::trunc(concat!("Egui", stringify!($main_component), "Holder")), bevy_replicon::shared::replication::Replicated, Visibility, Transform)]
            pub struct [<Egui $abbreviation sHolder>];

            #[derive(bevy::prelude::Resource, std::fmt::Debug, Clone, Reflect)]
            #[reflect(Resource)]
            pub struct [<$main_component EntityMap>](pub common::common_types::HashIdToEntityMap);

            impl Default for [<$main_component EntityMap>] { fn default() -> Self { Self(Default::default()) } }

            #[derive(Component, std::fmt::Debug, serde::Deserialize, serde::Serialize, Copy, Clone, std::hash::Hash, PartialEq, Eq, Reflect, bevy::ecs::entity::MapEntities, )]
            pub struct [<$abbreviation Ref>](#[entities] pub Entity);
            impl [<$abbreviation Ref>] {
                pub fn is_placeholder(&self) -> bool {
                    self.0 == Entity::PLACEHOLDER
                }
            }
            #[derive(Component, std::fmt::Debug, Clone, PartialEq, Eq, Reflect, )]
            pub struct [<$abbreviation StrIdRef>](pub common::common_components::StrId);

            #[derive(Component, std::fmt::Debug, Clone, PartialEq, Eq, Reflect, serde::Deserialize, serde::Serialize, Default)]
            pub struct [<DoNotRetryConvert $abbreviation StrIdRef>](pub common::common_components::StrId);

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
            #[allow(unused_parens)]
            pub fn [<add_ $main_component:snake _ezeros_to_egui_holder>](
                mut cmd: Commands,
                holder_query: Query<(Entity, Option<&Children>), (With<[<Egui $abbreviation sHolder>]>)>,
                query: Query<(Entity), (With<$main_component>, $with_filters, Without<ChildOf>)>,
            ) {
                let holders: Vec<_> = holder_query.iter().collect();
                let holder_id = if holders.is_empty() {
                    cmd.spawn(([<Egui $abbreviation sHolder>],)).id()
                } else {
                    let (max_holder, _) = holders.iter()
                        .map(|(ent, children)| (ent, children.map(|c| c.len()).unwrap_or(0)))
                        .max_by_key(|(_, child_count)| *child_count)
                        .unwrap();
                    for (ent, _) in holders.iter() {
                        if ent != max_holder {
                            cmd.entity(*ent).try_despawn();
                        }
                    }
                    *max_holder
                };

                let mut child_ofs = Vec::with_capacity(query.iter().size_hint().0);
                for ezero_ent in query.iter() {
                    child_ofs.push((ezero_ent, ChildOf(holder_id)));
                }
                cmd.try_insert_batch(child_ofs);
            }
            pub fn [<permit_ $abbreviation:snake _strid_ref_to_ent_ref_retries>](
                mut cmd: Commands,
                query: Query<(Entity), (Changed<[<$abbreviation StrIdRef>]>)>,
            ) {
                for (customer_ent) in query.iter() {
                    cmd.entity(customer_ent).try_remove::<[<DoNotRetryConvert $abbreviation StrIdRef>]>();
                }
            }

            pub fn [<convert_ $abbreviation:snake _strid_ref_to_ent_ref>](
                mut cmd: Commands,
                query: Query<(Entity, &[<$abbreviation StrIdRef>]), (Without<[<DoNotRetryConvert $abbreviation StrIdRef>]>)>,
                bit_emap: Option<Res<[<$main_component EntityMap>]>>,
            ) {
                if query.is_empty() {
                    return;
                }
                let Some(bit_emap) = bit_emap else {
                    error!(target: $target, "{} EntityMap does not exist, inject {} into bevy", stringify!($main_component), stringify!([<plugin_ $main_component:snake>]));
                    return;
                };
                let mut refs = Vec::with_capacity(query.iter().size_hint().0);
                for (customer_ent, str_id_ref) in query.iter() {
                    let Ok(bit_entity) = bit_emap.0.get_cloned(&str_id_ref.0) else {
                        error!(target: $target, "{} StrIdRef '{}' could not be resolved to entity in {}", stringify!($abbreviation), str_id_ref.0, stringify!($main_component EntityMap));
                        cmd.entity(customer_ent).try_insert(([<DoNotRetryConvert $abbreviation StrIdRef>]::default()));
                        continue;
                    };
                    refs.push((customer_ent, [<$abbreviation Ref>](bit_entity)));
                }
                cmd.try_insert_batch(refs);
            }

            pub fn [<plugin_ $main_component:snake>](app: &mut App) {
                use bevy_replicon::prelude::AppRuleExt;
                app
                    .init_resource::<[<$main_component EntityMap>]>()
                    .register_type::<[<$main_component EntityMap>]>()
                    .register_type::<[<$abbreviation Ref>]>()
                    .register_type::<[<Egui $abbreviation sHolder>]>()
                    .add_systems(Update, ([<map_ $main_component:snake _id_to_entity>],
                         [<add_ $main_component:snake _ezeros_to_egui_holder>].run_if(bevy::time::common_conditions::on_timer(core::time::Duration::from_secs(1))),
                         [<permit_ $abbreviation:snake _strid_ref_to_ent_ref_retries>],
                    ))
                    .add_observer([<remove_ $main_component:snake _from_ $main_component:snake _on_despawn>])
                    .replicate::<$main_component>()
                    .replicate::<[<$abbreviation Ref>]>()
                    .replicate::<[<Egui $abbreviation sHolder>]>()
                    .replicate_filtered_as::<Visibility, common::common_components::VisibilityGameState, (With<[<Egui $abbreviation sHolder>]>,)>()
                    ;
            }
        }
    };

    // Full version with all parameters (with asset variadics)
    (
        $main_component:ident,
        $with_filters:ty,
        $abbreviation: ident,
        $target:expr,
        $entity_prefix:expr,
        $despawn_trigger:ty,
        $id_type:ty,
        $($seri_type:ty, $ron_dir:literal, $ron_suffix:literal ),+
        $(,)?
    ) => {
        paste::paste! {
            #[allow(unused_imports)]
            use bevy_asset_loader::prelude::AssetCollection;

            $(
                #[derive(bevy_asset_loader::asset_collection::AssetCollection, Resource, Default, Reflect)]
                #[reflect(Resource, Default)]
                pub struct [<$seri_type sHandles>] {
                    #[asset(path = $ron_dir)]
                    #[asset(collection(typed))]
                    pub handles: Vec<Handle<$seri_type>>,
                }
            )+

            #[derive(Component, Debug, Default, serde::Deserialize, serde::Serialize, Clone, Reflect)]
            #[require(common::common_components::SparedFromHotReloading, common::common_components::AssetScoped, common::common_id_components::Prefix::trunc(concat!("Egui", stringify!($main_component), "Holder")), bevy_replicon::shared::replication::Replicated, Visibility, Transform)]
            pub struct [<Egui $abbreviation sHolder>];

            #[derive(bevy::prelude::Resource, std::fmt::Debug, Clone, Reflect)]
            #[reflect(Resource)]
            pub struct [<$main_component EntityMap>](pub common::common_types::HashIdToEntityMap);

            impl Default for [<$main_component EntityMap>] { fn default() -> Self { Self(Default::default()) } }

            #[derive(Component, std::fmt::Debug, serde::Deserialize, serde::Serialize, Copy, Clone, std::hash::Hash, PartialEq, Eq, Reflect, bevy::ecs::entity::MapEntities, )]
            pub struct [<$abbreviation Ref>](#[entities] pub Entity);
            impl [<$abbreviation Ref>] {
                pub fn is_placeholder(&self) -> bool {
                    self.0 == Entity::PLACEHOLDER
                }
            }
            #[derive(Component, std::fmt::Debug, Clone, PartialEq, Eq, Reflect, )]
            pub struct [<$abbreviation StrIdRef>](pub common::common_components::StrId);

            #[derive(Component, std::fmt::Debug, Clone, PartialEq, Eq, Reflect, serde::Deserialize, serde::Serialize, Default)]
            pub struct [<DoNotRetryConvert $abbreviation StrIdRef>](pub common::common_components::StrId);

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
            #[allow(unused_parens)]
            pub fn [<add_ $main_component:snake _ezeros_to_egui_holder>](
                mut cmd: Commands,
                holder_query: Query<(Entity, Option<&Children>), (With<[<Egui $abbreviation sHolder>]>)>,
                query: Query<(Entity), (With<$main_component>, $with_filters, Without<ChildOf>)>,
            ) {
                let holders: Vec<_> = holder_query.iter().collect();
                let holder_id = if holders.is_empty() {
                    cmd.spawn(([<Egui $abbreviation sHolder>],)).id()
                } else {
                    let (max_holder, _) = holders.iter()
                        .map(|(ent, children)| (ent, children.map(|c| c.len()).unwrap_or(0)))
                        .max_by_key(|(_, child_count)| *child_count)
                        .unwrap();
                    for (ent, _) in holders.iter() {
                        if ent != max_holder {
                            cmd.entity(*ent).try_despawn();
                        }
                    }
                    *max_holder
                };

                let mut child_ofs = Vec::with_capacity(query.iter().size_hint().0);
                for ezero_ent in query.iter() {
                    child_ofs.push((ezero_ent, ChildOf(holder_id)));
                }
                cmd.try_insert_batch(child_ofs);
            }
            pub fn [<permit_ $abbreviation:snake _strid_ref_to_ent_ref_retries>](
                mut cmd: Commands,
                query: Query<(Entity), (Changed<[<$abbreviation StrIdRef>]>)>,
            ) {
                for (customer_ent) in query.iter() {
                    cmd.entity(customer_ent).try_remove::<[<DoNotRetryConvert $abbreviation StrIdRef>]>();
                }
            }

            pub fn [<convert_ $abbreviation:snake _strid_ref_to_ent_ref>](
                mut cmd: Commands,
                query: Query<(Entity, &[<$abbreviation StrIdRef>]), (Without<[<DoNotRetryConvert $abbreviation StrIdRef>]>)>,
                bit_emap: Option<Res<[<$main_component EntityMap>]>>,
            ) {
                if query.is_empty() {
                    return;
                }
                let Some(bit_emap) = bit_emap else {
                    error!(target: $target, "{} EntityMap does not exist, inject {} into bevy", stringify!($main_component), stringify!([<plugin_ $main_component:snake>]));
                    return;
                };
                let mut refs = Vec::with_capacity(query.iter().size_hint().0);
                for (customer_ent, str_id_ref) in query.iter() {
                    let Ok(bit_entity) = bit_emap.0.get_cloned(&str_id_ref.0) else {
                        error!(target: $target, "{} StrIdRef '{}' could not be resolved to entity in {}", stringify!($abbreviation), str_id_ref.0, stringify!($main_component EntityMap));
                        cmd.entity(customer_ent).try_insert(([<DoNotRetryConvert $abbreviation StrIdRef>]::default()));
                        continue;
                    };
                    refs.push((customer_ent, [<$abbreviation Ref>](bit_entity)));
                }
                cmd.try_insert_batch(refs);
            }

            pub fn [<plugin_ $main_component:snake>](app: &mut App) {
                use bevy_replicon::prelude::AppRuleExt;
                use bevy_asset_loader::prelude::*;

                app
                    .init_resource::<[<$main_component EntityMap>]>()
                    .register_type::<[<$main_component EntityMap>]>()
                    .register_type::<[<$abbreviation Ref>]>()
                    .register_type::<[<Egui $abbreviation sHolder>]>()
                    .add_systems(Update, ([<map_ $main_component:snake _id_to_entity>],
                         [<add_ $main_component:snake _ezeros_to_egui_holder>].run_if(bevy::time::common_conditions::on_timer(core::time::Duration::from_secs(1))),
                         [<permit_ $abbreviation:snake _strid_ref_to_ent_ref_retries>],
                    ))
                    .add_observer([<remove_ $main_component:snake _from_ $main_component:snake _on_despawn>])
                    .replicate::<$main_component>()
                    .replicate::<[<$abbreviation Ref>]>()
                    .replicate::<[<Egui $abbreviation sHolder>]>()
                    .replicate_filtered_as::<Visibility, common::common_components::VisibilityGameState, (With<[<Egui $abbreviation sHolder>]>,)>()
                    .configure_loading_state(
                        bevy_asset_loader::prelude::LoadingStateConfig::new(common::common_states::AssetLoading::LoadingAssetsIntoHandles)
                            $(.load_collection::<[<$seri_type sHandles>]>() )+
                    )
                    .add_plugins((
                        $(
                            bevy_common_assets::ron::RonAssetPlugin::<$seri_type>::new(&[$ron_suffix]),
                        )+
                    ))
                    ;
            }
        }
    };
}
