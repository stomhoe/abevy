#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
use serde::{Deserialize, Serialize};

use crate::modifier_components::{ModifierTags, ApplyMode};

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, )]
#[require(ModifierTags, )]
pub struct WalkSpeed;

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, )]
#[require(ModifierTags, )]
pub struct FlySpeed;

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, )]
#[require(ModifierTags, )]
pub struct SwimSpeed;

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, )]
#[require(ModifierTags, )]
/// if negative, regenerates blood, if positive, causes bleed. Value is the rate of hp lost or regenerated per second.
pub struct BleedRate;

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, )]
#[require(ModifierTags, )]
pub struct InvertMovement;

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, )]
#[require(ModifierTags, )]
/// Slowdown multiplier applied to movement speeds based on pain. Value is 1.0 - pain.
pub struct PainSlowdown;


#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, )]
#[require(ModifierTags, )]
pub struct HandlingCapability;


#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, )]
#[require(ModifierTags, )]
pub struct HitpointsCapacity;


#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, )]
#[require(ModifierTags, )]
pub struct HitpointRegenRate;

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, )]
#[require(ModifierTags, )]
pub struct BloodCapacity;


#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, )]
#[require(ModifierTags, )]
pub struct Consciousness;

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, )]
#[require(ModifierTags, )]
pub struct PainSensitivity;

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, )]
#[require(ModifierTags, )]
pub struct PainInfliction;


#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, )]
#[require(ModifierTags, )]
pub struct Manipulation;

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, )]
#[require(ModifierTags, )]
pub struct Vision;
