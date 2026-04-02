#[macro_export]
macro_rules! __define_marker_component_with_serde {
    ($name:ident, ($($req:path),* $(,)?)) => {
        #[derive(bevy::prelude::Component, std::fmt::Debug, Default, serde::Deserialize, serde::Serialize, Clone, )]
        #[require($($req),*)]
        pub struct $name;
    };
    ($name:ident ($vis:vis $ty:ty), ($($req:path),* $(,)?)) => {
        #[derive(bevy::prelude::Component, std::fmt::Debug, Default, serde::Deserialize, serde::Serialize, Clone, Copy, )]
        #[require($($req),*)]
        pub struct $name($vis $ty);
    };
}

#[macro_export]
macro_rules! __define_marker_component_no_serde {
    ($name:ident, ($($req:path),* $(,)?)) => {
        #[derive(bevy::prelude::Component, std::fmt::Debug, Default, Clone, )]
        #[require($($req),*)]
        pub struct $name;
    };
    ($name:ident ($vis:vis $ty:ty), ($($req:path),* $(,)?)) => {
        #[derive(bevy::prelude::Component, std::fmt::Debug, Default, Clone, Copy, )]
        #[require($($req),*)]
        pub struct $name($vis $ty);
    };
}

#[macro_export]
macro_rules! __define_marker_components_impl {
    (repli, ($($req:path),* $(,)?), ) => {};
    (no_repli, ($($req:path),* $(,)?), ) => {};

    (repli, ($($req:path),* $(,)?), serialize $name:ident $(($vis:vis $ty:ty))?, $($rest:tt)*) => {
        $crate::__define_marker_component_with_serde!($name $(($vis $ty))?, ($($req),*));
        $crate::__define_marker_components_impl!(repli, ($($req),*), $($rest)*);
    };
    (no_repli, ($($req:path),* $(,)?), serialize $name:ident $(($vis:vis $ty:ty))?, $($rest:tt)*) => {
        $crate::__define_marker_component_with_serde!($name $(($vis $ty))?, ($($req),*));
        $crate::__define_marker_components_impl!(no_repli, ($($req),*), $($rest)*);
    };

    (repli, ($($req:path),* $(,)?), local $name:ident $(($vis:vis $ty:ty))?, $($rest:tt)*) => {
        $crate::__define_marker_component_no_serde!($name $(($vis $ty))?, ($($req),*));
        $crate::__define_marker_components_impl!(repli, ($($req),*), $($rest)*);
    };
    (no_repli, ($($req:path),* $(,)?), local $name:ident $(($vis:vis $ty:ty))?, $($rest:tt)*) => {
        $crate::__define_marker_component_no_serde!($name $(($vis $ty))?, ($($req),*));
        $crate::__define_marker_components_impl!(no_repli, ($($req),*), $($rest)*);
    };

    (repli, ($($req:path),* $(,)?), $name:ident $(($vis:vis $ty:ty))?, $($rest:tt)*) => {
        $crate::__define_marker_component_with_serde!($name $(($vis $ty))?, ($($req),*));
        $crate::__define_marker_components_impl!(repli, ($($req),*), $($rest)*);
    };
    (no_repli, ($($req:path),* $(,)?), $name:ident $(($vis:vis $ty:ty))?, $($rest:tt)*) => {
        $crate::__define_marker_component_no_serde!($name $(($vis $ty))?, ($($req),*));
        $crate::__define_marker_components_impl!(no_repli, ($($req),*), $($rest)*);
    };

    (repli, ($($req:path),* $(,)?), $name:ident $(($vis:vis $ty:ty))?) => {
        $crate::__define_marker_component_with_serde!($name $(($vis $ty))?, ($($req),*));
    };
    (no_repli, ($($req:path),* $(,)?), $name:ident $(($vis:vis $ty:ty))?) => {
        $crate::__define_marker_component_no_serde!($name $(($vis $ty))?, ($($req),*));
    };

    (repli, ($($req:path),* $(,)?), serialize $name:ident $(($vis:vis $ty:ty))?) => {
        $crate::__define_marker_component_with_serde!($name $(($vis $ty))?, ($($req),*));
    };
    (no_repli, ($($req:path),* $(,)?), serialize $name:ident $(($vis:vis $ty:ty))?) => {
        $crate::__define_marker_component_with_serde!($name $(($vis $ty))?, ($($req),*));
    };
    (repli, ($($req:path),* $(,)?), local $name:ident $(($vis:vis $ty:ty))?) => {
        $crate::__define_marker_component_no_serde!($name $(($vis $ty))?, ($($req),*));
    };
    (no_repli, ($($req:path),* $(,)?), local $name:ident $(($vis:vis $ty:ty))?) => {
        $crate::__define_marker_component_no_serde!($name $(($vis $ty))?, ($($req),*));
    };
}

#[macro_export]
macro_rules! define_marker_components {
    (
        $serde_mode:ident,
        require($($req:path),* $(,)?),
        define_plugin($plugin_fn_name:ident),
        define_copy_from_template_system($copy_fn_name:ident),
        $($defs:tt)+
    ) => {
        $crate::define_marker_components!(
            $serde_mode,
            require($($req),*),
            define_copy_from_template_system($copy_fn_name),
            define_plugin($plugin_fn_name),
            $($defs)+
        );
    };
    (
        $serde_mode:ident,
        require($($req:path),* $(,)?),
        define_copy_from_template_system($copy_fn_name:ident),
        define_plugin($plugin_fn_name:ident),
        $($defs:tt)+
    ) => {
        $crate::__define_marker_components_impl!($serde_mode, ($($req),*), $($defs)+);

        #[allow(unused_parens, )]
        pub fn $copy_fn_name(
            mut cmd: bevy::prelude::Commands,
            modifiers_query: bevy::prelude::Query<
                (bevy::prelude::Entity, &game_common::game_common_components::TemplEntiRef, ),
                (
                    bevy::prelude::With<crate::modifier_components::ModifierTarget>,
                    bevy::prelude::Without<game_common::game_common_components::Templ>,
                    bevy::prelude::Changed<game_common::game_common_components::TemplEntiRef>,
                ),
            >,
            entities_query: bevy::prelude::Query<bevy::ecs::world::EntityRef>,
        ) {
            for (entity, templ_ref, ) in modifiers_query.iter() {
                let Ok(templ_entity_ref) = entities_query.get(templ_ref.0) else { continue; };
                $crate::__insert_markers_from_template!(cmd, entity, templ_entity_ref, $($defs)+);
            }
        }

        pub fn $plugin_fn_name(app: &mut bevy::prelude::App) {
            use bevy_replicon::prelude::*;
            app.add_systems(bevy::prelude::PreUpdate, ($copy_fn_name,));
        $crate::__replicate_marker_components!(app, $serde_mode, $($defs)+);
        }
    };
    (
        $serde_mode:ident,
        require($($req:path),* $(,)?),
        define_copy_from_template_system($copy_fn_name:ident),
        $($defs:tt)+
    ) => {
        $crate::__define_marker_components_impl!($serde_mode, ($($req),*), $($defs)+);

        #[allow(unused_parens, )]
        pub fn $copy_fn_name(
            mut cmd: bevy::prelude::Commands,
            modifiers_query: bevy::prelude::Query<
                (bevy::prelude::Entity, &game_common::game_common_components::TemplEntiRef, ),
                (
                    bevy::prelude::With<crate::modifier_components::ModifierTarget>,
                    bevy::prelude::Without<game_common::game_common_components::Templ>,
                    bevy::prelude::Changed<game_common::game_common_components::TemplEntiRef>,
                ),
            >,
            entities_query: bevy::prelude::Query<bevy::ecs::world::EntityRef>,
        ) {
            for (entity, templ_ref, ) in modifiers_query.iter() {
                let Ok(templ_entity_ref) = entities_query.get(templ_ref.0) else { continue; };
                $crate::__insert_markers_from_template!(cmd, entity, templ_entity_ref, $($defs)+);
            }
        }
    };
    (
        $serde_mode:ident,
        require($($req:path),* $(,)?),
        define_plugin($plugin_fn_name:ident),
        $($defs:tt)+
    ) => {
        $crate::__define_marker_components_impl!($serde_mode, ($($req),*), $($defs)+);

        pub fn $plugin_fn_name(app: &mut bevy::prelude::App) {
            use bevy_replicon::prelude::*;
        $crate::__replicate_marker_components!(app, $serde_mode, $($defs)+);
        }
    };
    (
        $serde_mode:ident,
        require($($req:path),* $(,)?),
        $($defs:tt)+
    ) => {
        $crate::__define_marker_components_impl!($serde_mode, ($($req),*), $($defs)+);
    };
}

#[macro_export]
macro_rules! __insert_markers_from_template {
    ($cmd:ident, $entity:ident, $templ_entity_ref:ident, ) => {};
    ($cmd:ident, $entity:ident, $templ_entity_ref:ident, serialize $name:ident $(($vis:vis $ty:ty))?, $($rest:tt)*) => {
        if $templ_entity_ref.contains::<$name>() {
            $cmd.entity($entity).try_insert_if_new($name::default());
        }
        $crate::__insert_markers_from_template!($cmd, $entity, $templ_entity_ref, $($rest)*);
    };
    ($cmd:ident, $entity:ident, $templ_entity_ref:ident, local $name:ident $(($vis:vis $ty:ty))?, $($rest:tt)*) => {
        if $templ_entity_ref.contains::<$name>() {
            $cmd.entity($entity).try_insert_if_new($name::default());
        }
        $crate::__insert_markers_from_template!($cmd, $entity, $templ_entity_ref, $($rest)*);
    };
    ($cmd:ident, $entity:ident, $templ_entity_ref:ident, $name:ident $(($vis:vis $ty:ty))?, $($rest:tt)*) => {
        if $templ_entity_ref.contains::<$name>() {
            $cmd.entity($entity).try_insert_if_new($name::default());
        }
        $crate::__insert_markers_from_template!($cmd, $entity, $templ_entity_ref, $($rest)*);
    };
    ($cmd:ident, $entity:ident, $templ_entity_ref:ident, serialize $name:ident $(($vis:vis $ty:ty))?) => {
        if $templ_entity_ref.contains::<$name>() {
            $cmd.entity($entity).try_insert_if_new($name::default());
        }
    };
    ($cmd:ident, $entity:ident, $templ_entity_ref:ident, local $name:ident $(($vis:vis $ty:ty))?) => {
        if $templ_entity_ref.contains::<$name>() {
            $cmd.entity($entity).try_insert_if_new($name::default());
        }
    };
    ($cmd:ident, $entity:ident, $templ_entity_ref:ident, $name:ident $(($vis:vis $ty:ty))?) => {
        if $templ_entity_ref.contains::<$name>() {
            $cmd.entity($entity).try_insert_if_new($name::default());
        }
    };
}

#[macro_export]
macro_rules! __replicate_marker_components {
    ($app:ident, $mode:ident, ) => {};
    ($app:ident, repli, serialize $name:ident $(($vis:vis $ty:ty))?, $($rest:tt)*) => {
        $app.replicate_once::<$name>();
        $crate::__replicate_marker_components!($app, repli, $($rest)*);
    };
    ($app:ident, no_repli, serialize $name:ident $(($vis:vis $ty:ty))?, $($rest:tt)*) => {
        $app.replicate_once::<$name>();
        $crate::__replicate_marker_components!($app, no_repli, $($rest)*);
    };
    ($app:ident, $mode:ident, local $name:ident $(($vis:vis $ty:ty))?, $($rest:tt)*) => {
        $crate::__replicate_marker_components!($app, $mode, $($rest)*);
    };
    ($app:ident, repli, $name:ident $(($vis:vis $ty:ty))?, $($rest:tt)*) => {
        $app.replicate_once::<$name>();
        $crate::__replicate_marker_components!($app, repli, $($rest)*);
    };
    ($app:ident, no_repli, $name:ident $(($vis:vis $ty:ty))?, $($rest:tt)*) => {
        $crate::__replicate_marker_components!($app, no_repli, $($rest)*);
    };
    ($app:ident, repli, serialize $name:ident $(($vis:vis $ty:ty))?) => {
        $app.replicate_once::<$name>();
    };
    ($app:ident, no_repli, serialize $name:ident $(($vis:vis $ty:ty))?) => {
        $app.replicate_once::<$name>();
    };
    ($app:ident, repli, local $name:ident $(($vis:vis $ty:ty))?) => {};
    ($app:ident, no_repli, local $name:ident $(($vis:vis $ty:ty))?) => {};
    ($app:ident, repli, $name:ident $(($vis:vis $ty:ty))?) => {
        $app.replicate_once::<$name>();
    };
    ($app:ident, no_repli, $name:ident $(($vis:vis $ty:ty))?) => {};
}

#[macro_export]
macro_rules! define_marker_Components {
    ($($tt:tt)*) => {
        $crate::define_marker_components!($($tt)*);
    };
}
