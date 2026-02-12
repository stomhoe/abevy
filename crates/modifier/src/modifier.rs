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
        FixedUpdate,
        (update_modifier_effective_values.run_if(on_timer(Duration::from_millis(200))),)
            .in_set(ModifierSystems),
    )
    .register_type::<AppliedModifiers>()

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
