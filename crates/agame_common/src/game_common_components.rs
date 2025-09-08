
use bevy::{platform::collections::{HashMap, HashSet}, prelude::*};
use bevy_replicon::prelude::Replicated;
use common::{common_components::{AssetScoped, EntityPrefix, SessionScoped, StrId}, common_types::FixedStr};
use rand::Rng;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use tilemap_shared::{AaGlobalGenSettings, GlobalTilePos};
use std::time::Duration;
#[allow(unused_imports)] use bevy::prelude::*;
use splines::{Interpolation, Key, Spline};
use strum_macros::{AsRefStr, Display, };
use bevy_inspector_egui::{egui, inspector_egui_impls::{InspectorPrimitive}, reflect_inspector::InspectorUi};

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, Eq, PartialEq, Hash, Copy, Reflect)]
pub struct MyZ(pub i32);
impl MyZ {
    pub fn as_float(&self) -> f32 { self.0 as f32 * Self::Z_MULTIPLIER }
    pub const Z_MULTIPLIER: f32 = 1e-5;
}

#[allow(unused_parens, dead_code)]
#[derive(Component, Debug, Default, Deserialize, Serialize, Reflect)]
pub struct Description(pub String);

#[derive(Component, Debug, Deserialize, Serialize, Copy, Clone, Reflect)]
pub struct SearchingForSuitablePos{ pub studied_op_ent: Entity, }

#[derive(Component, Debug, Deserialize, Serialize, Clone, Copy)]
pub struct Directionable;

#[allow(unused_parens, )]
#[derive(Component, Debug, Deserialize, Serialize, Default, AsRefStr, Display, Reflect, )]
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

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, Hash, PartialEq, Reflect)]
pub struct ClonedSpawned(pub Vec<Entity>);

#[derive(Component, Debug, Deserialize, Serialize, Clone)]
pub struct ClonedSpawnedAsChildren(pub Vec<Entity>);



#[derive(Component, Debug, Clone, Deserialize, Serialize, Reflect, Copy, PartialEq, Eq, Hash)]
pub struct EntityZeroRef(pub Entity);



#[derive(Component, Debug, Default, Deserialize, Serialize, Reflect, Clone, Copy, )]
pub struct YSortOrigin(pub f32);//TAL VEZ ES BUENA IDEA PONERLE ESTO OBLIGATORIAMENTE A TODOS LOS SPRITES, ASÍ TODOS AUMENTAN O DISMINUYEN CONJUNTAMENTE DE Z
impl YSortOrigin {
    pub const Y_SORT_DIV: f32 = 1e-7;
}


#[derive(Debug, Default, Deserialize, Serialize, Clone, PartialEq, Eq, Hash, Reflect, )]
pub struct Category(StrId);

impl Category {
    pub fn new<S: AsRef<str>>(id: S) -> Self {
        if id.as_ref().len() > Self::STR_SIZE {
            warn!("Category str too long: {}, omitting chars beyond i={} -> {}", id.as_ref(), (Self::STR_SIZE - 1), &id.as_ref()[..Self::STR_SIZE]);
        }

        Self(StrId::new(id))
    }

    const STR_SIZE: usize = 32;
}
impl std::fmt::Display for Category {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl InspectorPrimitive for Category {
    fn ui(
        &mut self,
        ui: &mut egui::Ui,
        _: &dyn std::any::Any,
        _: egui::Id,
        _: InspectorUi<'_, '_>,
    ) -> bool {
        let mut s = self.0.as_str().to_string();
        let mut changed = false;
        if ui.text_edit_singleline(&mut s).changed() {
            *self = Category::new(&s);
            changed = true;
        }
        changed
    }

    fn ui_readonly(
        &self,
        ui: &mut egui::Ui,
        _: &dyn std::any::Any,
        _: egui::Id,
        _: InspectorUi<'_, '_>,
    ) {
        ui.label(self.0.as_str());
    }
}

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, Reflect)]
pub struct Categories(pub HashSet<Category>);

impl Categories {
    pub fn new<S: AsRef<str>>(ids: impl IntoIterator<Item = S>) -> Self {
        let set = ids.into_iter().map(Category::new).collect();
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

