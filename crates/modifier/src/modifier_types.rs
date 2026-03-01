use crate::modifier_components::ModifierTags;

common::define_marker_components!(
    with_serde,
    require(ModifierTags),
    WalkSpeed,
    FlySpeed,
    SwimSpeed,
    BleedRate,
    InvertMovement,
    PainSlowdown,
    HitpointsCapacity,
    HitpointRegenRate,
    BloodCapacity,
    Consciousness,
    PainSensitivity,
    PainInfliction,
    Manipulation,
    Vision,
);
