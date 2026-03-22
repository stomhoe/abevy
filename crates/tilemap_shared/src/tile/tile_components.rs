use bevy::prelude::*;
use bevy::platform::collections::HashMap;
use bevy_ecs_tilemap::prelude::*;
use common::common_components::*;
use serde::{Deserialize, Serialize};

use crate::{
    tile::tile_seris::InteractionZoneSeri, CardinalDirection, GlobalTilePos, SizeInTiles,
};

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone)]
pub struct SpriteTile;

#[derive(Component, Debug, Clone, Default)]
/// maps handle's ids to texture index to use within tilemap as a tile belonging to it
pub struct HashIdToTexIndex(HashIdMap<TileTextureIndex>);
impl HashIdToTexIndex {
    pub fn with_capacity(capacity: usize) -> Self {
        Self(HashIdMap::with_capacity(capacity))
    }
    pub fn reserve(&mut self, additional: usize) {
        self.0.reserve(additional);
    }
    pub fn insert(&mut self, tile_hid: HashId, handle_hid: HashId, tex_index: TileTextureIndex) {
        let _ = self.0.insert(tile_hid.merge(handle_hid), tex_index);
    }
    pub fn get(&self, tile_hid: HashId, handle_hid: HashId) -> Result<TileTextureIndex, ()> {
        let merged = tile_hid.merge(handle_hid);
        self.0.get(merged).cloned()
    }
}

#[derive(Component, Debug, Deserialize, Serialize, Copy, Clone)]
pub struct WalkSpeedMultIfOnTop(pub f32);
impl WalkSpeedMultIfOnTop {
    pub fn is_extremely_low(&self) -> bool {
        self.0 <= 0.01
    }
}
impl Default for WalkSpeedMultIfOnTop {
    fn default() -> Self {
        Self(1.0)
    }
}

#[derive(Component, Clone, Deserialize, Serialize, Debug)]
/// interaction positions (offsets relative to the tile's anchor GlobalTilePos)
pub struct InteractionZones(pub HashIdMap<InteractionZone>);
impl InteractionZones {
    pub fn from_seri(map: HashMap<String, InteractionZoneSeri>) -> Self {
        let mut zones = HashIdMap::with_capacity(map.len());
        for (id, seri) in map {
            zones.overwrite(HashId::from(id), InteractionZone::new(seri));
        }
        Self(zones)
    }
    pub fn collision_mask_zone_from_rows(
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

        Ok(InteractionZone {
            offset_positions,
            radius_paired_w_offsets: Vec::new(),
        })
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
        self.0.get(Self::COLLISION_MASK_HASHID).ok()
    }
    pub fn gather_zone_positions_for_hashid(
        &self,
        zone_id: HashId,
        direction: CardinalDirection,
        anchor_transf: Vec2,
        out: &mut Vec<GlobalTilePos>,
    ) {
        let Some(zone) = self.0.get(zone_id).ok() else {
            return;
        };
        zone.gather_zone_positions(direction, anchor_transf, out);
    }
    pub fn melee_default_zone() -> InteractionZone {
        InteractionZone {
            offset_positions: vec![GlobalTilePos::new(0, -1)],
            radius_paired_w_offsets: Vec::new(),
        }
    }
    pub fn collision_default_zone() -> InteractionZone {
        InteractionZone {
            offset_positions: vec![GlobalTilePos::new(0, 0)],
            radius_paired_w_offsets: Vec::new(),
        }
    }
    pub fn melee_default() -> Self {
        let mut zones = HashIdMap::with_capacity(1);
        zones.overwrite(Self::MELEE_ATTACK, Self::melee_default_zone());
        Self(zones)
    }
    pub fn collision_default() -> Self {
        let mut zones = HashIdMap::with_capacity(1);
        zones.overwrite(Self::COLLISION_MASK_HASHID, Self::collision_default_zone());
        Self(zones)
    }
    pub const ENTER: HashId = HashId::hash("enter");
    pub const MELEE_ATTACK: HashId = HashId::hash("melee_attack");
    pub const COLLISION_MASK_HASHID: HashId = HashId::hash("collision_mask");
}

#[derive(Component, Clone, Deserialize, Serialize, Debug)]
pub struct InteractionZone {
    offset_positions: Vec<GlobalTilePos>,
    radius_paired_w_offsets: Vec<(f32, Vec2)>,
}
impl InteractionZone {
    pub fn new(seri: InteractionZoneSeri) -> Self {
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

        Self {
            offset_positions,
            radius_paired_w_offsets,
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

    pub fn gather_zone_positions(
        &self,
        direction: CardinalDirection,
        anchor_transf: Vec2,
        out: &mut Vec<GlobalTilePos>,
    ) {
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
}

fn rotate_gpos_offset(offset: GlobalTilePos, direction: CardinalDirection) -> GlobalTilePos {
    let x = offset.0.x;
    let y = offset.0.y;
    match direction {
        CardinalDirection::South => offset,
        CardinalDirection::West => GlobalTilePos::new(-y, x),
        CardinalDirection::North => GlobalTilePos::new(-x, -y),
        CardinalDirection::East => GlobalTilePos::new(y, -x),
    }
}

fn rotate_vec2_offset(offset: Vec2, direction: CardinalDirection) -> Vec2 {
    let x = offset.x;
    let y = offset.y;
    match direction {
        CardinalDirection::South => offset,
        CardinalDirection::West => Vec2::new(-y, x),
        CardinalDirection::North => Vec2::new(-x, -y),
        CardinalDirection::East => Vec2::new(y, -x),
    }
}
