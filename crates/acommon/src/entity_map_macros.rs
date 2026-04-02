#[macro_export]
macro_rules! __entity_map_define_ref_struct {
    ($abbreviation:ident $(,)?) => {
        paste::paste! {
            #[derive(Component, std::fmt::Debug, serde::Deserialize, serde::Serialize, Copy, Clone, std::hash::Hash, PartialEq, Eq, bevy::ecs::entity::MapEntities, )]
            pub struct [<$abbreviation Ref>](#[entities] pub Entity);
            impl [<$abbreviation Ref>] {
                pub fn is_placeholder(&self) -> bool {
                    self.0 == Entity::PLACEHOLDER
                }
            }
        }
    };
    ($abbreviation:ident, no_reflect) => {
        $crate::__entity_map_define_ref_struct!($abbreviation);
    };
    ($abbreviation:ident, reflect_ref) => {
        paste::paste! {
            #[derive(Component, std::fmt::Debug, serde::Deserialize, serde::Serialize, Copy, Clone, std::hash::Hash, PartialEq, Eq, Reflect, bevy::ecs::entity::MapEntities, )]
            pub struct [<$abbreviation Ref>](#[entities] pub Entity);
            impl [<$abbreviation Ref>] {
                pub fn is_placeholder(&self) -> bool {
                    self.0 == Entity::PLACEHOLDER
                }
            }
        }
    };
}

#[macro_export]
macro_rules! __entity_map_register_reflect_type {
    ($app:ident, $ty:ty) => {};
    ($app:ident, no_reflect, $ty:ty) => {};
    ($app:ident, reflect_ref, $ty:ty) => {
        $app.register_type::<$ty>();
    };
}

#[macro_export]
macro_rules! define_entity_map_systems {
    // Simplified version - most common case
    (
        $main_component:ident
        $(, $seri_type:ty, $dynamic_key:literal, $ron_suffix:literal )*
        $(,)?
    ) => {
        $crate::define_entity_map_systems!($main_component, (), $main_component, $crate::log_targets::ENTITY_MAP_SYSTEM, "", $main_component, common::common_components::StrId $(, $seri_type, $dynamic_key, $ron_suffix )*);
    };

    // With additional filters (using StrId by default)
    (
        $main_component:ident,
        $with_filters:ty
        $(, $seri_type:ty, $dynamic_key:literal, $ron_suffix:literal )*
        $(,)?
    ) => {
        $crate::define_entity_map_systems!($main_component, $with_filters, $main_component, $crate::log_targets::ENTITY_MAP_SYSTEM, "", $main_component, common::common_components::StrId $(, $seri_type, $dynamic_key, $ron_suffix )*);
    };

    // With additional filters and custom id type
    (
        $main_component:ident,
        $with_filters:ty,
        $despawn_trigger:ty
        $(, $seri_type:ty, $dynamic_key:literal, $ron_suffix:literal )*
        $(,)?
    ) => {
        $crate::define_entity_map_systems!(
            $main_component,
            $with_filters,
            $main_component,
            $crate::log_targets::ENTITY_MAP_SYSTEM,
            "",
            $despawn_trigger,
            common::common_components::StrId
            $(, $seri_type, $dynamic_key, $ron_suffix )*
        );
    };

    // Positional compatibility wrapper -> named parameters (no asset variadics)
    (
        $main_component:ident,
        $with_filters:ty,
        $abbreviation: ident,
        $target:expr,
        $entity_prefix:expr,
        $despawn_trigger:ty,
        $id_type:ty
        $(, $ref_reflect:ident)?
        $(,)?
    ) => {
        $crate::define_entity_map_systems!(
            main_component: $main_component,
            with_filters: $with_filters,
            abbreviation: $abbreviation,
            target: $target,
            entity_prefix: $entity_prefix,
            despawn_trigger: $despawn_trigger,
            id_type: $id_type
            $(, ref_reflect: $ref_reflect)?
        );
    };

    // Full version with named parameters (no asset variadics)
    (
        main_component: $main_component:ident,
        with_filters: $with_filters:ty,
        abbreviation: $abbreviation: ident,
        target: $target:expr,
        entity_prefix: $entity_prefix:expr,
        despawn_trigger: $despawn_trigger:ty,
        id_type: $id_type:ty
        $(, ref_reflect: $ref_reflect:ident)?
        $(,)?
    ) => {
        paste::paste! {
            #[derive(Component, Debug, Default, serde::Deserialize, serde::Serialize, Clone, )]
            #[require(common::common_components::SparedFromHotReloading, common::common_components::AssetScoped, common::common_id_components::Prefix::trunc(concat!("Egui", stringify!($main_component), "Holder")), bevy_replicon::shared::replication::Replicated, Visibility::Hidden, Transform)]
            pub struct [<Egui $abbreviation sHolder>];

            #[derive(bevy::prelude::Resource, std::fmt::Debug, Clone)]
            pub struct [<$main_component EntityMap>](pub common::common_types::HashIdToEntityMap);

            impl Default for [<$main_component EntityMap>] { fn default() -> Self { Self(Default::default()) } }

            $crate::__entity_map_define_ref_struct!($abbreviation $(, $ref_reflect)?);
            #[derive(Component, std::fmt::Debug, Clone, PartialEq, Eq,
                //Reflect
            )]
            pub struct [<$abbreviation StrIdRef>](pub common::common_components::StrId);
            impl [<$abbreviation StrIdRef>] {
                pub fn asd<S: AsRef<str>>(id: S) -> Self {
                    let str_id = common::common_components::StrId::trunc(id.as_ref());
                    Self(str_id)
                }

            }

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
            pub fn [<add_ $main_component:snake _templs_to_egui_holder>](
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

                let iter = query.iter();
                let mut child_ofs = Vec::with_capacity(iter.size_hint().1.unwrap_or(iter.size_hint().0));
                for templ_ent in iter {
                    child_ofs.push((templ_ent, ChildOf(holder_id)));
                }
                cmd.try_insert_batch(child_ofs);
            }
            pub fn [<convert_ $abbreviation:snake _strid_ref_to_ent_ref>](
                mut cmd: Commands,
                query: Query<(Entity, &[<$abbreviation StrIdRef>]), (Changed<[<$abbreviation StrIdRef>]>)>,
                emap: Option<Res<[<$main_component EntityMap>]>>,
            ) {
                if query.is_empty() {
                    return;
                }
                let Some(emap) = emap else {
                    error!(target: $target, "{} EntityMap does not exist, inject {} into bevy", stringify!($main_component), stringify!([<plugin_ $main_component:snake>]));
                    return;
                };
                let iter = query.iter();
                let mut refs = Vec::with_capacity(iter.size_hint().1.unwrap_or(iter.size_hint().0));
                for (customer_ent, str_id_ref) in iter {
                    let Ok(bit_entity) = emap.0.get_cloned(&str_id_ref.0) else {
                        error_once!(target: $target, "{} StrIdRef '{}' could not be resolved to entity in {}", stringify!($abbreviation), str_id_ref.0, stringify!($main_component EntityMap));
                        continue;
                    };
                    refs.push((customer_ent, [<$abbreviation Ref>](bit_entity)));
                    cmd.entity(customer_ent).try_remove::<[<$abbreviation StrIdRef>]>();
                }
                cmd.try_insert_batch(refs);
            }

            pub fn [<plugin_ $main_component:snake>](app: &mut App) {
                use bevy_replicon::prelude::AppRuleExt;
                app.replicate::<$main_component>()
                    .replicate::<[<Egui $abbreviation sHolder>]>()
                    .replicate::<[<$abbreviation Ref>]>()
                    .replicate_filtered_as::<Visibility, common::common_components::VisibilityGameState, (With<[<Egui $abbreviation sHolder>]>,)>()
                ;
                [<plugin_common_ $main_component:snake>](app);
            }

            pub fn [<plugin_ $main_component:snake _no_replicate>](app: &mut App) {
                [<plugin_common_ $main_component:snake>](app);
            }

            fn [<plugin_common_ $main_component:snake>](app: &mut App) {
                $crate::__entity_map_register_reflect_type!(app, $($ref_reflect,)? [<$abbreviation Ref>]);
                app
                    .init_resource::<[<$main_component EntityMap>]>()
                    //.register_type::<[<Egui $abbreviation sHolder>]>()
                    .add_systems(Update, ([<map_ $main_component:snake _id_to_entity>],
                         [<add_ $main_component:snake _templs_to_egui_holder>].run_if(bevy::time::common_conditions::on_timer(core::time::Duration::from_secs(1))),
                    ))
                    .add_observer([<remove_ $main_component:snake _from_ $main_component:snake _on_despawn>])
                    ;
            }
        }
    };

    // Positional compatibility wrapper -> named parameters (with asset variadics)
    (
        $main_component:ident,
        $with_filters:ty,
        $abbreviation: ident,
        $target:expr,
        $entity_prefix:expr,
        $despawn_trigger:ty,
        $id_type:ty,
        $($seri_type:ty, $dynamic_key:literal, $ron_suffix:literal ),+
        $(, $ref_reflect:ident)?
        $(,)?
    ) => {
        $crate::define_entity_map_systems!(
            main_component: $main_component,
            with_filters: $with_filters,
            abbreviation: $abbreviation,
            target: $target,
            entity_prefix: $entity_prefix,
            despawn_trigger: $despawn_trigger,
            id_type: $id_type,
            assets: [$(($seri_type, $dynamic_key, $ron_suffix)),+]
            $(, ref_reflect: $ref_reflect)?
        );
    };

    // Full version with named parameters (with asset variadics)
    (
        main_component: $main_component:ident,
        with_filters: $with_filters:ty,
        abbreviation: $abbreviation: ident,
        target: $target:expr,
        entity_prefix: $entity_prefix:expr,
        despawn_trigger: $despawn_trigger:ty,
        id_type: $id_type:ty,
        assets: [$(($seri_type:ty, $dynamic_key:literal, $ron_suffix:literal)),+]
        $(, ref_reflect: $ref_reflect:ident)?
        $(,)?
    ) => {
        paste::paste! {
            #[allow(unused_imports)]
            use bevy_asset_loader::prelude::AssetCollection;

            $(
                #[derive(bevy_asset_loader::asset_collection::AssetCollection, Resource, Default, )]
                pub struct [<$seri_type sHandles>] {
                    #[asset(key = $dynamic_key)]
                    #[asset(collection(typed))]
                    pub handles: Vec<Handle<$seri_type>>,
                }

                pub fn [<load_ $seri_type:snake _defs>]() -> Vec<$seri_type> {
                    let db = match common::def_db::DefDatabase::<$seri_type>::load_from_assets_dir_with_type(
                        stringify!($seri_type),
                        &[$ron_suffix],
                        |seri| seri.id.as_str(),
                    ) {
                        Ok(db) => db,
                        Err(err) => {
                            error!(
                                target: $target,
                                "Failed loading {} defs: {err:#}",
                                stringify!($seri_type)
                            );
                            return Vec::new();
                        }
                    };
                    if !$target.is_empty() {
                        for ov in db.overrides() {
                            info!(
                                target: $target,
                                "{} def '{}' overridden: '{}' -> '{}'",
                                stringify!($seri_type),
                                ov.id,
                                ov.previous_source.rel_path,
                                ov.replacement_source.rel_path
                            );
                        }
                    }
                    db.into_records().into_iter().map(|r| r.value).collect()
                }
            )+

            fn [<do_register_ $main_component:snake _dynamic_assets>](
                dynamic_assets: &mut bevy_asset_loader::dynamic_asset::DynamicAssets,
            ) {
                $(
                    common::common_resources::register_seri_dynamic_asset_key(dynamic_assets, $dynamic_key);
                )*
            }
            pub fn [<register_ $main_component:snake _dynamic_assets>](
                mut dynamic_assets: ResMut<bevy_asset_loader::dynamic_asset::DynamicAssets>,
            ) {
                [<do_register_ $main_component:snake _dynamic_assets>](&mut dynamic_assets);
            }

            #[derive(Component, Debug, Default, serde::Deserialize, serde::Serialize, Copy, Clone)]
            #[require(common::common_components::SparedFromHotReloading, common::common_components::AssetScoped, common::common_id_components::Prefix::trunc(concat!("Egui", stringify!($main_component), "Holder")), bevy_replicon::shared::replication::Replicated, Visibility, Transform)]
            pub struct [<Egui $abbreviation sHolder>];

            #[derive(bevy::prelude::Resource, Clone, )]
            pub struct [<$main_component EntityMap>](pub common::common_types::HashIdToEntityMap);

            impl Default for [<$main_component EntityMap>] { fn default() -> Self { Self(Default::default()) } }

            $crate::__entity_map_define_ref_struct!($abbreviation $(, $ref_reflect)?);
            #[derive(Component, std::fmt::Debug, Clone,
                //Reflect,
            )]
            pub struct [<$abbreviation StrIdRef>](pub common::common_components::StrId);
            impl [<$abbreviation StrIdRef>] {
                pub fn new<I: Into<common::common_components::StrId>>(id: I) -> Self {
                    Self(id.into())
                }
            }

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
            pub fn [<add_ $main_component:snake _templs_to_egui_holder>](
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

                let iter = query.iter();
                let mut child_ofs = Vec::with_capacity(iter.size_hint().1.unwrap_or(iter.size_hint().0));
                for templ_ent in iter {
                    child_ofs.push((templ_ent, ChildOf(holder_id)));
                }
                cmd.try_insert_batch(child_ofs);
            }
            pub fn [<convert_ $abbreviation:snake _strid_ref_to_ent_ref>](
                mut cmd: Commands,
                query: Query<(Entity, &[<$abbreviation StrIdRef>]), (Changed<[<$abbreviation StrIdRef>]>, )>,
                emap: Option<Res<[<$main_component EntityMap>]>>,
            ) {
                if query.is_empty() {
                    return;
                }
                let Some(emap) = emap else {
                    error_once!(target: $target, "{} EntityMap does not exist, inject {} into app", stringify!($main_component), stringify!([<plugin_ $main_component:snake>]));
                    return;
                };
                let iter = query.iter();
                let mut refs = Vec::with_capacity(iter.size_hint().1.unwrap_or(iter.size_hint().0));
                for (customer_ent, str_id_ref) in iter {
                    let Ok(bit_entity) = emap.0.get_cloned(&str_id_ref.0) else {
                        error_once!(target: $target, "{} StrIdRef '{}' could not be resolved to entity in {}", stringify!($abbreviation), str_id_ref.0, stringify!($main_component EntityMap));
                        continue;
                    };
                    refs.push((customer_ent, [<$abbreviation Ref>](bit_entity)));
                    cmd.entity(customer_ent).try_remove::<[<$abbreviation StrIdRef>]>();
                }
                cmd.try_insert_batch(refs);
            }

            pub fn [<plugin_ $main_component:snake>](app: &mut App) {
                use bevy_replicon::prelude::AppRuleExt;
                use bevy_asset_loader::prelude::*;
                $crate::__entity_map_register_reflect_type!(app, $($ref_reflect,)? [<$abbreviation Ref>]);
                $(
                    common::common_resources::register_seri_auto_routing_rule($dynamic_key, $ron_suffix);
                    common::def_db::register_expected_def_type(stringify!($seri_type));
                )*
                app
                    .init_resource::<bevy_asset_loader::dynamic_asset::DynamicAssets>()
                    .replicate::<$main_component>()
                    .replicate::<[<$abbreviation Ref>]>()
                    .replicate::<[<Egui $abbreviation sHolder>]>()

                    .configure_loading_state(
                        bevy_asset_loader::prelude::LoadingStateConfig::new(common::common_states::AssetLoading::LoadingAssetsIntoHandles)
                            $(.load_collection::<[<$seri_type sHandles>]>() )*
                    )
                    .add_systems(OnEnter(common::common_states::AssetLoading::LoadingAssetsIntoHandles), [<register_ $main_component:snake _dynamic_assets>])
                    .add_plugins((
                        $(
                            bevy_common_assets::ron::RonAssetPlugin::<$seri_type>::new(&[$ron_suffix]),
                        )*
                    ))
                    ;
                [<do_register_ $main_component:snake _dynamic_assets>](
                    &mut app.world_mut().resource_mut::<bevy_asset_loader::dynamic_asset::DynamicAssets>(),
                );
                [<plugin_common_ $main_component:snake>](app);
            }

            fn [<plugin_common_ $main_component:snake>](app: &mut App) {
                app
                    .init_resource::<[<$main_component EntityMap>]>()
                    //.register_type::<[<Egui $abbreviation sHolder>]>()
                    .add_systems(Update, ([<map_ $main_component:snake _id_to_entity>],
                         [<add_ $main_component:snake _templs_to_egui_holder>]
                            .run_if(bevy::time::common_conditions::on_timer(core::time::Duration::from_secs(1)))
                            .run_if(in_state(bevy_replicon::prelude::ClientState::Disconnected)),
                    ))
                    .add_observer([<remove_ $main_component:snake _from_ $main_component:snake _on_despawn>])
                    ;
            }
        }
    };
}

#[macro_export]
macro_rules! define_entity_map_systems_no_replicate {
    (
        main_component: $main_component:ident,
        with_filters: $with_filters:ty,
        abbreviation: $abbreviation: ident,
        target: $target:expr,
        entity_prefix: $entity_prefix:expr,
        despawn_trigger: $despawn_trigger:ty,
        id_type: $id_type:ty
        $(, ref_reflect: $ref_reflect:ident)?
        $(,)?
    ) => {
        paste::paste! {
            #[derive(Component, Debug, Default, serde::Deserialize, serde::Serialize, Copy, Clone)]
            #[require(common::common_components::SparedFromHotReloading, common::common_components::AssetScoped, common::common_id_components::Prefix::trunc(concat!("Egui", stringify!($main_component), "Holder")), bevy_replicon::shared::replication::Replicated, Visibility, Transform)]
            pub struct [<Egui $abbreviation sHolder>];

            #[derive(bevy::prelude::Resource, Clone, )]
            pub struct [<$main_component EntityMap>](pub common::common_types::HashIdToEntityMap);

            impl Default for [<$main_component EntityMap>] { fn default() -> Self { Self(Default::default()) } }

            $crate::__entity_map_define_ref_struct!($abbreviation $(, $ref_reflect)?);
            #[derive(Component, std::fmt::Debug, Clone, )]
            pub struct [<$abbreviation StrIdRef>](pub common::common_components::StrId);
            impl [<$abbreviation StrIdRef>] {
                pub fn new<I: Into<common::common_components::StrId>>(id: I) -> Self {
                    Self(id.into())
                }
            }

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
            pub fn [<add_ $main_component:snake _templs_to_egui_holder>](
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

                let iter = query.iter();
                let mut child_ofs = Vec::with_capacity(iter.size_hint().1.unwrap_or(iter.size_hint().0));
                for templ_ent in iter {
                    child_ofs.push((templ_ent, ChildOf(holder_id)));
                }
                cmd.try_insert_batch(child_ofs);
            }
            pub fn [<convert_ $abbreviation:snake _strid_ref_to_ent_ref>](
                mut cmd: Commands,
                query: Query<(Entity, &[<$abbreviation StrIdRef>]), (Changed<[<$abbreviation StrIdRef>]>, )>,
                emap: Option<Res<[<$main_component EntityMap>]>>,
            ) {
                if query.is_empty() {
                    return;
                }
                let Some(emap) = emap else {
                    error_once!(target: $target, "{} EntityMap does not exist, inject {} into app", stringify!($main_component), stringify!([<plugin_ $main_component:snake>]));
                    return;
                };
                let iter = query.iter();
                let mut refs = Vec::with_capacity(iter.size_hint().1.unwrap_or(iter.size_hint().0));
                for (customer_ent, str_id_ref) in iter {
                    let Ok(bit_entity) = emap.0.get_cloned(&str_id_ref.0) else {
                        error_once!(target: $target, "{} StrIdRef '{}' could not be resolved to entity in {}", stringify!($abbreviation), str_id_ref.0, stringify!($main_component EntityMap));
                        continue;
                    };
                    refs.push((customer_ent, [<$abbreviation Ref>](bit_entity)));
                    cmd.entity(customer_ent).try_remove::<[<$abbreviation StrIdRef>]>();
                }
                cmd.try_insert_batch(refs);
            }

            pub fn [<plugin_ $main_component:snake>](app: &mut App) {
                $crate::__entity_map_register_reflect_type!(app, $($ref_reflect,)? [<$abbreviation Ref>]);

                app
                    .init_resource::<[<$main_component EntityMap>]>()
                    .add_systems(Update, ([<map_ $main_component:snake _id_to_entity>],
                         [<add_ $main_component:snake _templs_to_egui_holder>]
                            .run_if(bevy::time::common_conditions::on_timer(core::time::Duration::from_secs(1)))
                            .run_if(in_state(bevy_replicon::prelude::ClientState::Disconnected)),
                    ))
                    .add_observer([<remove_ $main_component:snake _from_ $main_component:snake _on_despawn>])
                    ;
            }
        }
    };

    (
        main_component: $main_component:ident,
        with_filters: $with_filters:ty,
        abbreviation: $abbreviation: ident,
        target: $target:expr,
        entity_prefix: $entity_prefix:expr,
        despawn_trigger: $despawn_trigger:ty,
        id_type: $id_type:ty,
        assets: [$(($seri_type:ty, $dynamic_key:literal, $ron_suffix:literal)),+]
        $(, ref_reflect: $ref_reflect:ident)?
        $(,)?
    ) => {
        paste::paste! {
            #[allow(unused_imports)]
            use bevy_asset_loader::prelude::AssetCollection;

            $(
                #[derive(bevy_asset_loader::asset_collection::AssetCollection, Resource, Default, )]
                pub struct [<$seri_type sHandles>] {
                    #[asset(key = $dynamic_key)]
                    #[asset(collection(typed))]
                    pub handles: Vec<Handle<$seri_type>>,
                }

                pub fn [<load_ $seri_type:snake _defs>]() -> Vec<$seri_type> {
                    let db = match common::def_db::DefDatabase::<$seri_type>::load_from_assets_dir_with_type(
                        stringify!($seri_type),
                        &[$ron_suffix],
                        |seri| seri.id.as_str(),
                    ) {
                        Ok(db) => db,
                        Err(err) => {
                            error!(
                                target: $target,
                                "Failed loading {} defs: {err:#}",
                                stringify!($seri_type)
                            );
                            return Vec::new();
                        }
                    };
                    if !$target.is_empty() {
                        for ov in db.overrides() {
                            info!(
                                target: $target,
                                "{} def '{}' overridden: '{}' -> '{}'",
                                stringify!($seri_type),
                                ov.id,
                                ov.previous_source.rel_path,
                                ov.replacement_source.rel_path
                            );
                        }
                    }
                    db.into_records().into_iter().map(|r| r.value).collect()
                }
            )+

            fn [<do_register_ $main_component:snake _dynamic_assets>](
                dynamic_assets: &mut bevy_asset_loader::dynamic_asset::DynamicAssets,
            ) {
                $(
                    common::common_resources::register_seri_dynamic_asset_key(dynamic_assets, $dynamic_key);
                )*
            }
            pub fn [<register_ $main_component:snake _dynamic_assets>](
                mut dynamic_assets: ResMut<bevy_asset_loader::dynamic_asset::DynamicAssets>,
            ) {
                [<do_register_ $main_component:snake _dynamic_assets>](&mut dynamic_assets);
            }

            #[derive(Component, Debug, Default, serde::Deserialize, serde::Serialize, Copy, Clone)]
            #[require(common::common_components::SparedFromHotReloading, common::common_components::AssetScoped, common::common_id_components::Prefix::trunc(concat!("Egui", stringify!($main_component), "Holder")), bevy_replicon::shared::replication::Replicated, Visibility, Transform)]
            pub struct [<Egui $abbreviation sHolder>];

            #[derive(bevy::prelude::Resource, Clone, )]
            pub struct [<$main_component EntityMap>](pub common::common_types::HashIdToEntityMap);

            impl Default for [<$main_component EntityMap>] { fn default() -> Self { Self(Default::default()) } }

            $crate::__entity_map_define_ref_struct!($abbreviation $(, $ref_reflect)?);
            #[derive(Component, std::fmt::Debug, Clone,
                //Reflect,
            )]
            pub struct [<$abbreviation StrIdRef>](pub common::common_components::StrId);
            impl [<$abbreviation StrIdRef>] {
                pub fn new<I: Into<common::common_components::StrId>>(id: I) -> Self {
                    Self(id.into())
                }
            }

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
            pub fn [<add_ $main_component:snake _templs_to_egui_holder>](
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

                let iter = query.iter();
                let mut child_ofs = Vec::with_capacity(iter.size_hint().1.unwrap_or(iter.size_hint().0));
                for templ_ent in iter {
                    child_ofs.push((templ_ent, ChildOf(holder_id)));
                }
                cmd.try_insert_batch(child_ofs);
            }
            pub fn [<convert_ $abbreviation:snake _strid_ref_to_ent_ref>](
                mut cmd: Commands,
                query: Query<(Entity, &[<$abbreviation StrIdRef>]), (Changed<[<$abbreviation StrIdRef>]>, )>,
                emap: Option<Res<[<$main_component EntityMap>]>>,
            ) {
                if query.is_empty() {
                    return;
                }
                let Some(emap) = emap else {
                    error_once!(target: $target, "{} EntityMap does not exist, inject {} into app", stringify!($main_component), stringify!([<plugin_ $main_component:snake _no_replicate>]));
                    return;
                };
                let iter = query.iter();
                let mut refs = Vec::with_capacity(iter.size_hint().1.unwrap_or(iter.size_hint().0));
                for (customer_ent, str_id_ref) in iter {
                    let Ok(bit_entity) = emap.0.get_cloned(&str_id_ref.0) else {
                        error_once!(target: $target, "{} StrIdRef '{}' could not be resolved to entity in {}", stringify!($abbreviation), str_id_ref.0, stringify!($main_component EntityMap));
                        continue;
                    };
                    refs.push((customer_ent, [<$abbreviation Ref>](bit_entity)));
                    cmd.entity(customer_ent).try_remove::<[<$abbreviation StrIdRef>]>();
                }
                cmd.try_insert_batch(refs);
            }

            pub fn [<plugin_ $main_component:snake>](app: &mut App) {
                use bevy_asset_loader::prelude::*;
                $crate::__entity_map_register_reflect_type!(app, $($ref_reflect,)? [<$abbreviation Ref>]);
                $(
                    common::common_resources::register_seri_auto_routing_rule($dynamic_key, $ron_suffix);
                    common::def_db::register_expected_def_type(stringify!($seri_type));
                )*

                app
                    .init_resource::<bevy_asset_loader::dynamic_asset::DynamicAssets>()
                    .init_resource::<[<$main_component EntityMap>]>()
                    .add_systems(Update, ([<map_ $main_component:snake _id_to_entity>],
                         [<add_ $main_component:snake _templs_to_egui_holder>]
                            .run_if(bevy::time::common_conditions::on_timer(core::time::Duration::from_secs(1)))
                            .run_if(in_state(bevy_replicon::prelude::ClientState::Disconnected)),
                    ))
                    .add_observer([<remove_ $main_component:snake _from_ $main_component:snake _on_despawn>])
                    .configure_loading_state(
                        bevy_asset_loader::prelude::LoadingStateConfig::new(common::common_states::AssetLoading::LoadingAssetsIntoHandles)
                            $(.load_collection::<[<$seri_type sHandles>]>() )*
                    )
                    .add_systems(OnEnter(common::common_states::AssetLoading::LoadingAssetsIntoHandles), [<register_ $main_component:snake _dynamic_assets>])
                    .add_plugins((
                        $(
                            bevy_common_assets::ron::RonAssetPlugin::<$seri_type>::new(&[$ron_suffix]),
                        )*
                    ))
                    ;
                [<do_register_ $main_component:snake _dynamic_assets>](
                    &mut app.world_mut().resource_mut::<bevy_asset_loader::dynamic_asset::DynamicAssets>(),
                );
            }
        }
    };
}
