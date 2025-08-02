use std::{default, time::Duration};

#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
use serde::{Deserialize, Serialize};
use splines::{Interpolation, Key, Spline};
use superstate::SuperstateInfo;
use rand::Rng;

use crate::game::being::sprite::animation_constants::*;


#[derive(Component, Debug, )]
pub struct SourceDest{
    pub source: Entity,
    pub destination: Entity,
}



#[derive(Component, Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub struct Nid(u64);

impl Nid {
    pub fn new(nid: u64) -> Self {Self(nid)}
    pub fn nid(&self) -> u64 { self.0 }
}

#[derive(Component, Debug,)]
pub struct Bullet();

#[derive(Component, Debug,)]
pub struct Health(pub i32,);//SOLO PARA ENEMIGOS ULTRA BÁSICOS SIN CUERPO (GRUNTS IRRECLUTABLES PARA FARMEAR XP O LOOT)

#[derive(Component, Debug,)]
pub struct PhysicallyImmune();

#[derive(Component, Debug,)]
pub struct MagicallyInvulnerable();

#[derive(Component, Debug, Default, Deserialize, Serialize, )]
pub struct ImgPathHolder(pub String);


#[derive(Component, Debug, )]
pub struct DimensionRef(pub Entity);


#[derive(Component, Debug, Default, Deserialize, Serialize, )]
pub struct YSort { z: f32, }

pub fn y_sort(
    mut q: Query<(&mut Transform, &YSort)>,
) {
    for (mut tf, ysort) in q.iter_mut() {
        tf.translation.z = ysort.z-(1.0f32 / (1.0f32 + (2.0f32.powf(-0.01*tf.translation.y))));
    }
}


#[allow(unused_parens, dead_code)]
#[derive(Component, Debug, Default, Deserialize, Serialize, )]
pub struct Description(pub String);
impl Description {
    pub fn new<S: Into<String>>(id: S) -> Self {Self (id.into())}
    pub fn id(&self) -> &String {&self.0}
}
#[allow(unused_parens, )]
#[derive(Component, Debug, Deserialize, Serialize, )]
pub enum FacingDirection { Down, Left, Right, Up, }
impl FacingDirection {
    pub fn as_suffix(&self) -> &str {
        match self {
            FacingDirection::Down => DOWN, FacingDirection::Left => LEFT,
            FacingDirection::Right => RIGHT, FacingDirection::Up => UP,
        }
    }
}
impl std::fmt::Display for FacingDirection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            FacingDirection::Down => "down", FacingDirection::Left => "left",
            FacingDirection::Right => "right", FacingDirection::Up => "up",
        };
        write!(f, "{}", s)
    }
}
impl FacingDirection {
    pub fn random() -> Self {
        let mut rng = rand::rng();
        match rng.random_range(0..4) {
            0 => FacingDirection::Down, 1 => FacingDirection::Left, 2 => FacingDirection::Right, _ => FacingDirection::Up,
        }
    }
}
impl Default for FacingDirection {fn default() -> Self {FacingDirection::random()}}


#[derive( Debug, Default, Deserialize, Serialize, Clone, )]
enum FunctionType {#[default] OneOnFinishZero, ZeroOnFinishOne, Curve(Spline<f32, f32>),}

#[derive(Debug, Default, Component, Deserialize, Serialize, Clone, )]
//ES FINITO PERO ES MEJOR, SIMPLEMENTE PONES UNA DURACIÓN ASTRONÓMICA PARA EL TIMER Y PODES SEGUIR USANDO CURVAS BEZIER, CON INFINITO NO SE PUEDE
pub struct TimeBasedMultiplier { pub timer: Timer, pub function: FunctionType, }
impl TimeBasedMultiplier {
    pub fn new(timer: Timer, spline: Spline<f32, f32>) -> Self {
        Self { timer, function: FunctionType::Curve(spline) }
    }
    /// A typical drug blood concentration falloff curve: rapid rise, peak, then slow falloff to zero.
    pub fn drug_curve(duration: Duration) -> Self {
        let keys = vec![
            Key::new(0.0, 0.0, Interpolation::Bezier(0.2)),   // Start at 0, quick rise
            Key::new(0.1, 1.0, Interpolation::Bezier(0.8)),   // Peak quickly
            Key::new(0.5, 0.7, Interpolation::Bezier(0.5)),   // Begin to fall
            Key::new(1.0, 0.0, Interpolation::Bezier(0.2)),   // End at 0
        ];
        Self { function: FunctionType::Curve(Spline::from_vec(keys)), timer: Timer::new(duration, TimerMode::Once) }
    }
    pub fn linear_wean(duration: Duration) -> Self {
        let keys = vec![
            Key::new(0.0, 1.0, Interpolation::Linear), // Start at 1
            Key::new(1.0, 0.0, Interpolation::Linear), // End at 0
        ];
        Self { function: FunctionType::Curve(Spline::from_vec(keys)), timer: Timer::new(duration, TimerMode::Once) }
    }
    pub fn zero_on_finish_one(duration: Duration) -> Self {
        Self { 
            function: FunctionType::ZeroOnFinishOne, 
            timer: Timer::new(duration, TimerMode::Once) 
        }
    }
    pub fn one_on_finish_zero(duration: Duration) -> Self {
        Self { 
            function: FunctionType::OneOnFinishZero, 
            timer: Timer::new(duration, TimerMode::Once) 
        }
    }
    pub fn sample(&self) -> f32 {
        if self.timer.finished() {
            match self.function {
                FunctionType::OneOnFinishZero => 0.0,
                FunctionType::ZeroOnFinishOne => 1.0,
                FunctionType::Curve(ref spline) => {
                    match spline.clamped_sample(1.0) {
                        Some(value) => value,
                        None => {
                            error!("Failed to sample spline at the end (1.0)");
                            1.0
                        }
                    }
                }
            }
        } else {
            match self.function {
                FunctionType::OneOnFinishZero => 1.0,
                FunctionType::ZeroOnFinishOne => 0.0,
                FunctionType::Curve(ref spline) => {
                    let passed_time_ratio = self.timer.elapsed_secs() / self.timer.duration().as_secs_f32();
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

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, )]
pub struct LocalCpu;
