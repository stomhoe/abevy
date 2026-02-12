macro_rules! impl_position_conversions {
    ($t:ty) => {
        impl Into<IVec2> for $t {
            fn into(self) -> IVec2 {
                self.0
            }
        }
        impl From<IVec2> for $t {
            fn from(ivec2: IVec2) -> Self {
                Self(ivec2)
            }
        }
    };
}

macro_rules! impl_position_ops {
    ($t:ty) => {
        impl std::ops::Add for $t {
            type Output = Self;
            fn add(self, other: Self) -> Self {
                Self(self.0 + other.0)
            }
        }
        impl std::ops::Sub for $t {
            type Output = Self;
            fn sub(self, other: Self) -> Self {
                Self(self.0 - other.0)
            }
        }
        impl std::ops::Add<IVec2> for $t {
            type Output = Self;
            fn add(self, other: IVec2) -> Self {
                Self(self.0 + other)
            }
        }
    };
}

macro_rules! impl_display_debug {
    ($t:ty, $display_name:expr, $debug_name:expr) => {
        impl std::fmt::Display for $t {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}({}, {})", $display_name, self.0.x, self.0.y)
            }
        }
        impl std::fmt::Debug for $t {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}({}, {})", $debug_name, self.0.x, self.0.y)
            }
        }
    };
}

macro_rules! impl_basic_funcs {
    ($t:ty) => {
        impl $t {
            pub const fn new(x: i32, y: i32) -> Self {
                Self(IVec2::new(x, y))
            }
            pub const fn splat(value: i32) -> Self {
                Self(IVec2::splat(value))
            }
            pub fn distance(&self, other: &Self) -> f32 {
                let dx = self.0.x - other.0.x;
                let dy = self.0.y - other.0.y;
                ((dx * dx + dy * dy) as f32).sqrt()
            }
            pub const fn distance_squared(&self, other: &Self) -> u64 {
                let dx = self.0.x - other.0.x;
                let dy = self.0.y - other.0.y;
                (dx * dx + dy * dy) as u64
            }
            pub const fn element_product(&self) -> i64 {
                self.0.x as i64 * self.0.y as i64
            }
            pub const fn area(&self) -> u64 {
                self.element_product().abs() as u64
            }
            pub const fn area_usize(&self) -> usize {
                self.element_product().abs() as usize
            }
        }
    };
}

macro_rules! impl_hashed_position {
    ($t:ty) => {
        impl HashablePosVec for $t {
            fn x(&self) -> i32 {
                self.0.x
            }
            fn y(&self) -> i32 {
                self.0.y
            }
        }
    };
}

macro_rules! impl_adjacent_position_methods {
    () => {
        pub fn adjacent_north(&self) -> Self {
            Self(self.0.wrapping_add(IVec2::new(0, 1)))
        }
        pub fn adjacent_south(&self) -> Self {
            Self(self.0.wrapping_add(IVec2::new(0, -1)))
        }
        pub fn adjacent_east(&self) -> Self {
            Self(self.0.wrapping_add(IVec2::new(1, 0)))
        }
        pub fn adjacent_west(&self) -> Self {
            Self(self.0.wrapping_add(IVec2::new(-1, 0)))
        }
        pub fn adjacent_northeast(&self) -> Self {
            Self(self.0.wrapping_add(IVec2::new(1, 1)))
        }
        pub fn adjacent_northwest(&self) -> Self {
            Self(self.0.wrapping_add(IVec2::new(-1, 1)))
        }
        pub fn adjacent_southeast(&self) -> Self {
            Self(self.0.wrapping_add(IVec2::new(1, -1)))
        }
        pub fn adjacent_southwest(&self) -> Self {
            Self(self.0.wrapping_add(IVec2::new(-1, -1)))
        }
        pub fn adjacent_dir(&self, dir: DiagonalCardinalDirection) -> Self {
            match dir {
                DiagonalCardinalDirection::North => self.adjacent_north(),
                DiagonalCardinalDirection::South => self.adjacent_south(),
                DiagonalCardinalDirection::East => self.adjacent_east(),
                DiagonalCardinalDirection::West => self.adjacent_west(),
                DiagonalCardinalDirection::NorthEast => self.adjacent_northeast(),
                DiagonalCardinalDirection::NorthWest => self.adjacent_northwest(),
                DiagonalCardinalDirection::SouthEast => self.adjacent_southeast(),
                DiagonalCardinalDirection::SouthWest => self.adjacent_southwest(),
            }
        }
    };
}
