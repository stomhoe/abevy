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
        let str_id = StrId::new(id)?;
        Ok(DimensionStrIdRef(str_id))
    }
    pub fn overworld() -> Self {
        DimensionStrIdRef(StrId::new("overworld").unwrap())
    }
}