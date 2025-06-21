#[allow(unused_imports)] use bevy::prelude::*;

use crate::game::time::time_types::*;

#[derive(Resource, PartialEq, PartialOrd, Copy, Clone, Debug)]
pub struct SimTimeScale(pub f32);
impl Default for SimTimeScale {fn default() -> Self {Self (100.0)}}

#[derive(Resource, PartialEq, PartialOrd, Debug, Clone, Copy)]
pub struct InGameTiming {
    time_scale_rela2irl: f32,
    days_per_year: Days,
}
impl InGameTiming {
    pub fn adjust_time_scale_rela_to_rl(&mut self, new_time_scale: f32) {
        self.time_scale_rela2irl = new_time_scale;
    }
    pub fn adjust_days_per_year(&mut self, new_days_per_year: Days) {
        self.days_per_year = new_days_per_year;
    }

    pub fn time_scale_rela2irl(&self) -> f32 {self.time_scale_rela2irl }
    pub fn days_per_year(&self) -> Days {self.days_per_year }
    pub fn days_per_season(&self) -> Days { self.days_per_year/4 }
}

impl Default for InGameTiming {
    fn default() -> Self {
        //48: media hora real por día de juego
        Self { time_scale_rela2irl: 48.0, days_per_year: Days(200) }
    }
}



// Implement division for Days


#[derive(Resource, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
pub struct CurrYear(pub Years);
impl Default for CurrYear {fn default() -> Self {Self (Years(1))}}

impl std::fmt::Display for CurrYear {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Year {}", self.0)
    }
}

#[derive(Resource, Default, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
pub struct CurrSeason(pub Season);

impl std::fmt::Display for CurrSeason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

#[derive(Resource, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
pub struct CurrDay(pub Days);
impl Default for CurrDay {fn default() -> Self {Self (Days(1))}}

impl std::fmt::Display for CurrDay {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Day {}", self.0)
    }
}

#[derive(Resource, Default, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
pub struct CurrHour(pub Hours);

impl std::fmt::Display for CurrHour {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:02}h", self.0.0)
    }
}

#[derive(Resource, Default, PartialEq, PartialOrd, Clone, Copy, Debug)]
pub struct CurrMin(pub Minutes);

impl std::fmt::Display for CurrMin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:02.0}m", self.0.0)
    }
}

//USAR Res<tIMER> PARA COOLDOWNS DE ATAQUES??
#[derive(Resource, Default, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
pub struct CurrSec(pub u8);


impl std::fmt::Display for CurrSec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:02}s", self.0)
    }
}

// New Hour and Minute structs
