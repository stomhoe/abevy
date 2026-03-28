use crate::modifier_components::ModifierTags;

common::define_marker_components!(
    repli,
    require(ModifierTags),
    define_copy_from_template_system(copy_modifier_markers_from_template),
    define_plugin(plugin),
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
    ManipulationDexterity,
    ManipulationStrength,
    Vision,
);
