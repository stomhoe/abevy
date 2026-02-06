use bevy::{ecs::entity::MapEntities, platform::collections::HashMap};
#[allow(unused_imports)] use bevy::prelude::*;
use bevy_replicon::prelude::*;
use common::{common_components::*, common_types::*, define_entity_map_systems};
use serde::{Deserialize, Serialize};
use std::hash::Hash;

use crate::sprite_scale_offset::{self, *};

#[allow(unused_imports)] use {bevy::prelude::*, };

pub fn plugin(app: &mut App) {
    app
        //.register_type::<SpriteConfigStrIds>()
        .register_type::<BaseHolderRef>()
        .register_type::<HeldSprites>()

        .replicate::<BaseHolderRef>()

        .replicate_once_filtered::<Transform, With<BaseHolderRef>>()
        .replicate_once_filtered::<ChildOf, With<BaseHolderRef>>()
        .replicate::<MovementBased>()
        .replicate::<GroundingBased>()

    ;
    sprite_scale_offset::plugin(app);
}



#[derive(Component, Debug, Default, Deserialize, Serialize, Copy, Clone, Reflect)]
pub struct MovementBased;

#[derive(Component, Debug, Default, Deserialize, Serialize, Copy, Clone, Reflect)]
pub struct GroundingBased;



#[derive(Component, Debug, Deserialize, Serialize, Copy, Clone, Reflect)]
#[relationship(relationship_target = HeldSprites)]
#[require(Prefix::trunc("Sprite"), )]
pub struct BaseHolderRef {#[relationship]#[entities]pub base: Entity, }

#[derive(Component, Debug, Reflect)]
#[relationship_target(relationship = BaseHolderRef)]
pub struct HeldSprites(Vec<Entity>);

impl HeldSprites {
    pub fn entities(&self) -> &Vec<Entity> {
        &self.0
    }
}
impl std::ops::Deref for HeldSprites {
    type Target = [Entity];
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl IntoIterator for HeldSprites {
    type Item = Entity;
    type IntoIter = std::vec::IntoIter<Entity>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}
impl<'a> IntoIterator for &'a HeldSprites {
    type Item = &'a Entity;
    type IntoIter = std::slice::Iter<'a, Entity>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

// #[derive(Component, Debug, Deserialize, Serialize, Clone, Reflect)]
// /// DON'T REPLICATE
// pub struct SpriteConfigStrIds(Vec<StrId>);
// impl SpriteConfigStrIds {
//     pub fn new<S: AsRef<str>>(ids: impl IntoIterator<Item = S>) -> Self {
//         Self(ids.into_iter().map(|s| StrId::trunc(s)).collect())
//     }
//     pub fn ids(&self) -> &Vec<StrId> { &self.0 }
// }

#[derive(Component, Debug, Deserialize, Serialize, Clone, Reflect)]

pub struct SampleSpritesFromStrIds(Vec<StrId>);
impl SampleSpritesFromStrIds {
    pub fn new<S: AsRef<str>>(ids: impl IntoIterator<Item = S>) -> Self {
        Self(ids.into_iter().map(|s| StrId::trunc(s)).collect())
    }
    pub fn ids(&self) -> &Vec<StrId> { &self.0 }
}

#[derive(Component, Debug, Deserialize, Serialize, Clone, Reflect, MapEntities, )]

pub struct SampleSprites(#[entities]pub Vec<Entity>);
impl SampleSprites {
    pub fn new(entities: Vec<Entity>) -> Self {
        Self(entities)
    }
    pub fn entities(&self) -> &Vec<Entity> {
        &self.0
    }
}

#[derive(Component, Debug, Default, Deserialize, Serialize, Reflect, Clone, Copy, )]
pub struct YSortOrigin(pub f32);//TAL VEZ ES BUENA IDEA PONERLE ESTO OBLIGATORIAMENTE A TODOS LOS SPRITES, ASÍ TODOS AUMENTAN O DISMINUYEN CONJUNTAMENTE DE Z
impl YSortOrigin {
    pub const Y_SORT_DIV: f32 = 1e-7;//-7
}

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, Copy, Reflect)]
pub struct AcZ(pub f32);

impl AcZ {
    pub fn new(z: f32) -> Self { Self(z) }
    pub fn used_float(&self) -> f32 { self.0 as f32 * Self::Z_MULTIPLIER }
    pub const Z_MULTIPLIER: f32 = 1e-5;
}

impl PartialEq for AcZ {
    fn eq(&self, other: &Self) -> bool {
        self.0.to_bits() == other.0.to_bits()
    }
}
impl Eq for AcZ {}
impl Hash for AcZ {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.to_bits().hash(state)
    }
}