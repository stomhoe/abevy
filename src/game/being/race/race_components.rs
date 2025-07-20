use bevy::platform::collections::HashMap;
#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
use serde::{Deserialize, Serialize};

use crate::game::game_utils::{StrId, WeightedMap};


#[derive(Component, Debug, PartialEq, Eq, Hash, Clone)]
#[require(Replicated)]
pub struct RaceId(String);

impl RaceId {
    pub fn new<S: Into<String>>(id: S) -> Self { Self (id.into()) }
    pub fn id(&self) -> &String {&self.0}
}





#[derive(Component, Debug, Default, Deserialize, Serialize, )]
pub struct SpritesPool(#[entities] pub Vec<Entity>);

//Usar DisplayName para cada grupo

#[derive(Component, Debug, Default, Deserialize, Serialize, )]
pub struct SelectableSprites(#[entities] pub Vec<Entity>);

#[derive(Component, Debug, Default, Deserialize, Serialize, )]
pub struct Demonym(pub String); 
impl Demonym {pub fn new<S: Into<String>>(id: S) -> Self { Self (id.into()) }}

#[derive(Component, Debug, Default, Deserialize, Serialize, )]
pub struct SingularDenomination(pub String); 
impl SingularDenomination {pub fn new<S: Into<String>>(id: S) -> Self { Self (id.into()) }}

#[derive(Component, Debug, Default, Deserialize, Serialize, )]
pub struct PluralDenomination(pub String); 
impl PluralDenomination {pub fn new<S: Into<String>>(id: S) -> Self { Self (id.into()) }}

#[derive(Component, Debug, Deserialize, Serialize, )]
pub struct Sexes(pub WeightedMap<StrId>);//id, weight
impl Sexes {
    pub fn new(map: HashMap<StrId, u32>) -> Self {Sexes(WeightedMap::new(map))}
}
type SexId = StrId;

#[derive(serde::Deserialize, Asset, TypePath, Default)]
pub struct RaceSeri {
    pub id: StrId,
    pub name: String,
    pub name_generator: Option<String>,
    pub icon_path: Option<String>,
    pub body_id: StrId,
    pub description: String,
    pub demonym: String,
    pub singular: String,
    pub plural: Option<String>,
    pub sexes: Option<HashMap<SexId, u32>>,
    pub can_equip_tools: bool,
    pub sprites_pool: Vec<String>,
    pub selectable_sprites: Vec<String>,

}