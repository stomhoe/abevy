use bevy::{platform::{collections::HashMap}, prelude::*};
use bevy_replicon::prelude::Replicated;
use serde::{Deserialize, Serialize};
use crate::{common::common_components::{EntityPrefix, MyZ}, game::{being::{body::body_components::BodyPartOf, modifier::modifier_components::AppliedModifiers, movement::movement_components::*, sprite::sprite_components::{MoveAnimActive, SpriteCfgsBuiltSoFar}}, game_components::FacingDirection, player::player_components::Controls, tilemap::chunking_components::ActivatesChunks}, AppState};


#[derive(Component, Debug, Deserialize, Serialize)]
#[require(InputMoveVector,  MyZ(Being::MINZ_I32), Replicated, MoveAnimActive,
Altitude, Visibility, FacingDirection, AppliedModifiers, 
EntityPrefix::new("BEING"), SpriteCfgsBuiltSoFar)]
pub struct Being;
impl Being {

    /// max Z (clothes included)
    pub const MAX_Z: MyZ = MyZ(1_000_000_000);

    /// lowest z allowed for either clothes or body sprites
    pub const MIN_Z: MyZ = MyZ(Self::MINZ_I32);

    pub const MINZ_I32: i32 = 1_000;
}

#[derive(Component, Debug, Deserialize, Serialize, Clone, Copy, Hash, PartialEq,  )]
pub struct MainCharacter{#[entities] created_by: Entity}

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, Copy, Hash, PartialEq,  )]
pub struct InfiniteMorale;



#[derive(Component)]
#[relationship_target(relationship = BodyPartOf)]
pub struct BodyParts(Vec<Entity>);
impl BodyParts { pub fn entities(&self) -> &Vec<Entity> {&self.0} }

#[allow(dead_code)] 
#[derive(Component, Debug)]
pub struct RaceRef(#[entities] pub Entity);

#[derive(Component, Default)]
#[require(ActivatesChunks)]//TODO PONER ActivatesChunks CUANDO SEA ADECUADO
pub struct PlayerDirectControllable;

#[derive(Component)]
//no insertar este component si no se quiere restringir quien puede tomar control
/// entities: whitelisted players
pub struct ControlTakeoverWhitelist(#[entities] pub Vec<Entity>);//chequear si es de la misma facción antes de intentar tomar control

#[derive(Component, Debug, Deserialize, Serialize)]
#[relationship(relationship_target = Controls)]
//client entity en control del being, ya sea manualmente o mediante su CPU
pub struct ControlledBy  { 
    #[relationship] #[entities]
    pub player: Entity 
}


#[derive(Component, Debug, Default, )]
pub struct ControlledLocally;

#[derive(Component, Debug, Deserialize, Serialize)]
pub struct CpuControlled;




#[derive(Component, Debug, Deserialize, Serialize)]
#[relationship(relationship_target = Followers)]
pub struct FollowerOf {
    #[relationship] #[entities]
    pub master: Entity,
}

#[derive(Component)]
#[relationship_target(relationship = FollowerOf)]
pub struct Followers(Vec<Entity>);


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




//TANTO PARA BEINGS COMO PARA OBJETOS Y TILES

//HACER Q AFECTE LA VISIBILIDAD DE LAS COSAS . Q TENGAS
//DESPUES EN EL TERRAIN_GEN_SYSTEMS SE PUEDE HACER UN MATCH SEGÚN LA DIMENSION ACTUAL DEL PLAYER
//Y TENER UN PROC DE GENERACIÓN DE TERRAIN POR DIMENSION ANTES DE ENTRAR AL DOBLE FOR DE GENERACIÓN DE CADA TILE

