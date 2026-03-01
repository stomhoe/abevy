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
    (with_serde, ($($req:path),* $(,)?), ) => {};
    (no_serde, ($($req:path),* $(,)?), ) => {};

    (with_serde, ($($req:path),* $(,)?), $name:ident $(($vis:vis $ty:ty))?, $($rest:tt)*) => {
        $crate::__define_marker_component_with_serde!($name $(($vis $ty))?, ($($req),*));
        $crate::__define_marker_components_impl!(with_serde, ($($req),*), $($rest)*);
    };
    (no_serde, ($($req:path),* $(,)?), $name:ident $(($vis:vis $ty:ty))?, $($rest:tt)*) => {
        $crate::__define_marker_component_no_serde!($name $(($vis $ty))?, ($($req),*));
        $crate::__define_marker_components_impl!(no_serde, ($($req),*), $($rest)*);
    };

    (with_serde, ($($req:path),* $(,)?), $name:ident $(($vis:vis $ty:ty))?) => {
        $crate::__define_marker_component_with_serde!($name $(($vis $ty))?, ($($req),*));
    };
    (no_serde, ($($req:path),* $(,)?), $name:ident $(($vis:vis $ty:ty))?) => {
        $crate::__define_marker_component_no_serde!($name $(($vis $ty))?, ($($req),*));
    };
}

#[macro_export]
macro_rules! define_marker_components {
    (
        $serde_mode:ident,
        require($($req:path),* $(,)?),
        $($defs:tt)+
    ) => {
        $crate::__define_marker_components_impl!($serde_mode, ($($req),*), $($defs)+);
    };
}

#[macro_export]
macro_rules! define_marker_Components {
    ($($tt:tt)*) => {
        $crate::define_marker_components!($($tt)*);
    };
}
