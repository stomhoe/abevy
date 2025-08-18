use bevy::{ecs::entity::EntityHashSet, platform::collections::HashMap, prelude::*};
use bevy::prelude::*;
use bevy_replicon::prelude::Replicated;
use common::common_components::*;
use serde::{Deserialize, Serialize};


//            .replicate::<MainComponentNameRef>()
#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, Hash, PartialEq, Reflect)]
#[require(Replicated, SessionScoped, AssetScoped, EntityPrefix::new("Dimension") )]
pub struct Dimension;
 
#[derive(Component, Debug, Deserialize, Serialize, Copy, Clone, Hash, PartialEq, Eq, Reflect)]
pub struct DimensionRef(#[entities] pub Entity);



#[derive(Component, Debug, Deserialize, Serialize, Clone, Hash, PartialEq, Eq, Reflect)]
pub struct DimensionStrIdRef(pub StrId);
impl DimensionStrIdRef {
    pub fn new<S: AsRef<str>>(id: S) -> Result<Self, BevyError> {
        let str_id = StrId::new(id, 2)?;
        Ok(DimensionStrIdRef(str_id))
    }
    pub fn overworld() -> Self {
        DimensionStrIdRef(StrId::new("ow", 0).unwrap())
    }
}

#[derive(Component, Debug, Deserialize, Serialize, Clone, Reflect)]
pub struct MultipleDimensionStringRefs(Vec<String>);

impl MultipleDimensionStringRefs {
    pub fn new(strings: Vec<String>) -> Self {
        let filtered = strings.into_iter().filter(|s| !s.is_empty()).collect();
        MultipleDimensionStringRefs(filtered)
    }

    pub fn iter(&self) -> std::slice::Iter<'_, String> {
        self.0.iter()
    }
}


#[derive(Component, Debug, Default, Serialize, Deserialize, Reflect)]
pub struct MultipleDimensionRefs(#[entities] pub EntityHashSet,);