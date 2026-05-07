use bevy::{ecs::entity::MapEntities, platform::collections::HashMap, prelude::*};

use splines::{Interpolation, Key, Spline};
use std::hash::Hash;
use std::time::Duration;

use serde::{Deserialize, Serialize};
pub use crate::entity_zero_components::*;

#[derive(Component, Debug, Deserialize, Serialize, Clone, Copy)]
pub struct Directionable;

#[derive(Component, Clone, Copy, Default)]
#[require(Transform)]
pub struct CameraTarget;


#[derive(Component, Debug, Default, Copy, Clone)]
pub struct ExcludedFromAutoRenamer;

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, Hash, PartialEq, )]
/// this component shouldn't be added preemptively to trees, only when their state is altered/differs from generation state
pub struct Persisted;

#[derive(Component, Debug, MapEntities, Clone)]
pub struct SourceDest {
    #[entities] pub source: Entity, #[entities]pub destination: Entity,
}



#[derive(Component, Debug, Deserialize, Serialize, Copy, Clone)]
pub struct Health(pub f32); //SOLO PARA ENEMIGOS ULTRA BÁSICOS SIN CUERPO (GRUNTS IRRECLUTABLES PARA FARMEAR XP O LOOT)

#[derive(Component, Debug, Default, Deserialize, Serialize, Copy, Clone)]
pub struct Dead;

#[derive(Component, Debug, Default, Deserialize, Serialize, Copy, Clone)]
pub struct DespawnOnDeath;

#[derive(Debug, Copy, Clone, Message)]
pub struct HealthDamage {
    pub entity: Entity, pub amount: f32,
}

#[derive(Component, Debug, Clone)]
pub struct PhysicallyImmune();

#[derive(Component, Debug, Clone)]
pub struct MagicallyInvulnerable();


#[derive(Debug, Default, Deserialize, Serialize, Clone)]
pub enum FunctionType {
    #[default]
    LinealOneToZero,
    LinealZeroToOne,
    Curve(Spline<f32, f32>),
}

#[derive(Debug, Default, Component, Deserialize, Serialize, Clone)]
//ES FINITO PERO ES MEJOR, SIMPLEMENTE PONES UNA DURACIÓN ASTRONÓMICA PARA EL TIMER Y PODES SEGUIR USANDO CURVAS BEZIER, CON INFINITO NO SE PUEDE USAR NINGUNA CURVA BEZIER
pub struct TimeBasedMultiplier {
    pub timer: Timer,
    pub function: FunctionType,
}
impl TimeBasedMultiplier {
    pub fn new(timer: Timer, spline: Spline<f32, f32>) -> Self {
        Self {
            timer,
            function: FunctionType::Curve(spline),
        }
    }
    /// A typical drug blood concentration falloff curve: rapid rise, peak, then slow falloff to zero.
    pub fn drug_curve(duration: Duration) -> Self {
        let keys = vec![
            Key::new(0.0, 0.0, Interpolation::Bezier(0.2)), // Start at 0, quick rise
            Key::new(0.1, 1.0, Interpolation::Bezier(0.8)), // Peak quickly
            Key::new(0.5, 0.7, Interpolation::Bezier(0.5)), // Begin to fall
            Key::new(1.0, 0.0, Interpolation::Bezier(0.2)), // End at 0
        ];
        Self {
            function: FunctionType::Curve(Spline::from_vec(keys)),
            timer: Timer::new(duration, TimerMode::Once),
        }
    }
    pub fn linear_wean(duration: Duration) -> Self {
        let keys = vec![
            Key::new(0.0, 1.0, Interpolation::Linear), // Start at 1
            Key::new(1.0, 0.0, Interpolation::Linear), // End at 0
        ];
        Self {
            function: FunctionType::Curve(Spline::from_vec(keys)),
            timer: Timer::new(duration, TimerMode::Once),
        }
    }
    pub fn zero_on_finish_one(duration: Duration) -> Self {
        Self {
            function: FunctionType::LinealZeroToOne,
            timer: Timer::new(duration, TimerMode::Once),
        }
    }
    pub fn one_on_finish_zero(duration: Duration) -> Self {
        Self {
            function: FunctionType::LinealOneToZero,
            timer: Timer::new(duration, TimerMode::Once),
        }
    }
    pub fn sample(&self) -> f32 {
        if self.timer.is_finished() {
            match self.function {
                FunctionType::LinealOneToZero => 0.0,
                FunctionType::LinealZeroToOne => 1.0,
                FunctionType::Curve(ref spline) => match spline.clamped_sample(1.0) {
                    Some(value) => value,
                    None => {
                        error!("Failed to sample spline at the end (1.0)");
                        1.0
                    }
                },
            }
        } else {
            match self.function {
                FunctionType::LinealOneToZero => 1.0,
                FunctionType::LinealZeroToOne => 0.0,
                FunctionType::Curve(ref spline) => {
                    let passed_time_ratio =
                        self.timer.elapsed_secs() / self.timer.duration().as_secs_f32();
                    match spline.clamped_sample(passed_time_ratio) {
                        Some(value) => value,
                        None => {
                            error!("Failed to sample spline at ratio {}", passed_time_ratio);
                            0.0
                        }
                    }
                }
            }
        }
    }
}
#[derive(Component, Debug, Default, Clone, PartialEq)]
pub struct TickMultFactors(pub Vec<TickMultFactor>);

impl TickMultFactors {
    pub fn new<I, T>(factors: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: Into<TickMultFactor>,
    {
        Self(factors.into_iter().map(Into::into).collect())
    }
}

#[derive(Component, Debug, Default, Clone, Copy, PartialEq)]
pub struct TickMultFactor(f32);

impl TickMultFactor {
    pub fn new(value: f32) -> Self {
        Self(value.max(0.0))
    }
    pub fn value(&self) -> f32 {
        self.0
    }
}
#[derive(Component, Debug, Default, Deserialize, Serialize, Copy, Clone)]
pub struct CloneTemplChildren;
