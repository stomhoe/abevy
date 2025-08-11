#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
#[allow(unused_imports)] use bevy_asset_loader::prelude::*;
use common::states::AppState;
use serde::{Deserialize, Serialize};

use crate::being_components::BodyParts;


#[derive(Component, Debug, Deserialize, Serialize)]
#[relationship(relationship_target = BodyParts)]
#[require(Replicated, StateScoped::<AppState>(AppState::StatefulGameSession),  )]
pub struct BodyPartOf {
    #[relationship] #[entities]
    pub being: Entity,
}


#[derive(Component, Debug, Deserialize, Serialize, Copy, Clone, Hash, PartialEq, Eq)]
pub struct HiddenSpritesOnLossRef(#[entities] pub Entity);


#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, Hash, PartialEq,  )]
pub struct CategoriesOfSpritesToAttachTo(Vec<String>);//EN ORDEN DE PRIORIDAD
//TODO MEJORAR EL TIPO INTERNO, O HACERLO PLURAL


#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, Hash, PartialEq,  Copy)]
pub struct CoverageWeight(u16);






