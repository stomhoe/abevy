#[macro_export]
macro_rules! __entity_map_define_ref_struct {
    ($abbreviation:ident $(,)?) => {
        paste::paste! {
            #[derive(Component, std::fmt::Debug, serde::Deserialize, serde::Serialize, Copy, Clone, std::hash::Hash, PartialEq, Eq, )]
            pub struct [<$abbreviation Ref>](pub common::HashId);
            impl [<$abbreviation Ref>] {
                pub fn is_placeholder(&self) -> bool {
                    self.0 == common::HashId::default()
                }
            }
        }
    };
    ($abbreviation:ident, no_reflect) => {
        $crate::__entity_map_define_ref_struct!($abbreviation);
    };
    ($abbreviation:ident, reflect_ref) => {
        paste::paste! {
            #[derive(Component, std::fmt::Debug, serde::Deserialize, serde::Serialize, Copy, Clone, std::hash::Hash, PartialEq, Eq, Reflect, )]
            pub struct [<$abbreviation Ref>](pub common::HashId);
            impl [<$abbreviation Ref>] {
                pub fn is_placeholder(&self) -> bool {
                    self.0 == common::HashId::default()
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
macro_rules! __entity_map_emit_shared_items {
    (
        main_component: $main_component:ident,
        with_filters: $with_filters:ty,
        abbreviation: $abbreviation:ident,
        target: $target:expr,
        entity_prefix: $entity_prefix:expr,
        despawn_trigger: $despawn_trigger:ty,
        id_type: $id_type:ty,
        holder_visibility: $holder_visibility:path
        $(, templ_enti_ref_sync: ($($templ_enti_ref_sync:ty),* $(,)?))?
        $(, ref_reflect: $ref_reflect:ident)?
        $(,)?
    ) => {
        paste::paste! {
            #[derive(Component, Debug, Default, serde::Deserialize, serde::Serialize, Clone, )]
            #[require(common::AssetScoped, common::EguiHolder, common::Prefix::trunc(concat!("Egui", stringify!($main_component), "Holder")), bevy_replicon::shared::replication::Replicated, $holder_visibility, Transform, Name)]
            pub struct [<Egui $abbreviation sHolder>];

            #[derive(bevy::prelude::Resource, std::fmt::Debug, Clone)]
            pub struct [<$main_component EntityMap>](pub common::HashIdToEntityMap);
            impl Default for [<$main_component EntityMap>] {
                fn default() -> Self {
                    Self(Default::default())
                }
            }

            $crate::__entity_map_define_ref_struct!($abbreviation $(, $ref_reflect)?);

            #[derive(Component, std::fmt::Debug, Clone, PartialEq, Eq)]
            pub struct [<$abbreviation StrIdRef>](pub common::StrId);
            impl [<$abbreviation StrIdRef>] {
                pub fn new<I: Into<common::StrId>>(id: I) -> Self {
                    Self(id.into())
                }
                pub fn asd<S: AsRef<str>>(id: S) -> Self {
                    Self(common::StrId::trunc(id.as_ref()))
                }
            }

            pub fn [<map_ $main_component:snake _id_to_entity>](
                mut cmd: Commands,
                map: Option<ResMut<[<$main_component EntityMap>]>>,
                client_state: Res<State<bevy_replicon::prelude::ClientState>>,
                query: Query<(Entity, Option<&common::Prefix>, &$id_type, Has<common::RemoveReplicatedAfterClone>), (Changed<$id_type>, With<$main_component>, $with_filters)>,
            ) {
                let am_i_client = *client_state.get() == bevy_replicon::prelude::ClientState::Connected;
                if let Some(mut map) = map {
                    for (entity, prefix, id, remove_after_clone) in query.iter() {
                        if am_i_client && remove_after_clone {
                            continue;
                        }
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

            #[allow(unused_parens, )]
            pub fn [<add_ $main_component:snake _templs_to_egui_holder>](
                mut cmd: Commands,
                holder_query: Query<(Entity, Option<&Children>, ), (With<[<Egui $abbreviation sHolder>]>, ),>,
                query: Query<(Entity, ), (With<$main_component>, $with_filters, Without<ChildOf>, ),>,
            ) {
                let holders: Vec<_> = holder_query.iter().collect();
                let holder_id = if holders.is_empty() {
                    cmd.spawn(([<Egui $abbreviation sHolder>], )).id()
                } else {
                    let (max_holder, _) = holders
                        .iter()
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
                for (templ_ent,) in iter {
                    child_ofs.push((templ_ent, ChildOf(holder_id)));
                }
                cmd.try_insert_batch(child_ofs);
            }

            pub fn [<convert_ $abbreviation:snake _strid_ref_to_ent_ref>](
                mut cmd: Commands,
                query: Query<(Entity, &[<$abbreviation StrIdRef>], ), (Changed<[<$abbreviation StrIdRef>]>, ),>,
            ) {
                if query.is_empty() {
                    return;
                }
                let iter = query.iter();
                let mut refs = Vec::with_capacity(iter.size_hint().1.unwrap_or(iter.size_hint().0));
                for (customer_ent, str_id_ref) in iter {
                    refs.push((customer_ent, [<$abbreviation Ref>](common::HashId::from(str_id_ref.0.as_str()))));
                    cmd.entity(customer_ent).try_remove::<[<$abbreviation StrIdRef>]>();
                }
                cmd.try_insert_batch(refs);
            }

            #[allow(unused_parens, )]
            pub fn [<resolve_ $abbreviation:snake _templ_enti_ref_from_hash_id>](
                mut cmd: Commands,
                emap: Option<Res<[<$main_component EntityMap>]>>,
                query: Query<
                    (Entity, &common::TemplHashIdRef, Option<&common::TemplEntiRef>, ),
                    (Or<(Changed<common::TemplHashIdRef>, Added<$main_component>)>, ),
                >,
            ) {
                let Some(emap) = emap else {
                    return;
                };
                for (entity, templ_hash_ref, templ_ref) in query.iter() {
                    let Ok(templ_ent) = emap.0.get_cloned(templ_hash_ref.0) else {
                        continue;
                    };
                    if templ_ref.map(|templ_ref| templ_ref.0 == templ_ent).unwrap_or(false) {
                        continue;
                    }
                    cmd.entity(entity).insert(common::TemplEntiRef(templ_ent));
                }
            }

            $(
                $crate::__entity_map_emit_instance_templ_enti_ref_sync_system!(
                    main_component: $main_component,
                    abbreviation: $abbreviation,
                    entity_map: [<$main_component EntityMap>],
                    target: $target,
                    templ_enti_ref_sync_filters: ($($templ_enti_ref_sync),*),
                );
            )?
        }
    };
}

#[macro_export]
macro_rules! __entity_map_emit_instance_templ_enti_ref_sync_system {
    (
        main_component: $main_component:ident,
        abbreviation: $abbreviation:ident,
        entity_map: $entity_map:ty,
        target: $target:expr,
        templ_enti_ref_sync_filters: (),
    ) => {
        paste::paste! {
            #[allow(unused_parens, )]
            pub fn [<sync_ $abbreviation:snake _instance_templ_enti_ref_from_map>](
                mut cmd: Commands,
                emap: Option<Res<$entity_map>>,
                query: Query<
                    (Entity, &[<$abbreviation Ref>], Option<&common::TemplEntiRef>, ),
                    (
                        Or<(Changed<[<$abbreviation Ref>]>, Added<$main_component>)>,
                        With<$main_component>,
                        Without<game_common::game_common_components::Templ>,
                    ),
                >,
            ) {
                let Some(emap) = emap else {
                    return;
                };
                for (entity, ref_component, templ_ref) in query.iter() {
                    let Ok(templ_ent) = emap.0.get_cloned(ref_component.0) else {
                        continue;
                    };
                    if templ_ref.map(|templ_ref| templ_ref.0 == templ_ent).unwrap_or(false) {
                        continue;
                    }
                    cmd.entity(entity).insert(common::TemplEntiRef(templ_ent));
                    if !$target.is_empty() {
                        trace!(
                            target: $target,
                            "Synced {} entity {:?} to template entity {:?} via {:?}",
                            stringify!($main_component),
                            entity,
                            templ_ent,
                            ref_component.0,
                        );
                    }
                }
            }
        }
    };
    (
        main_component: $main_component:ident,
        abbreviation: $abbreviation:ident,
        entity_map: $entity_map:ty,
        target: $target:expr,
        templ_enti_ref_sync_filters: ($first_filter:ty $(, $rest_filters:ty)*),
    ) => {
        paste::paste! {
            #[allow(unused_parens, )]
            pub fn [<sync_ $abbreviation:snake _instance_templ_enti_ref_from_map>](
                mut cmd: Commands,
                emap: Option<Res<$entity_map>>,
                mut query: Query<
                    (Entity, &[<$abbreviation Ref>], Option<&mut common::TemplEntiRef>, ),
                    (
                        Or<(Changed<[<$abbreviation Ref>]>, Added<$main_component>)>,
                        With<$main_component>,
                        Without<game_common::game_common_components::Templ>,
                        $first_filter,
                        $($rest_filters, )*
                    ),
                >,
            ) {
                let Some(emap) = emap else {
                    return;
                };
                let mut to_insert = Vec::new();
                for (entity, ref_component, mut templ_ref) in query.iter_mut() {
                    let Ok(templ_ent) = emap.0.get_cloned(ref_component.0) else {
                        continue;
                    };
                    if let Some(mut templ_ref) = templ_ref {
                        if templ_ref.0 == templ_ent {
                            continue;
                        }
                        templ_ref.0 = templ_ent;
                    } else {
                        to_insert.push((entity, common::TemplEntiRef(templ_ent)));
                    }
                }
                cmd.try_insert_batch(to_insert);
            }
        }
    };
}

#[macro_export]
macro_rules! __entity_map_emit_asset_support {
    (
        main_component: $main_component:ident,
        target: $target:expr,
        assets: [$(($seri_type:ty, $dynamic_key:literal, $ron_suffix:literal)),+]
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
        }
    };
}

#[macro_export]
macro_rules! define_entity_map_systems {
    (
        main_component: $main_component:ident,
        with_filters: $with_filters:ty,
        abbreviation: $abbreviation: ident,
        target: $target:expr,
        entity_prefix: $entity_prefix:expr,
        despawn_trigger: $despawn_trigger:ty,
        id_type: $id_type:ty
        $(, templ_enti_ref_sync: ($($templ_enti_ref_sync:ty),* $(,)?))?
        $(, ref_reflect: $ref_reflect:ident)?
        $(,)?
    ) => {
        $crate::__entity_map_emit_shared_items!(
            main_component: $main_component,
            with_filters: $with_filters,
            abbreviation: $abbreviation,
            target: $target,
            entity_prefix: $entity_prefix,
            despawn_trigger: $despawn_trigger,
            id_type: $id_type,
            holder_visibility: Visibility::Hidden
            $(, templ_enti_ref_sync: ($($templ_enti_ref_sync),*))?
            $(, ref_reflect: $ref_reflect)?
        );

        paste::paste! {
            pub fn [<plugin_ $main_component:snake>](app: &mut App) {
                use bevy_replicon::prelude::AppRuleExt;
                app
                    .replicate::<$main_component>()
                    .replicate::<[<Egui $abbreviation sHolder>]>()
                    .replicate::<[<$abbreviation Ref>]>()
                    .replicate_filtered_as::<Visibility, common::VisibilityGameState, (With<[<Egui $abbreviation sHolder>]>,)>();
                [<plugin_common_ $main_component:snake>](app);
            }

            pub fn [<plugin_ $main_component:snake _no_replicate>](app: &mut App) {
                [<plugin_common_ $main_component:snake>](app);
            }

            fn [<plugin_common_ $main_component:snake>](app: &mut App) {
                $crate::__entity_map_register_reflect_type!(app, $($ref_reflect,)? [<$abbreviation Ref>]);
                app
                    .init_resource::<[<$main_component EntityMap>]>()
                    .add_systems(Update, (
                        [<map_ $main_component:snake _id_to_entity>],
                        [<resolve_ $abbreviation:snake _templ_enti_ref_from_hash_id>].after([<map_ $main_component:snake _id_to_entity>]),
                        [<add_ $main_component:snake _templs_to_egui_holder>].run_if(bevy::time::common_conditions::on_timer(core::time::Duration::from_secs(1))),
                    ))
                    .add_observer([<remove_ $main_component:snake _from_ $main_component:snake _on_despawn>]);
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
        $(, templ_enti_ref_sync: ($($templ_enti_ref_sync:ty),* $(,)?))?
        $(, ref_reflect: $ref_reflect:ident)?
        $(,)?
    ) => {
        $crate::__entity_map_emit_asset_support!(
            main_component: $main_component,
            target: $target,
            assets: [$(($seri_type, $dynamic_key, $ron_suffix)),+]
        );
        $crate::__entity_map_emit_shared_items!(
            main_component: $main_component,
            with_filters: $with_filters,
            abbreviation: $abbreviation,
            target: $target,
            entity_prefix: $entity_prefix,
            despawn_trigger: $despawn_trigger,
            id_type: $id_type,
            holder_visibility: Visibility
            $(, templ_enti_ref_sync: ($($templ_enti_ref_sync),*))?
            $(, ref_reflect: $ref_reflect)?
        );

        paste::paste! {
            pub fn [<plugin_ $main_component:snake>](app: &mut App) {
                use bevy_asset_loader::prelude::*;
                use bevy_replicon::prelude::AppRuleExt;

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
                    ));
                [<do_register_ $main_component:snake _dynamic_assets>](
                    &mut app.world_mut().resource_mut::<bevy_asset_loader::dynamic_asset::DynamicAssets>(),
                );
                [<plugin_common_ $main_component:snake>](app);
            }

            fn [<plugin_common_ $main_component:snake>](app: &mut App) {
                app
                    .init_resource::<[<$main_component EntityMap>]>()
                    .add_systems(Update, (
                        [<map_ $main_component:snake _id_to_entity>],
                        [<add_ $main_component:snake _templs_to_egui_holder>]
                            .run_if(bevy::time::common_conditions::on_timer(core::time::Duration::from_secs(1)))
                            .run_if(in_state(bevy_replicon::prelude::ClientState::Disconnected)),
                    ))
                    .add_observer([<remove_ $main_component:snake _from_ $main_component:snake _on_despawn>]);
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
        $(, templ_enti_ref_sync: ($($templ_enti_ref_sync:ty),* $(,)?))?
        $(, ref_reflect: $ref_reflect:ident)?
        $(,)?
    ) => {
        $crate::__entity_map_emit_shared_items!(
            main_component: $main_component,
            with_filters: $with_filters,
            abbreviation: $abbreviation,
            target: $target,
            entity_prefix: $entity_prefix,
            despawn_trigger: $despawn_trigger,
            id_type: $id_type,
            holder_visibility: Visibility
            $(, templ_enti_ref_sync: ($($templ_enti_ref_sync),*))?
            $(, ref_reflect: $ref_reflect)?
        );

        paste::paste! {
            pub fn [<plugin_ $main_component:snake>](app: &mut App) {
                $crate::__entity_map_register_reflect_type!(app, $($ref_reflect,)? [<$abbreviation Ref>]);
                app
                    .init_resource::<[<$main_component EntityMap>]>()
                    .add_systems(Update, (
                        [<map_ $main_component:snake _id_to_entity>],
                        [<add_ $main_component:snake _templs_to_egui_holder>]
                            .run_if(bevy::time::common_conditions::on_timer(core::time::Duration::from_secs(1)))
                            .run_if(in_state(bevy_replicon::prelude::ClientState::Disconnected)),
                    ))
                    .add_observer([<remove_ $main_component:snake _from_ $main_component:snake _on_despawn>]);
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
        $(, templ_enti_ref_sync: ($($templ_enti_ref_sync:ty),* $(,)?))?
        $(, ref_reflect: $ref_reflect:ident)?
        $(,)?
    ) => {
        $crate::__entity_map_emit_asset_support!(
            main_component: $main_component,
            target: $target,
            assets: [$(($seri_type, $dynamic_key, $ron_suffix)),+]
        );
        $crate::__entity_map_emit_shared_items!(
            main_component: $main_component,
            with_filters: $with_filters,
            abbreviation: $abbreviation,
            target: $target,
            entity_prefix: $entity_prefix,
            despawn_trigger: $despawn_trigger,
            id_type: $id_type,
            holder_visibility: Visibility
            $(, templ_enti_ref_sync: ($($templ_enti_ref_sync),*))?
            $(, ref_reflect: $ref_reflect)?
        );

        paste::paste! {
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
                    .add_systems(Update, (
                        [<map_ $main_component:snake _id_to_entity>],
                        [<resolve_ $abbreviation:snake _templ_enti_ref_from_hash_id>].after([<map_ $main_component:snake _id_to_entity>]),
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
                    ));
                [<do_register_ $main_component:snake _dynamic_assets>](
                    &mut app.world_mut().resource_mut::<bevy_asset_loader::dynamic_asset::DynamicAssets>(),
                );
            }
        }
    };
}
