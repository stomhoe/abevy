use {crate::modifier_components::*};
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
       .register_type::<OperationType>()
       .register_type::<MitigatingOnly>()
       .register_type::<HandlingCapability>();

}