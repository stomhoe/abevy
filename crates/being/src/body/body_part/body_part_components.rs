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

#[derive(Component, Debug, Deserialize, Serialize, MapEntities, Clone, )]
#[relationship(relationship_target = BodyParts)]
pub struct BodyPartOf {#[relationship] #[entities] pub body: Entity,}

#[derive(Component, Debug, Clone)]
#[relationship_target(relationship = BodyPartOf)]
pub struct BodyParts(Vec<Entity>);
impl BodyParts { pub fn entities(&self) -> &Vec<Entity> {&self.0} }

#[derive(Component, Debug, Deserialize, Serialize, MapEntities, Clone)]
#[relationship(relationship_target = BodyPartChildren)]
pub struct BodyPartParent {#[relationship] #[entities] pub parent: Entity,}

#[derive(Component, Debug, Clone)]
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

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone)]
pub struct BodyPartForcedDistribution(pub HashIdMap<f32>);

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone)]
pub struct BodyPartWeightedDistribution(pub HashIdMap<f32>);

pub struct BodyPartStat;
impl BodyPartStat {
    pub const STAT_MASS_KG: HashId = HashId::hash("mass_kg");
    pub const STAT_HP_CAPACITY: HashId = HashId::hash("hp_capacity");
    pub const STAT_HP_REGEN_RATE: HashId = HashId::hash("hp_regen_rate");
    pub const STAT_BLOOD_CAPACITY: HashId = HashId::hash("blood_capacity");
    pub const STAT_BLOOD_PUMPING: HashId = HashId::hash("blood_pumping");
    pub const STAT_WALK_SPEED: HashId = HashId::hash("walk_speed");
    pub const STAT_SWIM_SPEED: HashId = HashId::hash("swim_speed");
    pub const STAT_FLY_SPEED: HashId = HashId::hash("fly_speed");
    pub const STAT_MANIPULATION_DEXTERITY: HashId = HashId::hash("manip_dex");
    pub const STAT_MANIPULATION_STRENGTH: HashId = HashId::hash("manip_str");
    pub const STAT_VISION: HashId = HashId::hash("vision");
    pub const STAT_PAIN_SENSITIVITY: HashId = HashId::hash("pain_sensitivity");
    pub const STAT_CALORIC_BURN_RATE: HashId = HashId::hash("caloric_burn_rate");
    pub const STAT_CALORIC_CAPACITY: HashId = HashId::hash("caloric_capacity");
}

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


pub type BodyPartTags = TagSet;
