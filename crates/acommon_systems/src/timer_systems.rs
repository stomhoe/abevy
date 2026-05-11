use bevy::prelude::*;
#[allow(unused_imports, )]
use bevy::ecs::entity::EntityHashSet;
#[allow(unused_imports, )]
use ::common::*;
use ::game_common::*;

#[allow(unused_parens)]
pub fn tick_timers(
    mut cmd: Commands,
    sim_state: Res<State<SimulationState>>,
    mut query: Query<(Entity, &mut TimerComp, Has<SimRunningOnly>, Has<DespawnOnTimeout>, Has<MessageOnTimeout>), ()>,
    time: Res<Time>,
    mut writer: MessageWriter<TimedOut>,
    mut to_write: Local<Vec<TimedOut>>,
) {
    for (ent, mut timer, sim_only, despawn, message_on_timeout) in query.iter_mut() {
        let should_tick = match (
            sim_only,
            sim_state.get().is_running(),
        ) {
            (true, false, ) => false,
            _ => true,
        };
        if should_tick {
            timer.0.tick(time.delta());
        }
        if timer.0.is_finished() {
            if despawn{
                cmd.entity(ent).try_despawn();
            }
            if message_on_timeout {
                to_write.push(TimedOut(ent));
            }
        }
    }
    writer.write_batch(to_write.drain(..));
}


pub fn tick_time_based_multipliers(
    time: Res<Time>,
    mut query: Query<(
        &mut TimeBasedMultiplier,
        Option<&TickMultFactor>,
        Option<&TickMultFactors>,
    )>,
) {
    for (mut multiplier, tick_mult_factor, tick_mult_factors) in query.iter_mut() {
        let mut factor = tick_mult_factor.map(|f| f.value()).unwrap_or(1.0);
        if let Some(factors) = tick_mult_factors {
            factor *= factors.0.iter().map(|f| f.value()).product::<f32>();
        }
        multiplier.timer.tick(time.delta().mul_f32(factor));
    }
}