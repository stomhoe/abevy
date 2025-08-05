use bevy::platform::collections::HashMap;
#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
use serde::{Deserialize, Serialize};

//use crate::game::being::gen_template::{
//    gen_template_constants::*,
//};


#[derive(serde::Deserialize, Asset, TypePath, Default)]
pub struct GenTemplateSeri {
    pub id: String,
    pub name_generator: String,
    pub race: String,
    pub sprites_weighted: HashMap<String, u32>, // de esto va a haber q crear multiples weightedmap, uno para cada categoria de sprite encontrada


}

