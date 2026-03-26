#[allow(unused_imports)]use bevy::ecs::entity::EntityHashSet;
use bevy::prelude::*;
#[allow(unused_imports)]
use bevy_replicon::prelude::*;
use bevy::platform::collections::HashMap;
use common::common_components::*;
use serde::{Deserialize, Serialize};
pub use being_shared::{Body, BodyOf, BodyTreeWeightSum};

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone)]
#[require(
    SparedFromHotReloading,
    AssetScoped,
    Replicated,
    Prefix::trunc("BodyTree")
)]
pub struct BodyTree;

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, )]
pub struct StatBudgetsToDistribute(pub HashIdMap<f32>);

#[derive(Component, Debug, Default, Clone)]
pub struct BodyTreeSexes(pub HashMap<String, crate::race::race_seris::RaceSexEntrySeri>);

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, Copy)]
pub struct CaloricBurnRateMultiplier(pub f32);

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, )]
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

#[derive(Debug, Copy, Clone, Message)]
pub struct IncomingDamage {
    pub body: Entity,
    pub amount: f32,
}


/*
BodyTreeDistributedTotals` is the cached stat budget for a body tree template.

Where it lives:
- Defined on the body-tree template side in [`crates/being/src/body/body_tree_seris.rs`](/mnt/data/abevy/crates/being/src/body/body_tree_seris.rs)
- Attached to the body-tree source entity in [`crates/being/src/body/body_tree_templ_init_systems.rs`](/mnt/data/abevy/crates/being/src/body/body_tree_templ_init_systems.rs)
- Read when building a concrete being body in [`crates/being/src/body/body_tree_build_systems.rs`](/mnt/data/abevy/crates/being/src/body/body_tree_build_systems.rs)

What it is used for:
- It stores the total available values for stats like mass, HP, regen, walk speed, swim speed, fly speed, manipulation, vision, pain, etc.
- `apply_distributions` uses it as the “pool” that gets split across the body parts based on each part’s forced and weighted distributions.
- In other words, it is the source budget that tells the builder how much of each stat the body tree should have overall.

What it is not:
- It is not the per-being runtime speed value.
- It is not the per-modifier instance state.
- It does not by itself update when parts are removed from a living being.

Why this matters for your refactor:
- Right now `build_body_tree` uses `BodyTreeDistributedTotals` once to compute the initial distributed modifiers for a being.
- If you want to reuse modifier entities and recalculate later when body parts change, `BodyTreeDistributedTotals` is still useful as the static template budget, but you still need a live per-being recompute pass for the current part set.

So the short version is:
- `BodyTreeDistributedTotals` = template-level total stat budget
- `apply_distributions` = distributes that budget onto body part modifiers
- runtime body changes still need a separate recalculation path

 */
