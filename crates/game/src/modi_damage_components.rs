
use bevy::platform::collections::HashMap;
#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;



#[derive(Component, Debug, Default, Deserialize, Serialize, Clone,  PartialEq, Reflect )]
//para flat/scaled damage reduction o increase. combinar con OperationType para flat damage reduction o scaled
pub struct AffectsIncDamage(pub HashMap<String, f32>);//f32: multiplier de efectividad
