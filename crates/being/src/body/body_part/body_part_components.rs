use bevy::ecs::entity::MapEntities;
#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
use common::common_components::*;
use common::common_tag_components::TagSet;
use serde::{Deserialize, Serialize};

#[derive(Component, Debug, Default, Deserialize, Serialize, Copy, Clone)]
#[require(Replicated, Prefix::trunc("BodyPart"), )]
pub struct BodyPart;

#[derive(Component, Debug, Default, Deserialize, Serialize, Copy, Clone)]
pub struct BodyRootPart;

#[derive(Component, Debug, Deserialize, Serialize, MapEntities, )]
#[relationship(relationship_target = BodyParts)]
pub struct BodyPartOf {#[relationship] #[entities] pub body: Entity,}

#[derive(Component, Debug, )]
#[relationship_target(relationship = BodyPartOf)]
pub struct BodyParts(Vec<Entity>);
impl BodyParts { pub fn entities(&self) -> &Vec<Entity> {&self.0} }

#[derive(Component, Debug, Deserialize, Serialize, MapEntities, )]
#[relationship(relationship_target = BodyPartChildren)]
pub struct BodyPartParent {#[relationship] #[entities] pub parent: Entity,}

#[derive(Component, Debug, )]
#[relationship_target(relationship = BodyPartParent)]
pub struct BodyPartChildren(Vec<Entity>);
impl BodyPartChildren { pub fn entities(&self) -> &Vec<Entity> {&self.0} }

#[derive(Component, Debug, Default, Clone, )]
pub struct BodyPartSlots(pub Vec<StrId>);
impl BodyPartSlots {
    pub fn new<S: AsRef<str>>(slots: impl IntoIterator<Item = S>) -> Self {
        Self(slots.into_iter().map(|s| StrId::trunc(s.as_ref())).collect())
    }
}

#[derive(Component, Debug, Default, Copy, Clone, )]
pub struct BodyPartCoverageWeight(pub u16);

#[derive(Component, Debug, Default, Deserialize, Serialize, Copy, Clone)]
pub struct BodyPartVital;

#[derive(Component, Debug, Default, Deserialize, Serialize, Copy, Clone)]
pub struct BodyPartMissing;

#[derive(Component, Debug, Default, Deserialize, Serialize, Copy, Clone, )]
pub struct BodyPartDamage(pub f32);

#[derive(Component, Debug, Default, Deserialize, Serialize, Copy, Clone, )]
pub enum BodyPartDepth {
    #[default]
    Surface,
    Inside,
    Core,
}

impl From<&str> for BodyPartDepth {
    fn from(value: &str) -> Self {
        match value.trim().to_lowercase().as_str() {
            "inside" | "inner" => BodyPartDepth::Inside,
            "core" | "root" => BodyPartDepth::Core,
            _ => BodyPartDepth::Surface,
        }
    }
}

impl From<String> for BodyPartDepth {
    fn from(value: String) -> Self {
        BodyPartDepth::from(value.as_str())
    }
}

#[derive(Component, Debug, Default, Clone, )]
pub struct BodyPartKind(pub StrId);

pub type BodyPartTags = TagSet;
