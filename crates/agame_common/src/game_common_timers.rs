use bevy::prelude::*;

use paste::paste;

pub use crate::entity_zero_components::*;
use crate::{game_common_states::SimulationState};

#[derive(Component, Debug, Clone, )]
pub struct TimerComp(pub Timer);
impl TimerComp {
    pub fn new(seconds: f32) -> Self {
        Self::new_with_mode(seconds, TimerMode::Once)
    }
    pub fn new_with_mode(seconds: f32, mode: TimerMode) -> Self {
        Self(Timer::from_seconds(seconds, mode))
    }
}
impl Default for TimerComp {
    fn default() -> Self {
        Self::new_with_mode(30.0, TimerMode::Once)
    }
}

macro_rules! define_timer_bundles {
    ($base:ident, $mode:expr; $($component:path),* $(,)?) => {
        paste! {
            #[derive(Bundle, Debug)]
            pub struct $base(pub TimerComp $(, $component)*);

            impl $base {
                pub fn secs(seconds: f32) -> Self {
                    Self(TimerComp::new_with_mode(seconds, $mode) $(, $component)*)
                }
            }
            #[derive(Bundle, Debug)]
            pub struct [<Sim $base>](pub TimerComp $(, $component)*, SimRunningOnly);

            impl [<Sim $base>] {
                pub fn new(seconds: f32) -> Self {
                    Self(
                        TimerComp::new_with_mode(seconds, $mode)
                        $(, $component)*
                        , SimRunningOnly,
                    )
                }
            }
        }
    };
}
#[derive(Component, Debug, Copy, Clone, )]
#[require(TimerComp)]
pub struct MessageOnTimeout;

#[derive(Component, Debug, Copy, Clone, )]
#[require(TimerComp)]
pub struct DespawnOnTimeout;

#[derive(Component, Debug, Copy, Clone, )]
pub struct SimRunningOnly;

define_timer_bundles!(DespawnTimer, TimerMode::Once; DespawnOnTimeout);
define_timer_bundles!(TimeoutTimer, TimerMode::Once; MessageOnTimeout);
define_timer_bundles!(RepeatingTimeoutTimer, TimerMode::Repeating; MessageOnTimeout);

#[derive(Message, Debug, Clone, )]
pub struct TimedOut(pub Entity);
