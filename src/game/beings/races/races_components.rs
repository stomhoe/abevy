#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
use serde::{Deserialize, Serialize};


#[derive(Component, Debug, PartialEq, Eq, Hash, Clone)]
#[require(Replicated)]
pub struct Race{id: String,}

impl Race {
    pub fn new(id: String) -> Self {Self { id }}
    pub fn id(&self) -> &str { &self.id }
}

#[derive(Component, Debug)]
pub struct RaceRef(pub Entity);

//^^
//APUNTA A ENTITIES DE LAS CUALES CADA UNA TIENE LO DE ABAJO



#[derive(Component, Debug, Default, Deserialize, Serialize, )]
pub struct PlayerChoosableSprites(#[entities] Vec<Entity>);

//Usar DisplayName para cada grupo





#[derive(Component, Debug, Default, Deserialize, Serialize, )]
pub struct ChoosableSprites(#[entities] Vec<Entity>);




#[derive(serde::Deserialize, Asset, TypePath, Default)]
pub struct RaceDto {
    pub id: String,
    pub name: String,
    pub icon_path: Option<String>,
    pub description: Option<String>,
    pub demonym: Option<String>,
    pub singular_denomination: Option<String>,
    pub plural_denomination: Option<String>,
    pub males_ratio: Option<f32>,
    pub can_equip_tools: bool,
    pub used_sprite_categories: Vec<String>, //head, body
    pub chooseable_sprite_types: Vec<String>,

}