
use bevy::{platform::collections::{HashMap, HashSet}, prelude::*};
use bevy_replicon::prelude::Replicated;
use common::common_components::*;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::time::Duration;
#[allow(unused_imports)] use bevy::prelude::*;
use splines::{Interpolation, Key, Spline};
use strum_macros::{AsRefStr, Display, };
use std::hash::Hash;



#[allow(unused_parens, dead_code)]
#[derive(Component, Debug, Default, Deserialize, Serialize, Reflect)]
pub struct Description(pub String);

#[derive(Component, Debug, Deserialize, Serialize, Copy, Clone, Reflect)]
pub struct SearchingForSuitablePos{ pub filtered_op_ent: Entity, }

#[derive(Component, Debug, Deserialize, Serialize, Clone, Copy)]
pub struct Directionable;

#[derive(Component, Debug, Default, Deserialize, Serialize, Copy, Clone, Reflect)]
pub struct EntityZero;


#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, Hash, PartialEq, Reflect)]
/// this component shouldn't be added preemptively to trees, only when their state is altered/differs from generation state
pub struct Persisted;

#[allow(unused_parens, )]
#[derive(Component, Debug, Deserialize, Serialize, Default, AsRefStr, Display, Reflect, Eq, PartialEq, Hash, Clone, Copy)]
#[strum(serialize_all = "lowercase")]
pub enum FacingDirection { #[default] South, West, East, North, }
impl FacingDirection {
    pub fn next_clockwise(&self) -> FacingDirection {
        match self {
            FacingDirection::South => FacingDirection::West,
            FacingDirection::West => FacingDirection::North,
            FacingDirection::North => FacingDirection::East,
            FacingDirection::East => FacingDirection::South,
        }
    }
    pub fn to_dir_vec(&self) -> IVec2 {
        match self {
            FacingDirection::South => IVec2::new(0, 1),
            FacingDirection::West => IVec2::new(-1, 0),
            FacingDirection::North => IVec2::new(0, -1),
            FacingDirection::East => IVec2::new(1, 0),
        }
    }
}
impl From<u8> for FacingDirection {
    fn from(value: u8) -> Self {
        match value {
            0 => FacingDirection::South,
            1 => FacingDirection::West,
            2 => FacingDirection::East,
            3 => FacingDirection::North,
            _ => FacingDirection::South, // unreachable, but for completeness
        }
    }
}
impl From<&str> for FacingDirection {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "south" | "down" | "sur" | "s" => FacingDirection::South,
            "west" | "left" | "lef" | "w" => FacingDirection::West,
            "east" | "right" | "rig" | "e" => FacingDirection::East,
            "north" | "up"  | "n" => FacingDirection::North,
            _ => FacingDirection::South,
        }
    }
}
impl From<String> for FacingDirection {
    fn from(s: String) -> Self {
        FacingDirection::from(s.as_str())
    }
}


#[derive(Component, Debug, )]
pub struct SourceDest{
    pub source: Entity,
    pub destination: Entity,
}


#[derive(Component, Debug,)]
pub struct Health(pub f32,);//SOLO PARA ENEMIGOS ULTRA BÁSICOS SIN CUERPO (GRUNTS IRRECLUTABLES PARA FARMEAR XP O LOOT)

#[derive(Component, Debug,)]
pub struct PhysicallyImmune();

#[derive(Component, Debug,)]
pub struct MagicallyInvulnerable();

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, Hash, PartialEq, Reflect)]
pub enum VisibilityGameState {
    #[default]
    Inherited,
    Visible,
    Hidden,
}
impl From<Visibility> for VisibilityGameState {
    fn from(vis: Visibility) -> Self {
        match vis {
            Visibility::Inherited => VisibilityGameState::Inherited,
            Visibility::Visible => VisibilityGameState::Visible,
            Visibility::Hidden => VisibilityGameState::Hidden,
        }
    }
}
impl From<VisibilityGameState> for Visibility {
    fn from(rvis: VisibilityGameState) -> Self {
        match rvis {
            VisibilityGameState::Inherited => Visibility::Inherited,
            VisibilityGameState::Visible => Visibility::Visible,
            VisibilityGameState::Hidden => Visibility::Hidden,
        }
    }
}

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, Hash, PartialEq, Reflect)]
pub struct ClonedSpawned(pub Vec<Entity>);

#[derive(Component, Debug, Deserialize, Serialize, Clone)]
pub struct ClonedSpawnedAsChildren(pub Vec<Entity>);



#[derive(Component, Debug, Clone, Deserialize, Serialize, Reflect, Copy, PartialEq, Eq, Hash)]
/// DON'T FORGET TO ADD <DISABLED> TO THE QUERY 
pub struct EntityZeroRef(#[entities] pub Entity);





#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, Reflect)]
pub struct Categories(pub HashSet<Category>);

impl Categories {
    pub fn new<S: AsRef<str>>(ids: impl IntoIterator<Item = S>) -> Self {
        let set = ids.into_iter().map(Category::new_truncated).collect();
        Self(set)
    }
}

#[derive( Debug, Default, Deserialize, Serialize, Clone, )]
pub enum FunctionType {#[default] OneOnFinishZero, ZeroOnFinishOne, Curve(Spline<f32, f32>),}

#[derive(Debug, Default, Component, Deserialize, Serialize, Clone, )]
//ES FINITO PERO ES MEJOR, SIMPLEMENTE PONES UNA DURACIÓN ASTRONÓMICA PARA EL TIMER Y PODES SEGUIR USANDO CURVAS BEZIER, CON INFINITO NO SE PUEDE USAR CURVA BEZIER
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
        if self.timer.is_finished() {
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





#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, PartialEq)]
pub struct TickMultFactors(pub Vec<TickMultFactor>);

impl TickMultFactors {
    pub fn new<I, T>(factors: I) -> Self
    where I: IntoIterator<Item = T>, T: Into<TickMultFactor>,
    {
        Self(factors.into_iter().map(Into::into).collect())
    }
}

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, Copy, PartialEq)]
pub struct TickMultFactor(f32);

impl TickMultFactor {
    pub fn new(value: f32) -> Self { Self(value.max(0.0)) }
    pub fn value(&self) -> f32 { self.0 }
}

