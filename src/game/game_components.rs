use std::time::Duration;

use bevy::{math::U16Vec2, platform::collections::{HashMap, HashSet}};
#[allow(unused_imports)] use bevy::prelude::*;
use bevy_ecs_tilemap::tiles::TileTextureIndex;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
use serde::{Deserialize, Serialize};
use splines::{Interpolation, Key, Spline};
use std::fmt::Display;

use crate::{common::common_components::{HashId, HashIdIndexMap, HashIdMap}, game::{being::sprite::animation_constants::*, game_resources::ImageSizeMap}};


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

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone)]
pub struct ImagePathHolder(String);
impl ImagePathHolder {
    pub fn new<S: AsRef<str>>(path: S) -> Result<Self, BevyError> {
        let img_path = format!("assets/{}", path.as_ref());
        if !std::path::Path::new(&img_path).exists() {
            let err = BevyError::from(format!("Image path does not exist: {}", img_path));
            error!(target: "image_loading", "{}", err);
            return Err(err);
        }
        Ok(Self(path.as_ref().to_string()))
    }
    pub fn path(&self) -> &str { &self.0 }
}
impl Display for ImagePathHolder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { write!(f, "{}", self.0) }
}
impl From<ImagePathHolder> for bevy::asset::AssetPath<'_> {
    fn from(holder: ImagePathHolder) -> Self { bevy::asset::AssetPath::from(holder.0) }
}


#[derive(Component, Debug, Clone, Default, Hash, PartialEq, Eq, Reflect)]
pub struct ImageHolder(pub Handle<Image>);
impl ImageHolder {

    pub fn new<S: AsRef<str>>(asset_server: &AssetServer, path: S) -> Result<Self, BevyError> {
        let img_path = format!("assets/{}", path.as_ref());
        if !std::path::Path::new(&img_path).exists() {
            let err = BevyError::from(format!("Image path does not exist: {}", img_path));
            error!(target: "image_loading", "{}", err);
            return Err(err);
        }
        Ok(Self(asset_server.load(path.as_ref())))
    }
}


#[derive(Component, Debug, Clone, Default, )]
pub struct ImageHolderMap(pub HashIdIndexMap<Handle<Image>>);
impl ImageHolderMap {
    pub fn from_paths(
        asset_server: &AssetServer, 
        img_paths: HashMap<String, String>, 
    ) -> Result<Self, BevyError> {
        let mut map = HashIdIndexMap::default();
        for (key, path) in img_paths {
            let image_holder = ImageHolder::new(asset_server, &path)?;
            map.insert(key, image_holder.0);
        }
        Ok(Self(map))
    }
    pub fn first_handle(&self) -> Handle<Image> {
        self.0.first().cloned().unwrap_or_else(|| Handle::default())
    }
   
}



#[derive(Component, Debug, Deserialize, Serialize, Clone)]
pub struct ClonedSpawned(pub Vec<Entity>);

#[derive(Component, Debug, Deserialize, Serialize, Clone)]
pub struct ClonedSpawnedAsChildren(pub Vec<Entity>);



#[derive(Component, Debug, Clone, Deserialize, Serialize)]
pub struct OriginalEntity(pub Entity);



#[derive(Component, Debug, Default, Deserialize, Serialize, )]
pub struct YSortOrigin (pub f32);

// pub fn y_sort(
//     tmap: Query<(&Transform, )>,//add this tranform to tile transform
//     tile: Query<(&ChildOf, &Transform, &YSortOrigin)>,
//     being: Query<(&Transform, ), ( With<Being>, Changed<Transform>, )>,
// ) -> Result {
   
// }


#[allow(unused_parens, dead_code)]
#[derive(Component, Debug, Default, Deserialize, Serialize, Reflect)]
pub struct Description(pub String);

#[allow(unused_parens, )]
#[derive(Component, Debug, Deserialize, Serialize, Default )]
pub enum FacingDirection { #[default] Down, Left, Right, Up, }//PARA CAMBIAR ALEATORIAMENTE AL SPAWNEAR, HACER UN SISTEMA PARA BEINGS ADDED Q USE BEVY_RAND
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

//CAN BE A BOT RUN IN THE CLIENT'S COMPUTER (P.EJ PATHFINDING)
#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, )]
pub struct ControlledByBot;

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, PartialEq)]
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

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, Copy, PartialEq)]
pub struct TickMultFactor(f32);

impl TickMultFactor {
    pub fn new(value: f32) -> Self { Self(value.max(0.0)) }
    pub fn value(&self) -> f32 { self.0 }
}
