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


#[derive(Component, Debug,)]
pub struct GameZindex(pub i32);

#[derive(Component, Debug, Clone, Default, Serialize, Deserialize, )]
pub struct DisplayName(pub String);
impl DisplayName {pub fn new(name: impl Into<String>) -> Self {DisplayName(name.into())}}

impl core::fmt::Display for DisplayName {
    #[inline(always)]
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        core::fmt::Display::fmt(&self.0, f)
    }
}