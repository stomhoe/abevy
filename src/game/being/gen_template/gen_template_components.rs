use bevy::platform::collections::HashMap;
#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
use serde::{Deserialize, Serialize};

use crate::game::game_utils::StrId;
//use crate::game::being::gen_template::{
//    gen_template_constants::*,
//};


#[derive(serde::Deserialize, Asset, TypePath, Default)]
pub struct GenTemplateSeri {
    pub id: String,
    pub name_generator: StrId,
    pub race: StrId,
    pub sprites_weighted: HashMap<StrId, u32>, // de esto va a haber q crear multiples weightedmap, uno para cada categoria de sprite encontrada


}

