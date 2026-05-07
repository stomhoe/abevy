#[allow(unused_imports)] use bevy::prelude::*;
use rand::Rng;
use rand::RngExt;
use ::tilemap_shared::*;

#[derive(Clone, Copy, Debug, Reflect, PartialEq, Eq, Hash)]
pub enum RoomShape {
    Rectangle,
    Ellipse,
    Trapezoid,
    RegularPolygon,
    Pentacle,
    RandomChamber,
}

impl Default for RoomShape {
    fn default() -> Self {
        Self::Rectangle
    }
}

impl RoomShape {
    pub fn as_str(self) -> &'static str {
        match self {
            RoomShape::Rectangle => "rectangle",
            RoomShape::Ellipse => "ellipse",
            RoomShape::Trapezoid => "trapezoid",
            RoomShape::RegularPolygon => "regular_polygon",
            RoomShape::Pentacle => "pentacle",
            RoomShape::RandomChamber => "random_chamber",
        }
    }

    pub fn can_fit(self, room: &Room, size_config: &RoomSizeConfig) -> bool {
        match self {
            RoomShape::Rectangle | RoomShape::Trapezoid | RoomShape::RegularPolygon => {
                let available_w = (room.w - 2).max(0);
                let available_h = (room.h - 2).max(0);
                available_w >= size_config.width.min && available_h >= size_config.height.min
            }
            RoomShape::Ellipse | RoomShape::Pentacle => {
                let available_w = (room.w - 2).max(0);
                let available_h = (room.h - 2).max(0);
                available_w >= size_config.width.min && available_h >= size_config.height.min
            }
            RoomShape::RandomChamber => {
                let available_diameter = (room.w.min(room.h) - 2).max(0);
                let min_diameter = size_config.width.min.max(size_config.height.min);
                available_diameter >= min_diameter
            }
        }
    }

    pub fn sample_dimensions(self, room: &Room, size_config: &RoomSizeConfig, rng: &mut impl Rng) -> Option<(i32, i32)> {
        match self {
            RoomShape::Rectangle | RoomShape::Trapezoid | RoomShape::RegularPolygon | RoomShape::Ellipse => {
                let available_w = (room.w - 2).max(0);
                let available_h = (room.h - 2).max(0);
                let room_w = size_config.width.sample(rng, available_w)?;
                let room_h = size_config.height.sample(rng, available_h)?;
                Some((room_w, room_h))
            }
            RoomShape::Pentacle => {
                let available_diameter = (room.w.min(room.h) - 2).max(0);
                let diameter = size_config.width.sample(rng, available_diameter)?;
                Some((diameter, diameter))
            }
            RoomShape::RandomChamber => {
                let available_diameter = (room.w.min(room.h) - 2).max(0);
                let min_diameter = size_config.width.min.max(size_config.height.min);
                let max_diameter = size_config
                    .width
                    .max
                    .unwrap_or(available_diameter)
                    .min(size_config.height.max.unwrap_or(available_diameter))
                    .min(available_diameter);

                if available_diameter < min_diameter || max_diameter < min_diameter {
                    return None;
                }

                let diameter = rng.random_range(min_diameter..=max_diameter);
                Some((diameter, diameter))
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Reflect)]
pub struct RoomSizeRange {
    pub min: i32,
    pub max: Option<i32>,
}

#[derive(Clone, Copy, Debug, Reflect)]
pub struct RoomSizeConfig {
    pub width: RoomSizeRange,
    pub height: RoomSizeRange,
}

#[derive(Clone, Copy, Debug, Reflect)]
pub struct RoomSpec {
    pub shape: RoomShape,
    pub w: i32,
    pub h: i32,
}

impl RoomSpec {
    pub fn area(self) -> i32 {
        self.w * self.h
    }

    pub fn into_room(self, x: i32, y: i32) -> Room {
        Room { x, y, w: self.w, h: self.h, shape: self.shape }
    }
}

impl RoomSizeRange {
    pub fn sample(self, rng: &mut impl Rng, available_max: i32) -> Option<i32> {
        let max = self.max.unwrap_or(available_max).min(available_max);
        if available_max < self.min || max < self.min {
            return None;
        }

        Some(rng.random_range(self.min..=max))
    }
}

impl RoomSizeConfig {
    pub fn default_global() -> Self {
        let width_min = 10;
        let height_min = 10;
        let width_max = Some(60);
        let height_max = Some(60);
        Self {
            width: RoomSizeRange { min: width_min, max: width_max },
            height: RoomSizeRange { min: height_min, max: height_max },
        }
    }
}

#[derive(Clone, Copy, Debug, Reflect)]
pub struct Room {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
    pub shape: RoomShape,
}

impl Room {
    pub fn new(x: i32, y: i32, w: i32, h: i32) -> Self {
        Self { x, y, w, h, shape: RoomShape::Rectangle }
    }

    pub fn area(&self) -> i32 {
        self.w * self.h
    }

    pub fn sample_spawn_anchor(
        &self,
        floor_map: &[u8],
        hazard_map: &[bool],
        map_width: usize,
        map_height: usize,
        origin_tile: GlobalTilePos,
        rng: &mut impl Rng,
        floor_none: u8,
    ) -> Option<GlobalTilePos> {
        let min_x = self.x.max(0);
        let min_y = self.y.max(0);
        let max_x = (self.x + self.w).min(map_width as i32);
        let max_y = (self.y + self.h).min(map_height as i32);
        if min_x >= max_x || min_y >= max_y {
            return None;
        }

        let center_x = (min_x + max_x - 1) as f32 * 0.5;
        let center_y = (min_y + max_y - 1) as f32 * 0.5;
        let max_center_distance = ((max_x - min_x - 1) + (max_y - min_y - 1)).max(1) as f32;

        let mut chosen = None;
        let mut total_weight = 0.0f32;
        for y in min_y..max_y {
            for x in min_x..max_x {
                let ux = x as usize;
                let uy = y as usize;
                let idx = uy * map_width + ux;
                if floor_map.get(idx).copied().unwrap_or(floor_none) == floor_none || hazard_map.get(idx).copied().unwrap_or(false) {
                    continue;
                }
                let center_distance = (x as f32 - center_x).abs() + (y as f32 - center_y).abs();
                let center_weight = (max_center_distance - center_distance + 1.0).max(1.0);

                let mut hazard_distance = i32::MAX;
                for hazard_y in min_y..max_y {
                    for hazard_x in min_x..max_x {
                        let hazard_idx = hazard_y as usize * map_width + hazard_x as usize;
                        if !hazard_map.get(hazard_idx).copied().unwrap_or(false) {
                            continue;
                        }
                        let distance = (hazard_x - x).abs() + (hazard_y - y).abs();
                        hazard_distance = hazard_distance.min(distance);
                    }
                }
                let hazard_weight = if hazard_distance == i32::MAX {
                    max_center_distance
                } else {
                    (hazard_distance + 1) as f32
                };

                let weight = center_weight * hazard_weight;
                total_weight += weight;
                if rng.random_range(0.0..total_weight) < weight {
                    chosen = Some(GlobalTilePos::new(origin_tile.x() + x, origin_tile.y() + y));
                }
            }
        }
        chosen
    }
}
