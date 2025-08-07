#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_asset_loader::prelude::*;
use bevy::platform::collections::HashMap;

#[derive(Resource, Debug, Default )]
pub struct DimensionEntityMap(pub HashMap<String, Entity>);
