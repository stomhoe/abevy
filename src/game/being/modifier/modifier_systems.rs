#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
#[allow(unused_imports)] use bevy_asset_loader::prelude::*;
use crate::game::{being::modifier::modifier_components::*, game_components::TimeBasedMultiplier};


// ----------------------> NO OLVIDARSE DE AGREGARLO AL Plugin DEL MÓDULO <-----------------------------
//                                                       ^^^^
#[allow(unused_parens)]
pub fn update_current_potency(mut cmd: Commands, 
    affected: Query<&AppliedModifiers>,
    mut antidotes: Query<(&Modifier, &BasePotency, Option<&TimeBasedMultiplier>),()>,
    mut query: Query<(&Modifier, &BasePotency, Option<&TimeBasedMultiplier>),()>
) {
    for ent in affected.iter() {
        
    }


}




