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

#[derive(Debug, Deserialize, Serialize, Clone, )]
pub struct Hit {
    pub damage: f32,
    pub source_ent: Entity,
}
impl Default for Hit {
    fn default() -> Self {
        Self {
            damage: 0.0,
            source_ent: Entity::PLACEHOLDER,
        }
    }
}

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, )]
pub struct AccuDamage {
    pub total: f32,
    pub hits: Vec<Hit>,
}
impl AccuDamage {
    pub fn with_hit(damage: f32, source_ent: Entity) -> Self {
        let mut this = Self::default();
        this.push_hit(damage, source_ent);
        this
    }

    pub fn push_hit(&mut self, damage: f32, source_ent: Entity) {
        if damage <= 0.0 {
            self.heal(-damage);
            return;
        }
        self.total += damage;
        self.hits.push(Hit { damage, source_ent });
    }

    pub fn heal(&mut self, mut amount: f32) {
        if amount <= 0.0 || self.hits.is_empty() {
            return;
        }
        while amount > 0.0 {
            let Some(first_hit) = self.hits.first_mut() else {
                break;
            };
            let healed = first_hit.damage.min(amount);
            first_hit.damage -= healed;
            self.total = (self.total - healed).max(0.0);
            amount -= healed;
            if first_hit.damage > 0.0 {
                break;
            }
            self.hits.remove(0);
        }
    }
}

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
