

use bevy::prelude::*;
use indexmap::IndexMap;
#[allow(unused_imports)]
use serde::{Deserialize, Serialize};
use bevy::platform::collections::HashMap;
use std::hash::{Hash, };
use crate::{common_types::*};
use bevy_inspector_egui::{egui, inspector_egui_impls::{InspectorPrimitive}, reflect_inspector::InspectorUi};
use std::fmt::{Debug, Display};

macro_rules! define_fixedstr_id {
    ($ty:ident, $len:expr) => {
        #[derive(Component, Deserialize, Serialize, Clone, Hash, Reflect, PartialEq, Eq, )]
        #[require(Name)]
        pub struct $ty(FixedStr<$len>);
        impl $ty {
            pub const SIZE: usize = $len;

            pub fn trunc<S: AsRef<str>>(id: S) -> Self {
                Self(FixedStr::<$len>::trunc(id.as_ref().trim()))
            }
            pub fn new_with_result<S: AsRef<str>>(id: S, min_length: u8) -> Result<Self, StringLengthError> {
                FixedStr::<$len>::new_with_result(id.as_ref().trim(), min_length).map(Self)
            }

            /// Custom error for ID creation
            pub fn as_str(&self) -> &str { self.0.as_str() }
            pub fn is_empty(&self) -> bool { self.0.is_empty() }
            /// Compare with a string (flexible equality)
            pub fn eq_str<S: AsRef<str>>(&self, other: S) -> bool {
                self.0.as_str() == other.as_ref().trim()
            }
        }
        impl std::fmt::Debug for $ty {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                if self.0.is_empty() { write!(f, "") } else { write!(f, "Id({})", self.0) }
            }
        }
        impl std::fmt::Display for $ty {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                if self.0.is_empty() { write!(f, "") } else { write!(f, "{}", self.0) }
            }
        }
        impl InspectorPrimitive for $ty {
            fn ui(&mut self, ui: &mut egui::Ui, _: &dyn std::any::Any, _: egui::Id, _: InspectorUi<'_, '_>) -> bool {
                let mut s = self.0.as_str().to_string();
                let mut changed = false;
                if ui.text_edit_singleline(&mut s).changed() {
                    if let Ok(fixed) = FixedStr::<$len>::new_with_result(&s, 0) {
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
        impl From<&str> for $ty {
            fn from(s: &str) -> Self {
                Self(FixedStr::<$len>::trunc(s.trim()))
            }
        }
        impl From<String> for $ty {
            fn from(s: String) -> Self {
                Self(FixedStr::<$len>::trunc(s.trim()))
            }
        }
        impl Default for $ty {
            fn default() -> Self {
                Self(FixedStr::<$len>::default())
            }
        }
    };
}
define_fixedstr_id!(StrId20B, 20);
define_fixedstr_id!(Tag, 32);
define_fixedstr_id!(StrId, 32);


define_fixedstr_id!(Prefix, 32);

#[derive(Component, Debug, Default, Deserialize, Serialize, Copy, Clone, Reflect)]
pub struct AddHashIdFromStrId;

#[derive(Component, Default, Deserialize, Serialize, Clone, Hash, PartialEq, Eq, Copy, Reflect)]
pub struct HashId(u64);
impl HashId {
    pub fn new(id: u64) -> Self {
        Self(id)
    }
    pub fn as_i32(self) -> i32 {
        self.0 as i32
    }
    pub fn as_u64(self) -> u64 {
        self.0
    }
    pub fn merge(&self, other: HashId) -> HashId {
        HashId(self.0.wrapping_add(other.0))
    }
    pub const fn hash(s: &str) -> Self {
        const OFFSET: u64 = 0xcbf29ce484222325;
        const PRIME: u64 = 0x100000001b3;
        let bytes = s.as_bytes();
        let mut hash = OFFSET;
        let mut i = 0;
        while i < bytes.len() {
            hash ^= bytes[i] as u64;
            hash = hash.wrapping_mul(PRIME);
            i += 1;
        }
        Self(hash)
    }
}
impl<S: AsRef<str>> From<S> for HashId {
    fn from(id: S) -> Self {
        Self::hash(id.as_ref())
    }
}
impl Display for HashId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "HId({:05})", self.0)
    }
}
impl Debug for HashId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "HId({:05})", self.0)
    }
}


#[derive(Component, Deserialize, Serialize, Clone, Debug, Reflect)]
pub struct HashIdMap<T>(pub HashMap<HashId, T>);
impl<T: Clone> HashIdMap<T> {
    pub fn new() -> Self {
        Self(HashMap::new())
    }
    pub fn insert<K: Into<HashId>>(&mut self, id: K, value: T) -> Result<(), DuplicateKeyError<T>> {
        let hash_id = id.into();
        if let Some(existing) = self.0.get(&hash_id) {
            return Err(DuplicateKeyError((*existing).clone()));
        }
        self.0.insert(hash_id, value);
        Ok(())
    }
    pub fn with_capacity(capacity: usize) -> Self {
        Self(HashMap::with_capacity(capacity))
    }
    pub fn reserve(&mut self, additional: usize) {
        self.0.reserve(additional);
    }

    pub fn overwrite<K: Into<HashId>>(&mut self, id: K, value: T) -> Option<T> {
        self.0.insert(id.into(), value)
    }

    pub fn remove<K: Into<HashId>>(&mut self, id: K) -> Option<T> {
        let hash_id: HashId = id.into();
        self.0.remove(&hash_id)
    }

    pub fn get<K: Into<HashId>>(&self, id: K) -> Result<&T, ()> {
        let hash_id: HashId = id.into();
        self.0.get(&hash_id).ok_or(())
    }
    pub fn get_cloned<K: Into<HashId>>(&self, id: K) -> Result<T, ()> {
        let hash_id: HashId = id.into();
        self.0.get(&hash_id).cloned().ok_or(())
    }

    pub fn iter(&self) -> impl Iterator<Item = (&HashId, &T)> {
        self.0.iter()
    }
    pub fn clear(&mut self) { self.0.clear(); }
    pub fn is_empty(&self) -> bool { self.0.is_empty() }
    pub fn len(&self) -> usize { self.0.len() }
}
impl<T> Default for HashIdMap<T> {
    fn default() -> Self {
        Self(HashMap::new())
    }
}

#[derive(Debug, Clone)]
pub struct DuplicateKeyError<T>(pub T);
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
