#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
#[allow(unused_imports)] use bevy_asset_loader::prelude::*;
use serde::{Deserialize, Serialize};
use crate::game::dimension::{
//    dimension_resources::*,
//    dimension_constants::*,
//    dimension_layout::*,
//    dimension_events::*,
};

#[derive(Component, Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub struct DimensionRef(pub Entity);


#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, )]
#[require(Replicated)]
pub struct Dimension(String);
impl Dimension {
    pub fn new<S: Into<String>>(id: S) -> Self { Self (id.into()) }
    pub fn id(&self) -> &String {&self.0}
}


