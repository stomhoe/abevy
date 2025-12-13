use {crate::{modifier_components::*, modifier_move_components::*}, bevy_replicon::prelude::*};
#[allow(unused_imports)] use {bevy::prelude::*, superstate::superstate_plugin};


pub fn plugin(app: &mut App) {
    app
        .register_type::<ModifierTarget>()
        .register_type::<AppliedModifiers>()
        .register_type::<ModifierCategories>()
        .register_type::<BaseValue>()
        .register_type::<EffectiveValue>()
        .register_type::<Antidote>()
        .register_type::<OffsetValForSelf>()
        .register_type::<CopyValPortionForSelf>()
        .register_type::<MinForDamage>()
        .register_type::<ConvertsDamageOnNonPenetration>()
        .register_type::<ApplyMode>()
        .register_type::<MitigatingOnly>()
        .register_type::<HandlingCapability>()

        .replicate::<ModifierTarget>()
        .replicate::<BaseValue>()
        .replicate::<MitigatingOnly>()
        .replicate::<ApplyMode>()
        .replicate::<Speed>()
        .replicate::<InvertMovement>()
        .replicate::<EffectiveValue>()
        .replicate_filtered::<ChildOf, With<ModifierTarget>>()
    ;
}