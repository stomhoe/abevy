use bevy::prelude::*;
use game_common::game_common::ModifierSystems;
use {
    crate::{
        modifier_components::*, modifier_move_components::*, modifier_systems::*, modifier_types::*,
    },
    bevy::time::common_conditions::on_timer,
    bevy_replicon::prelude::*,
    std::time::Duration,
};

pub fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (update_modifier_effective_values.run_if(on_timer(Duration::from_millis(10000))),)
            .in_set(ModifierSystems),
    )
    .register_type::<ModifierTarget>()
    .register_type::<AppliedModifiers>()
    .register_type::<ModifierTags>()
    .register_type::<BaseValue>()
    .register_type::<CurrEffectiveValue>()
    .register_type::<Antidote>()
    .register_type::<OffsetValForSelf>()
    .register_type::<CopyMultOfOthersIntoSelf>()
    .register_type::<MinForDamage>()
    .register_type::<ConvertsDamageOnNonPenetration>()
    .register_type::<ApplyMode>()
    .register_type::<MitigatingOnly>()
    .register_type::<HandlingCapability>()
    .register_type::<HitpointRegenRate>()
    .register_type::<BloodCapacity>()
    .register_type::<Consciousness>()
    .register_type::<PainSensitivity>()
    .register_type::<PainInfliction>()
    .register_type::<PainSlowdown>()
    .register_type::<Manipulation>()
    .register_type::<Vision>()
    .register_type::<WalkSpeed>()
    .register_type::<SwimSpeed>()
    .register_type::<FlySpeed>()
    .replicate::<ModifierTarget>()
    .replicate::<BaseValue>()
    .replicate::<CurrEffectiveValue>() //NO SÉ SI ES TEMPORAL O NO
    .replicate::<MitigatingOnly>()
    .replicate::<ApplyMode>()
    .replicate::<WalkSpeed>()
    .replicate::<SwimSpeed>()
    .replicate::<FlySpeed>()
    .replicate::<InvertMovement>()
    .replicate::<HitpointsCapacity>()
    .replicate::<HitpointRegenRate>()
    .replicate::<BloodCapacity>()
    .replicate::<Consciousness>()
    .replicate::<PainSensitivity>()
    .replicate::<PainInfliction>()
    .replicate::<PainSlowdown>()
    .replicate::<HandlingCapability>()
    .replicate::<Manipulation>()
    .replicate::<Vision>()
    .replicate::<Antidote>()
    .replicate::<OffsetValForSelf>()
    .replicate::<CopyMultOfOthersIntoSelf>()
    .replicate::<MinForDamage>()
    .replicate::<ConvertsDamageOnNonPenetration>()
    .replicate::<BleedRate>()
    .replicate_filtered::<ChildOf, With<ModifierTarget>>();
}
