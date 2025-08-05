use bevy::prelude::*;
#[allow(unused_imports)] 
use serde::{Deserialize, Serialize};


#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, )]
pub struct EntityPrefix(String);
impl EntityPrefix {
    pub fn new<S: Into<String>>(id: S) -> Self { Self (id.into()) }
}
impl core::fmt::Display for EntityPrefix {
    #[inline(always)]
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        core::fmt::Display::fmt(&self.0, f)
    }
}


#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, Eq, PartialEq, Hash, Copy)]
pub struct MyZ(pub i32);
impl MyZ {
    pub fn new(z: i32) -> Self { Self(z) } 
    pub fn div_1e9(&self) -> f32 { self.0 as f32 * Self::Z_MULTIPLIER }
    pub const Z_MULTIPLIER: f32 = 1e-16;
}


#[derive(Component, Clone, Default, Serialize, Deserialize, )]
pub struct DisplayName(pub String);
impl DisplayName {pub fn new(name: impl Into<String>) -> Self {DisplayName(name.into())}}

impl core::fmt::Display for DisplayName {
    #[inline(always)]
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        core::fmt::Display::fmt(&self.0, f)
    }
}
impl core::fmt::Debug for DisplayName {
    #[inline(always)]
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        if self.0.is_empty() {
            write!(f, "")
        } else {
            write!(f, "dname:'{}'", self.0)
        }
    }
}

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, Hash, PartialEq, Eq )]
//#[require(Replicated, /*StateScoped::<AppState>, */ )]
pub struct StrId(String);
impl StrId {
    pub fn new<S: Into<String>>(id: S) -> Result<Self, &'static str> {
        let s = id.into();
        if s.len() >= 3 {
            Ok(Self(s))
        } else {
            Err("StrId must be at least 3 characters long")
        }
    }
    pub fn new_take(id: &mut String) -> Result<Self, &'static str> {
        let s: String = std::mem::take(id);
        if s.len() >= 3 {
            Ok(Self(s))
        } else {
            Err("StrId must be at least 3 characters long")
        }
    }
    pub fn as_str(&self) -> &String {&self.0}
    pub fn len(&self) -> usize {self.0.len()}
    pub fn is_empty(&self) -> bool {self.0.is_empty()}
}
impl std::fmt::Display for StrId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.0.is_empty() {
            write!(f, "")
        } else {
            write!(f, "Id({})", self.0)
        }
    }
}
impl From<StrId> for String {
    fn from(str_id: StrId) -> Self {
        str_id.0
    }
}
impl AsRef<str> for StrId {fn as_ref(&self) -> &str {&self.0 }}

#[derive(Component, Default, Deserialize, Serialize, Clone, Hash, PartialEq, Eq, Copy )]
//#[require(Replicated, /*StateScoped::<AppState>, */ )]
pub struct HashId(u64);
impl HashId {}
impl<S: AsRef<str>> From<S> for HashId {
    fn from(id: S) -> Self {
        let s = id.as_ref();
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        use std::hash::{Hash, Hasher};
        s.hash(&mut hasher);
        Self((&hasher).finish())
    }
}

impl std::fmt::Display for HashId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "HashId({:05}...)", self.0 & 0xFFFFF)
    }
}
impl std::fmt::Debug for HashId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "HashId({})", self.0)
    }
}