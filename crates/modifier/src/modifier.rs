use {crate::{modifier_components::*, modifier_move_components::*}, bevy_replicon::prelude::*};
use bevy::prelude::*;

pub fn plugin(app: &mut App) {
    app
        .register_type::<ModifierTarget>()
        .register_type::<AppliedModifiers>()
        .register_type::<ModifierTags>()
        .register_type::<BaseValue>()
        .register_type::<CurrFinalValue>()
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
        .replicate::<CurrFinalValue>()//NO SÉ SI ES TEMPORAL O NO
        .replicate::<MitigatingOnly>()
        .replicate::<ApplyMode>()
        .replicate::<Speed>()
        .replicate::<InvertMovement>()
        .replicate::<CurrFinalValue>()
        .replicate_filtered::<ChildOf, With<ModifierTarget>>()
    ;
}