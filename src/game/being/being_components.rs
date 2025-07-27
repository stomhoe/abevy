use bevy::{platform::{collections::HashMap}, prelude::*};
use bevy_replicon::prelude::Replicated;
use serde::{Deserialize, Serialize};
use superstate::{SuperstateInfo};
use crate::{common::common_components::GameZindex, game::{being::{modifier::modifier_components::AppliedModifiers, movement::movement_components::*}, game_components::FacingDirection, tilemap::chunking_components::ActivatesChunks}, AppState};



#[derive(Component)]
#[require(StateScoped::<AppState>)]
pub struct Body {}
 



#[allow(dead_code)] 
#[derive(Component, Debug)]
pub struct RaceRef(pub Entity);

#[derive(Component, Default)]
#[require(SuperstateInfo<PlayerDirectControllable>, ActivatesChunks)]//TODO PONER ActivatesChunks CUANDO SEA ADECUADO
pub struct PlayerDirectControllable;

#[derive(Component)]
#[require(PlayerDirectControllable)]
pub struct AvailableForControl;//chequear si es de la misma facción antes de intentar tomar control

#[derive(Component, Debug, Deserialize, Serialize)]
#[require(PlayerDirectControllable, Replicated)]
pub struct ControlledBy ( #[entities] pub Entity);

#[derive(Component, Debug, Default, )]
pub struct ControlledBySelf;

#[derive(Component, Debug, Deserialize, Serialize)]
#[require(InputMoveVector, FinalMoveVector, GameZindex(500), Replicated, Altitude, Visibility, FacingDirection, AppliedModifiers)]
pub struct Being;

#[derive(Component)]
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





#[derive(Component, Debug, )]
pub struct CurrentDimension(u32);//TANTO PARA BEINGS COMO PARA OBJETOS Y TILES

//HACER Q AFECTE LA VISIBILIDAD DE LAS COSAS . Q TENGAS
//DESPUES EN EL TERRAIN_GEN_SYSTEMS SE PUEDE HACER UN MATCH SEGÚN LA DIMENSION ACTUAL DEL PLAYER
//Y TENER UN PROC DE GENERACIÓN DE TERRAIN POR DIMENSION ANTES DE ENTRAR AL DOBLE FOR DE GENERACIÓN DE CADA TILE

