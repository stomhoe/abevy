use bevy::prelude::*;
use game_common::{HostSystems, Templ, game_common::ModifierSystems};
use {
    crate::modifier_hp_systems::*,
    crate::modifier_manip_systems::*,
    crate::modifier_systems::*,
    bevy::time::common_conditions::on_timer,
    bevy_replicon::prelude::*,
    modifier_shared::modifier_components::*,
    std::time::Duration,
};

pub fn plugin(app: &mut App) {
    modifier_shared::modifier_types::plugin(app);
    modifier_shared::modifier_tool_types::plugin(app);
    modifier_shared::modifier_item_types::plugin(app);

    app.add_systems(
        FixedUpdate,
        (
            materialize_modifier_synergies,
            update_modifier_effective_values.run_if(on_timer(Duration::from_millis(300))),
            sync_modifier_name_to_effects,
        )
            .in_set(ModifierSystems).in_set(HostSystems),
    )
    .add_systems(
        Update,
        (
            update_body_manipulation_totals.run_if(on_timer(Duration::from_millis(300))),
            (
                despawn_entities_on_death,
            )
        )
        .in_set(ModifierSystems).in_set(HostSystems),
    )
    .register_type::<AppliedModifiers>()
    .register_type::<ModifierTarget>()

    .replicate::<ModifierTarget>()
    .replicate::<ModifierTags>()
    .replicate::<BaseValue>()
    .replicate::<CurrEffectiveValue>()
    .replicate::<MitigatingOnly>()
    .replicate::<ApplyMode>()
    .replicate::<Antidote>()
    .replicate::<ModifierSynergies>()
    .replicate::<OffsetValForSelf>()
    .replicate::<CopyFracOfOthersIntoSelf>()
    .replicate::<MinForDamage>()
    .replicate::<ConvertsDamageOnNonPenetration>()
    .replicate_filtered::<ChildOf, (With<ModifierTarget>, Without<Templ>)>();
}
