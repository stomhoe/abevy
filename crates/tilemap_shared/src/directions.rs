use strum_macros::{AsRefStr, Display};
use bevy::prelude::*;
use rand::RngExt;
use serde::{Deserialize, Serialize};

#[allow(unused_parens)]
#[derive(Component, Debug, Deserialize, Serialize, Default, AsRefStr, Display, Eq, PartialEq, Hash, Clone, Copy)]
#[strum(serialize_all = "lowercase")]
pub enum CardinalDirection {
    #[default]
    South,
    West,
    East,
    North,
}
impl CardinalDirection {
    pub fn from_dir_vec(dir: IVec2) -> CardinalDirection {
        let dir = dir.clamp(IVec2::NEG_ONE, IVec2::ONE);
        match dir {
            IVec2 { x: -1, y: 0 } => CardinalDirection::West,
            IVec2 { x: 1, y: 0 } => CardinalDirection::East,
            IVec2 { x: 0, y: 1 } => CardinalDirection::North,
            _ => CardinalDirection::South,
        }
    }
    pub fn next_clockwise(&self) -> CardinalDirection {
        match self {
            CardinalDirection::South => CardinalDirection::West,
            CardinalDirection::West => CardinalDirection::North,
            CardinalDirection::North => CardinalDirection::East,
            CardinalDirection::East => CardinalDirection::South,
        }
    }
    pub fn opposite_dir(&self) -> CardinalDirection {
        match self {
            CardinalDirection::South => CardinalDirection::North,
            CardinalDirection::West => CardinalDirection::East,
            CardinalDirection::North => CardinalDirection::South,
            CardinalDirection::East => CardinalDirection::West,
        }
    }
    pub fn to_dir_vec(&self) -> IVec2 {
        match self {
            CardinalDirection::South => IVec2::new(0, -1),
            CardinalDirection::West => IVec2::new(-1, 0),
            CardinalDirection::North => IVec2::new(0, 1),
            CardinalDirection::East => IVec2::new(1, 0),
        }
    }
    pub fn rotation_angle(&self) -> f32 {
        match self {
            CardinalDirection::South => 0.0,
            CardinalDirection::West => std::f32::consts::FRAC_PI_2,
            CardinalDirection::North => std::f32::consts::PI,
            CardinalDirection::East => -std::f32::consts::FRAC_PI_2,
        }
    }
    pub fn sprite_sheet_row(&self) -> usize {
        match self {
            CardinalDirection::South => 0,
            CardinalDirection::North => 1,
            CardinalDirection::West => 2,
            CardinalDirection::East => 3,
        }
    }
    pub fn random(rng: &mut impl rand::Rng) -> Self {
        match rng.random_range(0..4) {
            0 => CardinalDirection::South,
            1 => CardinalDirection::West,
            2 => CardinalDirection::North,
            _ => CardinalDirection::East,
        }
    }
}
impl From<u8> for CardinalDirection {
    fn from(value: u8) -> Self {
        match value {
            0 => CardinalDirection::South,
            1 => CardinalDirection::West,
            2 => CardinalDirection::East,
            3 => CardinalDirection::North,
            _ => CardinalDirection::South, // unreachable, but for completeness
        }
    }
}
impl From<&str> for CardinalDirection {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "south" | "down" | "sur" | "s" => CardinalDirection::South,
            "west" | "left" | "lef" | "w" => CardinalDirection::West,
            "east" | "right" | "rig" | "e" => CardinalDirection::East,
            "north" | "up" | "n" | "nor" => CardinalDirection::North,
            _ => CardinalDirection::South,
        }
    }
}
impl From<String> for CardinalDirection {
    fn from(s: String) -> Self { CardinalDirection::from(s.as_str()) }
}


#[derive(Component, Debug, Deserialize, Serialize, Eq, PartialEq, Hash, Copy, Clone)]
pub enum DiagonalCardinalDirection {
    North,
    South,
    East,
    West,
    NorthEast,
    NorthWest,
    SouthEast,
    SouthWest,
}
impl DiagonalCardinalDirection {
    pub const ALL_DIRS: [Self; 8] = [
        Self::North,
        Self::South,
        Self::East,
        Self::West,
        Self::NorthEast,
        Self::NorthWest,
        Self::SouthEast,
        Self::SouthWest,
    ];

    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "north" | "up" | "n" => Some(Self::North),
            "south" | "down" | "s" => Some(Self::South),
            "east" | "right" | "e" => Some(Self::East),
            "west" | "left" | "w" => Some(Self::West),
            "northeast" | "north_east" | "north-east" | "ne" => Some(Self::NorthEast),
            "northwest" | "north_west" | "north-west" | "nw" => Some(Self::NorthWest),
            "southeast" | "south_east" | "south-east" | "se" => Some(Self::SouthEast),
            "southwest" | "south_west" | "south-west" | "sw" => Some(Self::SouthWest),
            _ => None,
        }
    }
}
