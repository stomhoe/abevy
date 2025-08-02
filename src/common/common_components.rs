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


#[derive(Component, Debug, Clone, Default, Serialize, Deserialize, )]
pub struct DisplayName(pub String);
impl DisplayName {pub fn new(name: impl Into<String>) -> Self {DisplayName(name.into())}}

impl core::fmt::Display for DisplayName {
    #[inline(always)]
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        core::fmt::Display::fmt(&self.0, f)
    }
}