#[allow(unused_imports)]use bevy::ecs::entity::EntityHashSet;
use bevy::prelude::*;
#[allow(unused_imports)]
use bevy_replicon::prelude::*;
use bevy::platform::collections::HashMap;
use common::common_components::*;
use serde::{Deserialize, Serialize};
pub use ::being_shared::*;

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone)]
#[require(
    SparedFromHotReloading,
    AssetScoped,
    Prefix::trunc("Body")
)]
pub struct Body;

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, )]
pub struct StatBudgetsToDistributeAmongBodyPartsOfTemplBody(pub HashIdMap<f32>);

#[derive(Component, Debug, Default, Clone)]
pub struct BodySexes(pub HashMap<String, RaceSexEntrySeri>);

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, Copy)]
pub struct CaloricBurnRateMultiplier(pub f32);

#[derive(Component, Debug, Deserialize, Serialize, Clone, )]
pub struct BodySums {
    pub total_hp: f32,
    pub current_hp: f32,
    pub blood: f32,
    pub blood_capacity: f32,
    pub bleed_rate: f32,
    pub consciousness: f32,
    pub pain: f32,
    pub vision: f32,
    pub manip_dex: f32,
    pub manip_str: f32,
}
impl Default for BodySums {
    fn default() -> Self {
        Self {
            total_hp: 0.0,
            current_hp: 0.0,
            blood: f32::NAN,
            blood_capacity: 0.0,
            bleed_rate: 0.0,
            consciousness: 0.0,
            pain: 0.0,
            vision: 0.0,
            manip_dex: 0.0,
            manip_str: 0.0,
        }
    }
}

#[derive(Debug, Copy, Clone, Message)]
pub struct IncHealthDamageOrHeal {
    pub target_ent: Entity,
    pub amount: f32,
    pub distribute_mode: DamageDistributeMode,
}
impl IncHealthDamageOrHeal {
    pub fn new(target: Entity, amount: f32, ) -> Self {
        Self {
            target_ent: target,
            amount,
            distribute_mode: DamageDistributeMode::default(),
        }
    }
}
#[derive(Debug, Copy, Clone, Message, Default)]
pub enum DamageDistributeMode {
    #[default]
    SampledBodyPart,
    EquitativelyDistributedBetweenAllBasedOnRatioOverBodyTotalHitpointsCapacity,
    /// can be used to specifically heal damaged bodyparts
    DistributeProportionalToPreexistentDamage,

}

/*
BodyDistributedTotals` is the cached stat budget for a body tree template.

Where it lives:
- Defined on the body-tree template side in [`crates/being/src/body/body_seris.rs`](/mnt/data/abevy/crates/being/src/body/body_seris.rs)
- Attached to the body-tree source entity in [`crates/being/src/body/body_templ_init_systems.rs`](/mnt/data/abevy/crates/being/src/body/body_templ_init_systems.rs)
- Read when building a concrete being body in [`crates/being/src/body/body_build_systems.rs`](/mnt/data/abevy/crates/being/src/body/body_build_systems.rs)

What it is used for:
- It stores the total available values for stats like mass, HP, regen, walk speed, swim speed, fly speed, manipulation, vision, pain, etc.
- `apply_distributions` uses it as the “pool” that gets split across the body parts based on each part’s forced and weighted distributions.
- In other words, it is the source budget that tells the builder how much of each stat the body tree should have overall.

What it is not:
- It is not the per-being runtime speed value.
- It is not the per-modifier instance state.
- It does not by itself update when parts are removed from a living being.

Why this matters for your refactor:
- Right now `build_body` uses `BodyDistributedTotals` once to compute the initial distributed modifiers for a being.
- If you want to reuse modifier entities and recalculate later when body parts change, `BodyDistributedTotals` is still useful as the static template budget, but you still need a live per-being recompute pass for the current part set.

So the short version is:
- `BodyDistributedTotals` = template-level total stat budget
- `apply_distributions` = distributes that budget onto body part modifiers
- runtime body changes still need a separate recalculation path

 */
