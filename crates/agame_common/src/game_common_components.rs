use bevy::{ecs::entity::MapEntities, platform::collections::HashMap, prelude::*};
use common::common_components::*;

use splines::{Interpolation, Key, Spline};
use std::hash::Hash;
use std::time::Duration;

use crate::game_common_seris::NormalDistSeri;
use serde::{Deserialize, Serialize};
pub use crate::entity_zero_components::*;

#[derive(Component, Debug, Deserialize, Serialize, Clone, Copy)]
pub struct Directionable;

#[derive(Component, Debug, Reflect, Clone)]
/// runs when simulation is running or not
pub struct DespawnTimer(pub Timer);
impl DespawnTimer {
    pub fn new(seconds: f32) -> Self {
        Self(Timer::from_seconds(seconds, TimerMode::Once))
    }
}

#[derive(Component, Debug, Reflect, Clone)]
/// runs only when simulation is running
pub struct SimRunningDespawnTimer(pub Timer);
impl SimRunningDespawnTimer {
    pub fn new(seconds: f32) -> Self {
        Self(Timer::from_seconds(seconds, TimerMode::Once))
    }
}

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, Hash, PartialEq, Reflect)]
/// this component shouldn't be added preemptively to trees, only when their state is altered/differs from generation state
pub struct Persisted;

#[derive(Component, Debug, MapEntities, Clone)]
pub struct SourceDest {
    #[entities] pub source: Entity, #[entities]pub destination: Entity,
}

#[derive(Component, Debug, Clone, Default, Deserialize, Serialize)]
pub struct ArgsDict(HashMap<StrId, Vec<String>>);
impl ArgsDict {
    pub fn with_capacity(capacity: usize) -> Self {
        Self(HashMap::with_capacity(capacity))
    }
    pub fn insert<T: Into<StrId>, U: Into<String>>(&mut self, key: T, val: Vec<U>) {
        let val_strs: Vec<String> = val.into_iter().map(|v| v.into()).collect();
        self.0.insert(key.into(), val_strs);
    }
    pub fn get<T: Into<StrId>>(&self, key: T) -> Option<&Vec<String>> {
        self.0.get(&key.into())
    }
    pub fn parse_arg<T: std::str::FromStr + Clone, K: Into<StrId>>(&self, key: K, default: T) -> T {
        self.get(key)
            .and_then(|v| v.first())
            .and_then(|s| s.parse::<T>().ok())
            .unwrap_or(default)
    }

    pub fn parse_opt_arg<T: std::str::FromStr, K: Into<StrId>>(&self, key: K) -> Option<T> {
        self.get(key)
            .and_then(|v| v.first())
            .and_then(|s| s.parse::<T>().ok())
    }
}

#[derive(Component, Debug, Clone)]
pub struct Health(pub f32); //SOLO PARA ENEMIGOS ULTRA BÁSICOS SIN CUERPO (GRUNTS IRRECLUTABLES PARA FARMEAR XP O LOOT)

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

#[derive(Component, Default, Debug, Clone, Deserialize, Serialize)]
pub struct CappedNormalDist {
    pub min: f32,
    pub max: f32,
    pub mean: f32,
    pub std_dev: f32,
}
impl CappedNormalDist {
    pub fn new(min: f32, max: f32, mean: f32, std_dev: f32) -> Self {
        let mut min = min.max(0.01);
        let mut max = max.max(0.01);
        let mut mean = mean;
        let std_dev = std_dev.max(0.01);
        if min > max {
            error!(
                "CappedNormalDist: min ({}) is greater than max ({}). Swapping values.",
                min, max
            );
            std::mem::swap(&mut min, &mut max);
        }
        if mean < min || mean > max {
            error!(
                "CappedNormalDist: mean ({}) is outside the range [{}, {}]. Clamping to range.",
                mean, min, max
            );
            mean = mean.clamp(min, max);
        }
        let range = max - min;
        if std_dev > range {
            error!(
                "CappedNormalDist: std_dev ({}) is larger than the range ({} - {} = {}). \
                 Most samples will be clamped.",
                std_dev, max, min, range
            );
        }
        Self {min, max, mean, std_dev,}
    }

    pub fn sample(&self, rng: &mut impl rand::Rng) -> f32 {
        use rand_distr::{Normal, Distribution};

        let normal = match Normal::new(self.mean, self.std_dev) {
            Ok(dist) => dist,
            Err(e) => {
                error!(
                    "CappedNormalDist: Failed to create normal distribution with mean={}, std_dev={}: {}. \
                     Using fallback distribution.",
                    self.mean, self.std_dev, e
                );
                Normal::new(self.mean, 0.1)
                    .unwrap_or_else(|_| {
                        error!(
                            "CappedNormalDist: Fallback distribution also failed. Returning mean value."
                        );
                        Normal::new(self.mean, 0.01)
                            .expect("Final fallback should always work")
                    })
            }
        };

        normal.sample(rng).clamp(self.min, self.max)
    }
    pub fn from_seri(seri: NormalDistSeri) -> Self {
        Self::new(seri.min, seri.max, seri.mean, seri.std_dev)
    }
}
