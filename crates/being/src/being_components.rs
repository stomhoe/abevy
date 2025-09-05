use bevy::{platform::{collections::HashMap}, prelude::*};
use bevy_replicon::prelude::Replicated;

use dimension_shared::DimensionStrIdRef;
use game_common::game_common_components::{FacingDirection, MyZ, YSortOrigin};
use modifier::modifier_components::AppliedModifiers;
use movement::movement_components::InputMoveVector;
use serde::{Deserialize, Serialize};
use common::common_components::*;
use sprite_animation_shared::MoveAnimActive;
use tilemap::chunking_components::ActivatingChunks;
use superstate::{SuperstateInfo};


#[derive(Component, Debug, Deserialize, Serialize)]
#[require(InputMoveVector, MyZ(Being::Z_LEVEL), Replicated, MoveAnimActive,
BeingAltitude, Visibility, FacingDirection, AppliedModifiers, Transform,
EntityPrefix::new("BEING"), DimensionStrIdRef::overworld_fallback(), YSortOrigin(10.)
)]
pub struct Being;
impl Being {

    // /// max Z (clothes included)
    // pub const MAX_Z: MyZ = MyZ(1_000_000_000);

    // /// lowest z allowed for either clothes or body sprites
    // pub const MIN_Z: MyZ = MyZ(Self::MINZ_I32);

    pub const Z_LEVEL: i32 = 1_000;
}

#[derive(Component, Debug, Deserialize, Serialize, Clone, Copy, Hash, PartialEq,  )]
pub struct MainCharacter{#[entities] created_by: Entity}

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, Copy, Hash, PartialEq,  )]
pub struct InfiniteMorale;

pub type BeingAltitude = being_shared::BeingAltitude;


// #[derive(Component)]
// #[relationship_target(relationship = BodyPartOf)]
// pub struct BodyParts(Vec<Entity>);
// impl BodyParts { pub fn entities(&self) -> &Vec<Entity> {&self.0} }

// #[allow(dead_code)] 
#[derive(Component, Debug)]
pub struct RaceRef(#[entities] pub Entity);

#[derive(Component, Default, Deserialize, Serialize)]
#[require(ActivatingChunks)]//PROVISORIO, HACER UN SISTEMA Q AGREGUE ActivatingChunks automaticamente al being cuando sea de nuestra faccion
pub struct PlayerDirectControllable;

#[derive(Component)]
//no insertar este component si no se quiere restringir quien puede tomar control
/// entities: whitelisted players
pub struct ControlTakeoverWhitelist(#[entities] pub Vec<Entity>);//chequear si es de la misma facción antes de intentar tomar control


#[derive(Component, Debug, Deserialize, Serialize, Copy, Clone, Reflect)]
pub struct TouchingPortal(#[entities] pub Entity);


pub type ControlledLocally = being_shared::ControlledLocally;
pub type HumanControlled = being_shared::HumanControlled;

pub type Controls = being_shared::Controls;
pub type DirControlledBy = being_shared::ControlledBy;

#[derive(Component, Debug, Deserialize, Serialize, Reflect, )]
#[relationship(relationship_target = Followers)]
pub struct FollowerOf {#[relationship] #[entities] pub master: Entity,}

#[derive(Component, Debug, Reflect)]
#[relationship_target(relationship = FollowerOf)]
pub struct Followers(Vec<Entity>);
impl Followers {pub fn entities(&self) -> &Vec<Entity> {&self.0}}

#[derive(Component, Debug)]
pub struct LearningMultiplier(pub HashMap<Entity, f32>);

#[derive(Component, Debug)]
pub struct LearnableSkill {
    pub nid: u32,
}

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, )]
pub struct TargetSpawnPos(pub Vec2);//NO SÉ SI PONERLE UN FIELD Q SEA LA DIMENSIÓN
impl TargetSpawnPos {
    pub fn new(x: f32, y: f32) -> Self {
        Self(Vec2::new(x, y))
    }
}

#[derive(Component, Debug, Deserialize, Serialize, Copy, Clone, Hash, PartialEq, Eq, Reflect)]
#[relationship(relationship_target = CreatedCharacters)]
#[require(PlayerDirectControllable, being_shared::HumanControlled(true))]
pub struct CharacterCreatedBy {
    #[relationship] #[entities] pub player: Entity,
}

#[derive(Component, Debug, Reflect)]
#[relationship_target(relationship = CharacterCreatedBy)]
pub struct CreatedCharacters(Vec<Entity>);
impl CreatedCharacters { pub fn entities(&self) -> &[Entity] { &self.0 } }



//TANTO PARA BEINGS COMO PARA OBJETOS Y TILES

//HACER Q AFECTE LA VISIBILIDAD DE LAS COSAS . Q TENGAS
//DESPUES EN EL TERRAIN_GEN_SYSTEMS SE PUEDE HACER UN MATCH SEGÚN LA DIMENSION ACTUAL DEL PLAYER
//Y TENER UN PROC DE GENERACIÓN DE TERRAIN POR DIMENSION ANTES DE ENTRAR AL DOBLE FOR DE GENERACIÓN DE CADA TILE

