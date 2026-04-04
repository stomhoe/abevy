use bevy::ecs::entity::MapEntities;
#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
use common::common_components::*;
use common::common_tag_components::TagSet;
use item_shared::item_components::SlottedItemHolder;
use serde::{Deserialize, Serialize};

#[derive(Component, Debug, Default, Deserialize, Serialize, Copy, Clone)]
#[require(Prefix::trunc("Bodypart"), )]
pub struct Bodypart;

#[derive(Component, Debug, Default, Deserialize, Serialize, Copy, Clone)]
pub struct TreeRoot;

#[derive(Component, Debug, Deserialize, Serialize, MapEntities, Clone, )]
#[relationship(relationship_target = BodypartChildrenBodyparts)]
pub struct BodypartChildOfBodypart {#[relationship] #[entities] pub parent_bodypart: Entity,}

#[derive(Component, Debug, Clone, Reflect)]
#[relationship_target(relationship = BodypartChildOfBodypart)]
pub struct BodypartChildrenBodyparts(Vec<Entity>);

//dont use
#[allow(unused, )]
type BodypartSlots = SlottedItemHolder;

#[derive(Component, Debug, Default, Copy, Clone, )]
pub struct BodypartCoverageWeight(pub u16);

pub struct BodypartStat;
impl BodypartStat {
    pub const STAT_MASS_KG: HashId = HashId::hash("mass_kg");
    pub const STAT_HP_CAPACITY: HashId = HashId::hash("hp_capacity");
    pub const STAT_HP_REGEN_RATE: HashId = HashId::hash("hp_regen_rate");
    pub const STAT_BLOOD_CAPACITY: HashId = HashId::hash("blood_capacity");
    pub const STAT_BLOOD_PUMPING: HashId = HashId::hash("blood_pumping");
    pub const STAT_WALK_STRENGTH: HashId = HashId::hash("walk_strength");
    pub const STAT_SWIM_STRENGTH: HashId = HashId::hash("swim_strength");
    pub const STAT_FLY_STRENGTH: HashId = HashId::hash("fly_strength");
    pub const STAT_MANIPULATION_DEXTERITY: HashId = HashId::hash("manip_dex");
    pub const STAT_MANIPULATION_STRENGTH: HashId = HashId::hash("manip_str");
    pub const STAT_VISION: HashId = HashId::hash("vision");
    pub const STAT_PAIN_SENSITIVITY: HashId = HashId::hash("pain_sensitivity");
    pub const STAT_CALORIC_BURN_RATE: HashId = HashId::hash("caloric_burn_rate");
    pub const STAT_CALORIC_CAPACITY: HashId = HashId::hash("caloric_capacity");
}

#[derive(Component, Debug, Default, Deserialize, Serialize, Copy, Clone)]
pub struct Vital;

#[derive(Component, Debug, Default, Deserialize, Serialize, Copy, Clone)]
pub struct Missing;

#[derive(Component, Debug, Default, Deserialize, Serialize, Copy, Clone, )]
pub struct AccuDamage(pub f32);

#[derive(Component, Debug, Default, Deserialize, Serialize, Copy, Clone, )]
pub enum BodypartDepth {
    #[default]
    Surface,
    Inside,
    Core,
}

impl From<&str> for BodypartDepth {
    fn from(value: &str) -> Self {
        match value.trim().to_lowercase().as_str() {
            "inside" | "inner" => BodypartDepth::Inside,
            "core" | "root" => BodypartDepth::Core,
            _ => BodypartDepth::Surface,
        }
    }
}

impl From<String> for BodypartDepth {
    fn from(value: String) -> Self {
        BodypartDepth::from(value.as_str())
    }
}

pub type BodypartTags = TagSet;


#[derive(Component, Debug, Default, Deserialize, Serialize, Clone)]
pub struct BodypartForcedStats(pub HashIdMap<f32>);

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, )]
pub struct BodypartWeightedDistribution(pub HashIdMap<f32>);
