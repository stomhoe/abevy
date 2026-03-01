use crate::modifier_components::ModifierTags;

common::define_marker_components!(
    with_serde,
    require(ModifierTags),
    MassKg,
    Encumberance,
    Bulk,
    Durability,
    MaxDurability,
    MarketValue,
    Warmth,
    ArmorBlunt,
    ArmorSharp,
    ArmorFire,
    StackLimit,
);
