use bevy::prelude::*;
use bevy::platform::collections::HashMap;
use bevy_ecs_tilemap::prelude::*;
use common::common_components::*;
use serde::{Deserialize, Serialize};

use crate::{CardinalDirection, GlobalTilePos, SizeInTiles};

#[derive(Component, Deserialize, TypePath, Clone, Debug, Default)]
pub struct InteractionZoneSeri {
    #[serde(default)]
    pub offset_positions: Vec<(i8, i8)>,
    #[serde(default)]
    pub radius_offset: Vec<(f32, (f32, f32))>,
}
impl InteractionZoneSeri {
    pub fn sentinel() -> Self {
        Self {
            offset_positions: Vec::new(),
            radius_offset: vec![(f32::NAN, (f32::NAN, f32::NAN))],
        }
    }
    pub fn sentinel_melee_interaction_zone() -> Self { Self::sentinel() }
    pub fn sentinel_collision_zone() -> Self { Self::sentinel() }
    pub fn is_sentinel(&self) -> bool {
        self.offset_positions.is_empty()
            && self.radius_offset.len() == 1
            && self.radius_offset[0].0.is_nan()
            && self.radius_offset[0].1.0.is_nan()
            && self.radius_offset[0].1.1.is_nan()
    }
    pub fn default_collision_zone() -> Self {
        Self {
            offset_positions: vec![(0, 0)],
            radius_offset: Vec::new(),
        }
    }
    pub fn default_melee_interaction_zone() -> Self {
        Self {
            offset_positions: vec![(0, 1)],
            radius_offset: Vec::new(),
        }
    }
}

pub fn sentinel_melee_interaction_zone() -> InteractionZoneSeri {
    InteractionZoneSeri::sentinel_melee_interaction_zone()
}
pub fn sentinel_collision_zone() -> InteractionZoneSeri {
    InteractionZoneSeri::sentinel_collision_zone()
}

#[derive(Component, Clone, Deserialize, Serialize, Debug)]
/// interaction positions (offsets relative to the tile's anchor GlobalTilePos)
pub struct InteractionZones(pub HashIdMap<InteractionZone>);
impl InteractionZones {
    pub fn from_seri(map: HashMap<String, InteractionZoneSeri>) -> Self {
        let mut zones = HashIdMap::with_capacity(map.len());
        for (id, seri) in map {
            zones.overwrite(HashId::from(id), InteractionZone::from_seri(seri));
        }
        Self(zones)
    }
    pub fn new_collision_mask_zone_tiles_only(
        rows: &[String],
        size_in_tiles: SizeInTiles,
    ) -> Result<InteractionZone, BevyError> {
        let width = size_in_tiles.inner().x as usize;
        let height = size_in_tiles.inner().y as usize;
        if rows.len() != height {
            return Err(BevyError::from(format!(
                "collision_mask row count ({}) does not match size_in_tiles.y ({})",
                rows.len(),
                height
            )));
        }

        let mut offset_positions = Vec::new();
        for (y, row) in rows.iter().enumerate() {
            let row = row.trim();
            if row.chars().count() != width {
                return Err(BevyError::from(format!(
                    "collision_mask row {} width ({}) does not match size_in_tiles.x ({})",
                    y,
                    row.chars().count(),
                    width
                )));
            }
            for (x, c) in row.chars().enumerate() {
                match c {
                    '0' => {}
                    '1' => {
                        let source_y = (height - 1) - y;
                        offset_positions.push(GlobalTilePos::new(x as i32, source_y as i32));
                    }
                    _ => {
                        return Err(BevyError::from(format!(
                            "collision_mask row {} contains invalid char '{}'; only '0' and '1' are allowed",
                            y, c
                        )));
                    }
                }
            }
        }

        Ok(InteractionZone::new(offset_positions, Vec::new()))
    }
    pub fn is_point_inside_zone(
        &self,
        zone_id: HashId,
        zone_anchor: Vec2,
        direction: CardinalDirection,
        flip: TileFlip,
        point_transf: Vec2,
    ) -> bool {
        let zone = self.0.get(zone_id).ok();
        zone.is_some_and(|zone| zone.is_inside_any(flip, direction, zone_anchor, point_transf))
    }
    pub fn interaction_zones_intersect(
        &self,
        zone_id: HashId,
        other_zone: &InteractionZone,
        anchor_direction: CardinalDirection,
        anchor_transf: Vec2,
        other_direction: CardinalDirection,
        other_anchor_transf: Vec2,
    ) -> bool {
        let Some(zone) = self.0.get(zone_id).ok() else {
            return false;
        };
        zone.intersects_zone(anchor_direction, anchor_transf, other_zone, other_direction, other_anchor_transf)
    }
    pub fn get_collision_mask(&self) -> Option<&InteractionZone> {
        self.0.get(Self::COLLISION).ok()
    }
    pub fn gather_zone_positions_for_hashid<'a>(
        &self,
        zone_id: HashId,
        direction: CardinalDirection,
        anchor_transf: Vec2,
        out: &'a mut Vec<GlobalTilePos>,
    ) -> &'a [GlobalTilePos] {
        let Some(zone) = self.0.get(zone_id).ok() else {
            return out.as_slice();
        };
        zone.gather_zone_positions(direction, anchor_transf, out)
    }
    pub const ENTER: HashId = HashId::hash("enter");
    pub const MELEE_ATTACK: HashId = HashId::hash("melee_attack");
    pub const COLLISION: HashId = HashId::hash("collision_mask");
}

#[derive(Component, Clone, Deserialize, Serialize, Debug)]
pub struct InteractionZone {
    offset_positions: Vec<GlobalTilePos>,
    radius_paired_w_offsets: Vec<(f32, Vec2)>,
    perimeter_size: u32,
}
impl InteractionZone {
    pub fn new(
        offset_positions: Vec<GlobalTilePos>,
        radius_paired_w_offsets: Vec<(f32, Vec2)>,
    ) -> Self {
        let mut zone = Self {
            offset_positions,
            radius_paired_w_offsets,
            perimeter_size: 0,
        };
        zone.perimeter_size = zone.perimeter_in_tiles();
        zone
    }

    pub fn from_seri(seri: InteractionZoneSeri) -> Self {
        let offset_positions = seri
            .offset_positions
            .into_iter()
            .map(GlobalTilePos::from)
            .collect();

        let radius_paired_w_offsets = seri
            .radius_offset
            .into_iter()
            .map(|(radius, (x, y))| (radius, Vec2::new(x, y)))
            .collect();

        Self::new(offset_positions, radius_paired_w_offsets)
    }

    pub fn melee_default_zone() -> Self {
        Self::new(vec![GlobalTilePos::new(0, -1)], Vec::new())
    }

    pub fn collision_default_zone() -> Self {
        Self::new(vec![GlobalTilePos::new(0, 0)], Vec::new())
    }

    pub fn perimeter_size(&self) -> u32 {
        self.perimeter_size
    }

    pub fn gather_accessible_border_positions_for_checked_pos(
        &self,
        anchor_pos: GlobalTilePos,
        checked_pos: GlobalTilePos,
        out: &mut Vec<GlobalTilePos>,
    ) {
        if self.perimeter_size == 0 {
            return;
        }
        out.reserve(4);
        let mut zone_positions = Vec::with_capacity(self.perimeter_size as usize);
        self.gather_zone_positions(CardinalDirection::South, anchor_pos.to_pixelpos(), &mut zone_positions);
        if zone_positions.is_empty() {
            return;
        }
        zone_positions.sort_unstable_by_key(|pos| (pos.0.x, pos.0.y));
        zone_positions.dedup();
        for delta in [IVec2::X, -IVec2::X, IVec2::Y, -IVec2::Y] {
            let neighbor = checked_pos + delta;
            if zone_positions
                .binary_search_by_key(&(neighbor.0.x, neighbor.0.y), |pos| (pos.0.x, pos.0.y))
                .is_ok()
            {
                out.push(neighbor);
            }
        }
    }

    pub fn is_inside_any(
        &self,
        flip: TileFlip,
        direction: CardinalDirection,
        anchor_transf: Vec2,
        consumer_transf: Vec2,
    ) -> bool {
        let _ = flip;
        self.contains_gpos(direction, anchor_transf.into(), consumer_transf.into())
    }

    pub fn intersects_zone(
        &self,
        anchor_direction: CardinalDirection,
        anchor_transf: Vec2,
        other_zone: &InteractionZone,
        other_direction: CardinalDirection,
        other_anchor_transf: Vec2,
    ) -> bool {
        let mut zone_positions = Vec::with_capacity(self.offset_positions.len() + self.radius_paired_w_offsets.len());
        self.gather_zone_positions(anchor_direction, anchor_transf, &mut zone_positions);
        let other_anchor_gpos: GlobalTilePos = other_anchor_transf.into();
        for zone_pos in zone_positions {
            if other_zone.contains_gpos(other_direction, other_anchor_gpos, zone_pos) {
                return true;
            }
        }
        false
    }

    pub fn gather_zone_positions<'a>(
        &self,
        direction: CardinalDirection,
        anchor_transf: Vec2,
        out: &'a mut Vec<GlobalTilePos>,
    ) -> &'a [GlobalTilePos] {
        let anchor_gpos: GlobalTilePos = anchor_transf.into();

        for &offset_pos in &self.offset_positions {
            out.push(anchor_gpos + rotate_gpos_offset(offset_pos, direction));
        }

        for &(radius, offset) in &self.radius_paired_w_offsets {
            let center = anchor_transf + rotate_vec2_offset(offset, direction);
            let center_gpos = GlobalTilePos::from(center);
            let tile_size = GlobalTilePos::TILE_SIZE_PXS.x.max(1) as f32;
            let radius_in_tiles = (radius / tile_size).ceil().max(0.0) as i32;
            for dy in -radius_in_tiles..=radius_in_tiles {
                for dx in -radius_in_tiles..=radius_in_tiles {
                    let gpos = center_gpos + GlobalTilePos::new(dx, dy);
                    if center.distance(gpos.to_pixelpos()) <= radius {
                        out.push(gpos);
                    }
                }
            }
        }
        out.as_slice()
    }

    fn contains_gpos(
        &self,
        direction: CardinalDirection,
        anchor_gpos: GlobalTilePos,
        checked_pos: GlobalTilePos,
    ) -> bool {
        for &offset_pos in &self.offset_positions {
            let transformed_pos = anchor_gpos + rotate_gpos_offset(offset_pos, direction);
            if transformed_pos == checked_pos {
                return true;
            }
        }

        let anchor_transf = anchor_gpos.to_pixelpos();
        let checked_transf = checked_pos.to_pixelpos();
        for &(radius, offset) in &self.radius_paired_w_offsets {
            let pos = anchor_transf + rotate_vec2_offset(offset, direction);
            if pos.distance(checked_transf) <= radius {
                return true;
            }
        }
        false
    }

    fn perimeter_in_tiles(&self) -> u32 {
        let mut occupied_positions = Vec::with_capacity(
            self.offset_positions.len()
                + self.radius_paired_w_offsets.len().saturating_mul(9),
        );
        self.gather_zone_positions(CardinalDirection::South, Vec2::ZERO, &mut occupied_positions);
        if occupied_positions.is_empty() {
            return 0;
        }
        occupied_positions.sort_unstable_by_key(|pos| (pos.0.x, pos.0.y));
        occupied_positions.dedup();
        let mut perimeter = 0u32;
        for occupied in occupied_positions.iter() {
            for delta in [IVec2::X, -IVec2::X, IVec2::Y, -IVec2::Y] {
                let neighbor_key = (occupied.0.x + delta.x, occupied.0.y + delta.y);
                if occupied_positions
                    .binary_search_by_key(&neighbor_key, |pos| (pos.0.x, pos.0.y))
                    .is_err()
                {
                    perimeter += 1;
                }
            }
        }
        perimeter
    }
}

fn rotate_gpos_offset(offset: GlobalTilePos, direction: CardinalDirection) -> GlobalTilePos {
    let x = offset.0.x;
    let y = offset.0.y;
    match direction {
        CardinalDirection::South => offset,
        CardinalDirection::West => GlobalTilePos::new(y, -x),
        CardinalDirection::North => GlobalTilePos::new(-x, -y),
        CardinalDirection::East => GlobalTilePos::new(-y, x),
    }
}

fn rotate_vec2_offset(offset: Vec2, direction: CardinalDirection) -> Vec2 {
    let x = offset.x;
    let y = offset.y;
    match direction {
        CardinalDirection::South => offset,
        CardinalDirection::West => Vec2::new(y, -x),
        CardinalDirection::North => Vec2::new(-x, -y),
        CardinalDirection::East => Vec2::new(-y, x),
    }
}
