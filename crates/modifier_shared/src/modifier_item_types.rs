use crate::modifier_components::ModifierTags;

common::define_marker_components!(
    repli,
    require(ModifierTags),
    define_copy_from_template_system(copy_modifier_item_markers_from_template),
    define_plugin(plugin),
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
