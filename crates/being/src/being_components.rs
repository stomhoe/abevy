use ::being_shared::*;
use bevy::{ecs::entity::{EntityHashSet, MapEntities}, platform::collections::HashMap, prelude::*};
use bevy_replicon::prelude::{ClientState, Replicated};

use modifier::modifier_components::AppliedModifiers;
use movement::movement_components::*;

use common::common_components::*;
use sprite_animation_shared::MoveAnimActive;
use serde::{Deserialize, Serialize};

#[derive(Component, Debug, Deserialize, Serialize, Copy, Clone, Default)]

#[require(InputDirection, MoveVecMag, Replicated, MoveAnimActive,
Grounding, Visibility, CardinalDirection, AppliedModifiers, Transform,
Prefix::trunc("BEING"), DimensionStrIdRef::overworld_fallback(), AssetScoped, SparedFromHotReloading,
GridLockedMovement )]
pub struct Being;
impl Being {

    // /// max Z (clothes included)
    // pub const MAX_Z: MyZ = MyZ(1_000_000_000);

    // /// lowest z allowed for either clothes or body sprites
    // pub const MIN_Z: MyZ = MyZ(Self::MINZ_I32);

    pub const Z_LEVEL: f32 = 1_000.;
}

#[derive(Component, Debug, Default, Deserialize, Serialize, Copy, Clone, Reflect)]
pub struct BodyCollisionRadius(pub u32);

#[derive(Component, Debug, Clone, Copy, Hash, PartialEq)]
pub struct MainCharacter{#[entities] created_by: Entity}

#[derive(Component, Debug, Default, Clone, Copy, Hash, PartialEq)]
pub struct InfiniteMorale;

// #[derive(Component, Clone)]
// #[relationship_target(relationship = BodyPartOf)]
// pub struct BodyParts(Vec<Entity>);
// impl BodyParts { pub fn entities(&self) -> &Vec<Entity> {&self.0} }

#[derive(Component, Default, Deserialize, Serialize, Clone)]
pub struct PlayerDirectControllable;

#[derive(Component, MapEntities, Clone)]
//no insertar este component si no se quiere restringir quien puede tomar control
/// entities: whitelisted players
pub struct ControlTakeoverWhitelist(#[entities] pub Vec<Entity>);//chequear si es de la misma facción antes de intentar tomar control

#[derive(Component, Debug, Copy, Clone, MapEntities)]
pub struct TouchingPortal(#[entities] pub Entity);

#[derive(Component, Debug, Deserialize, Serialize, Reflect, MapEntities, Copy, Clone, )]
#[relationship(relationship_target = Followers)]
pub struct FollowerOf {#[relationship] #[entities] pub master: Entity,}

#[derive(Component, Debug, Reflect, Clone)]
#[relationship_target(relationship = FollowerOf)]
pub struct Followers(Vec<Entity>);
impl Followers {pub fn entities(&self) -> &Vec<Entity> {&self.0}}

#[derive(Component, Debug, Clone)]
pub struct LearningMultiplier(pub HashMap<Entity, f32>);

#[derive(Component, Debug, Default, Clone)]
pub struct TargetSpawnPos(pub Vec2);//NO SÉ SI PONERLE UN FIELD Q SEA LA DIMENSIÓN
impl TargetSpawnPos {
    pub fn new(x: f32, y: f32) -> Self {
        Self(Vec2::new(x, y))
    }
}

#[derive(Component, Debug, Deserialize, Serialize, Reflect, MapEntities, Copy, Clone, )]
#[relationship(relationship_target = CreatedCharacters)]
#[require(PlayerDirectControllable, being_shared::IsHumanControlled(true))]
pub struct CharacterCreatedBy {
    #[relationship] #[entities] pub player: Entity,
}

#[derive(Component, Debug, Clone)]
#[relationship_target(relationship = CharacterCreatedBy)]
pub struct CreatedCharacters(Vec<Entity>);
impl CreatedCharacters { pub fn entities(&self) -> &[Entity] { &self.0 } }

use bevy::ecs::entity::EntityHashMap;
use sprite_shared::SampleSpriteEnts;
use ::tilemap_shared::*;
#[derive(Component, Debug, Deserialize, Serialize, Clone, MapEntities, )]
pub struct MappedSpritesToSample(
    /// sexent - samplespriteents
    #[entities] pub EntityHashMap<SampleSpriteEnts>,
);
