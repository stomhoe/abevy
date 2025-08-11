
use bevy::platform::collections::HashMap;
#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
use serde::{Deserialize, Serialize};
use crate::game::modifier::{
//    module_name_resources::*,
//    module_name_constants::*,
//    module_name_layout::*,
//    module_name_events::*,
};

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone,  PartialEq, Reflect )]
//para flat/scaled damage reduction o increase. combinar con OperationType para flat damage reduction o scaled
pub struct AffectsIncDamage(pub HashMap<String, f32>);//f32: multiplier de efectividad