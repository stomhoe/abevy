use std::fmt::Display;

use bevy::prelude::*;
#[allow(unused_imports)] 
use serde::{Deserialize, Serialize};


#[derive(Clone, PartialEq, Eq, Hash, )]
pub struct FixedStr<const N: usize>([u8; N]);

impl<const N: usize> FixedStr<N> {
    pub fn new<S: AsRef<str>>(s: S) -> Self {
        let bytes = s.as_ref().as_bytes();
        let mut arr = [0u8; N];
        let len = bytes.len().min(N);
        arr[..len].copy_from_slice(&bytes[..len]);
        Self(arr)
    }
    pub fn new_with_result<S: AsRef<str>>(s: S) -> Result<Self, BevyError> {
        if s.as_ref().len() > N {
            return Err(BevyError::from(format!(
                "String too long for FixedStr<{}>: '{}'",
                N, s.as_ref()
            )));
        }
        Ok(Self::new(s))
    }
    pub fn is_empty(&self) -> bool { self.0.iter().all(|&b| b == 0) }
    pub fn as_str(&self) -> &str {
        let nul_pos = self.0.iter().position(|&b| b == 0).unwrap_or(N);
        std::str::from_utf8(&self.0[..nul_pos]).unwrap_or("")
    }
}

impl<const N: usize> Default for FixedStr<N> {fn default() -> Self { Self([0u8; N]) } }
impl<const N: usize> std::fmt::Display for FixedStr<N> { #[inline(always)] fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result { std::fmt::Display::fmt(self.as_str(), f) } }
impl<const N: usize> std::fmt::Debug for FixedStr<N> { #[inline(always)] fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result { write!(f, "{}", self.as_str()) } }
impl<const N: usize> serde::Serialize for FixedStr<N> { fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: serde::Serializer, { serializer.serialize_str(self.as_str()) } }
impl<'de, const N: usize> serde::Deserialize<'de> for FixedStr<N> { fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: serde::Deserializer<'de>, { let s = <&str>::deserialize(deserializer)?; Ok(FixedStr::new(s)) } }
impl<const N: usize> From<&str> for FixedStr<N> { fn from(s: &str) -> Self { FixedStr::new(s) } }
impl<const N: usize> From<String> for FixedStr<N> { fn from(s: String) -> Self { FixedStr::new(s) } }
impl<const N: usize> AsRef<str> for FixedStr<N> { fn as_ref(&self) -> &str { self.as_str() } }

#[derive(Component, Default, Deserialize, Serialize, Clone, )]
pub struct EntityPrefix(pub FixedStr<20>);

impl EntityPrefix {
    pub fn new<S: AsRef<str>>(id: S) -> Self { Self(FixedStr::new(id)) }
    pub fn as_str(&self) -> &str { self.0.as_str() }
}
impl core::fmt::Debug for EntityPrefix {
    #[inline(always)]
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "EntityPrefix({})", self.as_str())
    }
}
impl core::fmt::Display for EntityPrefix {
    #[inline(always)]
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        core::fmt::Display::fmt(self.as_str(), f)
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

impl DisplayName {
    pub fn new(name: impl Into<String>) -> Self {
        DisplayName(name.into())
    }
}

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
pub struct StrId(FixedStr<32>);
impl StrId {
    pub fn new<S: Into<String>>(id: S) -> Result<Self, BevyError> {
        let s = id.into();
        if s.len() >= 3 {
            FixedStr::new_with_result(s).map(Self)
        } else {
            Err(BevyError::from(format!("StrId {} must be at least 3 characters long", s)))
        }
    }
    pub fn as_str(&self) -> &str { &self.0.as_str() }
    pub fn is_empty(&self) -> bool { self.0.is_empty() }
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

impl AsRef<str> for StrId {fn as_ref(&self) -> &str {&self.0.as_str() }}

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