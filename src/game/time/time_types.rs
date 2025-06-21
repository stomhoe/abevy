#[allow(unused_imports)] use bevy::prelude::*;
use std::ops::*;
use strum_macros::VariantNames;


#[derive(Default, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
pub struct Years(pub u32);

impl std::fmt::Display for Years {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}y", self.0)
    }
}

impl Add for Years {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}
impl AddAssign for Years {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

impl Sub for Years {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}
impl SubAssign for Years {
    fn sub_assign(&mut self, rhs: Self) {
        self.0 -= rhs.0;
    }
}

#[derive(Default, VariantNames, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
#[strum(serialize_all = "PascalCase")]
pub enum Season {#[default]Spring = 1, Summer, Autumn, Winter,}
impl Season {
    pub fn next(&self) -> Self {
        match self {
            Season::Spring => Season::Summer,
            Season::Summer => Season::Autumn,
            Season::Autumn => Season::Winter,
            Season::Winter => Season::Spring,
        }
    }
    pub fn prev(&self) -> Self {
        match self {
            Season::Spring => Season::Winter,
            Season::Summer => Season::Spring,
            Season::Autumn => Season::Summer,
            Season::Winter => Season::Autumn,
        }
    }
    pub fn distance_to(self, other: Self) -> u8 {
        ((self as i8 - other as i8).abs()) as u8
    }
}

#[derive(Default, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
pub struct Days(pub u32);

impl std::fmt::Display for Days {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}d", self.0)
    }
}

impl Add for Days {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}
impl AddAssign for Days {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

impl Div<u32> for Days {
    type Output = Self;
    fn div(self, rhs: u32) -> Self::Output {
        Self(self.0 / rhs)
    }
}

impl DivAssign<u32> for Days {
    fn div_assign(&mut self, rhs: u32) {
        self.0 /= rhs;
    }
}

impl Sub for Days {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}
impl SubAssign for Days {
    fn sub_assign(&mut self, rhs: Self) {
        self.0 -= rhs.0;
    }
}

#[derive(Default, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
pub struct Hours(pub u32);

impl std::fmt::Display for Hours {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}h", self.0)
    }
}

impl Add for Hours {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}
impl AddAssign for Hours {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

#[derive(Default, PartialEq, PartialOrd, Clone, Copy, Debug)]
pub struct Minutes(pub f32);

impl std::fmt::Display for Minutes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:.0}m", self.0)
    }
}

impl Add for Minutes {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}
impl AddAssign for Minutes {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}








// Hours
impl Sub for Hours {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}
impl SubAssign for Hours {
    fn sub_assign(&mut self, rhs: Self) {
        self.0 -= rhs.0;
    }
}

// Minutes
impl Sub for Minutes {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}
impl SubAssign for Minutes {
    fn sub_assign(&mut self, rhs: Self) {
        self.0 -= rhs.0;
    }
}
