

use bevy::prelude::*;
use indexmap::IndexMap;
#[allow(unused_imports)] 
use serde::{Deserialize, Serialize};
use bevy::platform::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;
use crate::{common_types::*};
use bevy_inspector_egui::{egui, inspector_egui_impls::{InspectorPrimitive}, reflect_inspector::InspectorUi};
use std::fmt::{Debug, Display};


macro_rules! define_fixedstr_id {
    ($ty:ident, $len:expr) => {
        #[derive(Component, Default, Deserialize, Serialize, Clone, Hash, Reflect, PartialEq, Eq, )]
        #[require(Name)]
        pub struct $ty(FixedStr<$len>);
        impl $ty {
            pub fn new<S: AsRef<str>>(id: S) -> Self { Self(FixedStr::<$len>::new(id)) }
            pub fn new_with_result<S: AsRef<str>>(id: S, min: u8) -> Result<Self, BevyError> {
                let s = id.as_ref();
                if s.len() >= min as usize {
                    FixedStr::<$len>::new_with_result(s).map(Self)
                } else {
                    Err(BevyError::from(format!(
                        "{} '{}' must be at least {} characters long",
                        stringify!($ty),
                        s,
                        min
                    )))
                }
            }
            pub fn as_str(&self) -> &str { self.0.as_str() }
            pub fn is_empty(&self) -> bool { self.0.is_empty() }
            /// Compare with a string (flexible equality)
            pub fn eq_str<S: AsRef<str>>(&self, other: S) -> bool {
                self.0.as_str() == other.as_ref()
            }
        }
        impl std::fmt::Debug for $ty {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                if self.0.is_empty() { write!(f, "") } else { write!(f, "Id({})", self.0) }
            }
        }
        impl std::fmt::Display for $ty {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                if self.0.is_empty() { write!(f, "") } else { write!(f, "Id({})", self.0) }
            }
        }
        impl InspectorPrimitive for $ty {
            fn ui(&mut self, ui: &mut egui::Ui, _: &dyn std::any::Any, _: egui::Id, _: InspectorUi<'_, '_>) -> bool {
                let mut s = self.0.as_str().to_string();
                let mut changed = false;
                if ui.text_edit_singleline(&mut s).changed() {
                    if let Ok(fixed) = FixedStr::<$len>::new_with_result(&s) {
                        self.0 = fixed;
                        changed = true;
                    }
                }
                changed
            }
            fn ui_readonly(&self, ui: &mut egui::Ui, _: &dyn std::any::Any, _: egui::Id, _: InspectorUi<'_, '_>) {
                ui.label(self.0.as_str());
            }
        }
        impl AsRef<str> for $ty { fn as_ref(&self) -> &str { self.0.as_str() } }
        /// Allow comparison with &str using PartialEq
        impl PartialEq<&str> for $ty {
            fn eq(&self, other: &&str) -> bool {
                self.0.as_str() == *other
            }
        }
    };
}
define_fixedstr_id!(StrId20B, 20);
define_fixedstr_id!(StrId, 32);


define_fixedstr_id!(EntityPrefix, 20);


#[derive(Component, Default, Deserialize, Serialize, Clone, Hash, PartialEq, Eq, Copy, Reflect, )]
pub struct HashId(u64);
impl HashId {
    pub fn new(id: u64) -> Self {HashId(id)}
    pub fn into_i32(self) -> i32 {self.0 as i32}
}
impl<S: AsRef<str>> From<S> for HashId {
    fn from(id: S) -> Self {
        let s = id.as_ref(); let mut hasher = DefaultHasher::new();
        s.hash(&mut hasher); Self((&hasher).finish())
    }
}

impl Display for HashId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "HId({:05})", self.0 & 0xFFFFF)
    }
}
impl Debug for HashId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "HId({:05})", self.0 & 0xFFFFF)
    }
}


#[derive(Component, Default, Deserialize, Serialize, Clone, Debug, Reflect)]
pub struct HashIdMap<T>(pub HashMap<HashId, T>);
impl<T> HashIdMap<T> {
    pub fn new() -> Self { Self(HashMap::new()) }
    pub fn insert<S: AsRef<str>>(&mut self, key: S, value: T) -> Option<T> { self.0.insert(HashId::from(key), value) }
    pub fn insert_with_id(&mut self, id: HashId, value: T) -> Option<T> { self.0.insert(id, value) }

    pub fn get<S: AsRef<str>>(&self, key: S) -> Option<&T> { self.0.get(&HashId::from(key)) }
    pub fn get_mut<S: AsRef<str>>(&mut self, key: S) -> Option<&mut T> { self.0.get_mut(&HashId::from(key)) }
    pub fn remove<S: AsRef<str>>(&mut self, key: S) -> Option<T> { self.0.remove(&HashId::from(key)) }
    pub fn contains_key<S: AsRef<str>>(&self, key: S) -> bool { self.0.contains_key(&HashId::from(key)) }
    pub fn len(&self) -> usize { self.0.len() }
    pub fn is_empty(&self) -> bool { self.0.is_empty() }
    pub fn iter(&self) -> impl Iterator<Item = (&HashId, &T)> { self.0.iter() }
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&HashId, &mut T)> { self.0.iter_mut() }
}
use delegate::delegate;

#[derive(Component, Default, Deserialize, Serialize, Clone, Debug)]
pub struct HashIdIndexMap<T>(pub IndexMap<HashId, T>);
impl<T> HashIdIndexMap<T> {
    pub fn new() -> Self { Self(IndexMap::new()) }
    pub fn insert<S: AsRef<str>>(&mut self, key: S, value: T) -> Option<T> { self.0.insert(HashId::from(key), value) }
    pub fn get<S: AsRef<str>>(&self, key: S) -> Option<&T> { self.0.get(&HashId::from(key)) }
    pub fn get_mut<S: AsRef<str>>(&mut self, key: S) -> Option<&mut T> { self.0.get_mut(&HashId::from(key)) }
    pub fn first(&self) -> Option<&T> {self.0.values().next()}
    pub fn contains_key<S: AsRef<str>>(&self, key: S) -> bool {self.0.contains_key(&HashId::from(key))}
    delegate! {
        to self.0 {
            pub fn iter(&self) -> impl Iterator<Item = (&HashId, &T)>;
            pub fn iter_mut(&mut self) -> impl Iterator<Item = (&HashId, &mut T)>;
            pub fn get_index(&self, i: usize) -> Option<(&HashId, &T)>;
            pub fn get_index_mut(&mut self, i: usize) -> Option<(&HashId, &mut T)>;
            pub fn values(&self) -> impl Iterator<Item = &T>;
            pub fn values_mut(&mut self) -> impl Iterator<Item = &mut T>;
            pub fn len(&self) -> usize;
            pub fn is_empty(&self) -> bool;
            pub fn keys(&self) -> impl Iterator<Item = &HashId>;
            pub fn clear(&mut self);
        }
    }
}
