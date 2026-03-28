use bevy::prelude::*;
use game_common::game_common::ModifierSystems;
use game_common::game_common_components::HealthDamage;
use item_systems::generate_items_on_deaths;
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
            .chain()
            .in_set(ModifierSystems),
    )
    .add_systems(
        Update,
        (
            apply_health_damage.run_if(on_message::<HealthDamage>),
            update_body_manipulation_totals,
            (
                mark_dead_by_health.after(apply_health_damage),
                generate_items_on_deaths,
                despawn_entities_on_death,
            )
                .chain(),
        )
            .in_set(ModifierSystems),
    )
    .register_type::<AppliedModifiers>()
    .register_type::<ModifierTarget>()

    .replicate::<ModifierTarget>()
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
    .replicate_filtered::<ChildOf, With<ModifierTarget>>();
}
