use std::{hash::{DefaultHasher, Hash, Hasher}};

use bevy::ecs::system::SystemParam;
#[allow(unused_imports)] use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
use common::common_components::*;
use serde::{Deserialize, Serialize};
#[allow(unused_imports, )]
use bevy::platform::collections::{HashSet, HashMap};
use smallvec::SmallVec;


use crate::{CardinalDirection, DiagonalCardinalDirection, DimensionRef, tilemap_positioning::*};

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct PreChunkDespawnSystems;


#[derive(Resource, Default)]
pub struct LoadedChunks (pub HashMap<(DimensionRef, ChunkPos), Entity>,);

#[derive(Component, Debug, Deserialize, Serialize, Clone, )]
#[require(Replicated, Prefix::trunc("GlobalGenSettings"), AssetScoped, HotReload)]
pub struct GlobalGenSettings {

    pub seed: i32,
    pub world_freq: f32,
    pub tectonic_frequency: f32,
    pub hot_reload_window_open_on_start: bool,
    /// Timeout in seconds to wait for StructureBuildCompliance before giving up
    pub structure_build_timeout_secs: f64,
    pub players_spawn_probe_id: StrId,
}
const DONT_TOUCH: f32 = 1000.;
impl Default for GlobalGenSettings {
    fn default() -> Self {
        Self {
            seed: 0,
            world_freq: 20.
            /DONT_TOUCH,
            tectonic_frequency: 20.
            /DONT_TOUCH,
            hot_reload_window_open_on_start: false,
            structure_build_timeout_secs: 4.0,
            players_spawn_probe_id: StrId::trunc("suland"),
        }
    }
}

#[derive(Debug, Message, Default)]
pub struct ForceAllChunksDespawn;

type PDiskDistType = i64;
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Component, )]
pub struct PoissonDisk { pub mindists_seeds: Vec<(PDiskDistType, u64)>, }
impl PoissonDisk {
    pub fn new(min_distance: u8, seed: u64) -> Result<Self, BevyError> {

        let max = 5;

        if min_distance > max {
            return Err(BevyError::from(format!("min_distance must be <= {}", max)));
        } else if min_distance == 0 {
            return Err(BevyError::from("min_distance must be > 0"));
        }
        Ok(Self { mindists_seeds: vec![(min_distance as PDiskDistType, seed)] })
    }
    pub fn multiple_tagged(mindists_tag: Vec<(Option<u8>, String)>, fallback_mindist: u8, max: u8) -> Result<Self, BevyError> {
        let mut mindists_seeds: Vec<(PDiskDistType, u64)> = Vec::with_capacity(mindists_tag.len());
        for (min_distance, tag) in mindists_tag.iter() {
            let min_distance = min_distance.unwrap_or(fallback_mindist);

            if min_distance > max {
                return Err(BevyError::from(format!("min_distance must be <= {}", max)));
            } else if min_distance == 0 {
                return Err(BevyError::from("min_distance must be > 0"));
            }
            let mut hasher = DefaultHasher::new();
            tag.hash(&mut hasher);
            let seed = hasher.finish();
            mindists_seeds.push((min_distance as PDiskDistType, seed));
        }
        Ok(Self { mindists_seeds })
    }
    pub fn is_allowed_position<T: HashablePosVec>(&self, pos: T, settings: &GlobalGenSettings, dim_hash: HashId, check_within_radius: bool, size_in_tiles: OplistSize) -> bool {
        self.sample(pos, settings, dim_hash, check_within_radius, size_in_tiles) > 0.0
    }

    pub fn sample<T: HashablePosVec>(&self, pos: T, settings: &GlobalGenSettings, dim_hash: HashId, check_within_radius: bool, size_in_tiles: OplistSize) -> f64 {

        let mut sum = 0.0;
        for &(min_distance, seed) in self.mindists_seeds.iter() {
            let val = pos.normalized_hash_value(settings, dim_hash, seed);
            sum += val;
            let added_sample_distance_x = size_in_tiles.x() as i32 - 1;
            let added_sample_distance_y = size_in_tiles.y() as i32 - 1;

            for dy in -(min_distance as i32)..=(min_distance as i32) {
                for dx in -(min_distance as i32)..=(min_distance as i32) {
                    if dx == 0 && dy == 0 {
                        continue;
                    }
                    // Only check within circle of radius min_distance
                    if check_within_radius && dx * dx + dy * dy > (min_distance as i32).pow(2) {
                        continue;
                    }
                    // Calculate the neighbor's position by offsetting the current tile position
                    let neighbor_x = pos.x() + dx + added_sample_distance_x;
                    let neighbor_y = pos.y() + dy + added_sample_distance_y;
                    let neighbor_pos = GlobalTilePos(IVec2::new(neighbor_x, neighbor_y));
                    let neighbor_val = neighbor_pos.normalized_hash_value(settings, dim_hash, seed);
                    if neighbor_val > val {
                        return 0.0;
                    }
                }
            }

        }
        sum / (self.mindists_seeds.len() as f64)
        }
}
impl Default for PoissonDisk { fn default() -> Self { Self { mindists_seeds: vec![(1, 0)] } } }


#[derive(Component, Debug, Copy, Clone, Hash, PartialEq, Eq, )]
#[relationship(relationship_target = Tilemaps)]
pub struct TilemapOf {
    #[relationship]
    pub chunk: Entity,
}
impl TilemapOf {
    pub fn new(chunk: Entity) -> Self {
        Self { chunk }
    }
}

#[derive(Component, Debug, Clone)]
#[relationship_target(relationship = TilemapOf)]
pub struct Tilemaps(Vec<Entity>);
impl Tilemaps { pub fn entities(&self) -> &[Entity] { &self.0 } }

#[derive(Debug, )]
pub struct ChunkEntityMatrix {
    cells: Box<[ReturnedVec; ChunkPos::CHUNK_AREA]>,
}
impl ChunkEntityMatrix {
    pub fn new() -> Self {
        Self::default()
    }
    fn index(local: UVec2) -> usize {
        let width = ChunkPos::CHUNK_SIZE.x as usize;
        (local.y as usize * width) + local.x as usize
    }
    pub fn get(&self, local: UVec2) -> &[Entity] {
        self.cells.get(Self::index(local)).map_or(&[], |cell| cell.as_slice())
    }
    pub fn push(&mut self, local: UVec2, entity: Entity) {
        if let Some(cell) = self.cells.get_mut(Self::index(local)) {
            cell.push(entity);
        }
    }
    pub fn swap_remove(&mut self, local: UVec2, entity: Entity) -> Option<()> {
        let cell = self.cells.get_mut(Self::index(local))?;
        let idx = cell.iter().position(|&e| e == entity)?;
        cell.swap_remove(idx);
        Some(())
    }
}
impl Default for ChunkEntityMatrix {
    fn default() -> Self {
        let cells = std::array::from_fn(|_| ReturnedVec::new());
        Self {
            cells: Box::new(cells),
        }
    }
}

#[derive(Resource, Debug, Default)]
pub struct SpriteTilesAtGpos  {
    pub map: bevy::platform::collections::HashMap<(DimensionRef, ChunkPos), ChunkEntityMatrix>,
}
impl SpriteTilesAtGpos {
    fn chunk_and_local(gpos: GlobalTilePos) -> (ChunkPos, UVec2) {
        let chunk = gpos.to_chunkpos();
        let chunk_origin = chunk.to_tilepos();
        let local = (gpos.0 - chunk_origin.0).as_uvec2();
        (chunk, local)
    }
    pub fn tiles_at_pos(&self, dim_ref: DimensionRef, gpos: GlobalTilePos) -> &[Entity] {
        let (chunk, local) = Self::chunk_and_local(gpos);
        self.map
            .get(&(dim_ref, chunk))
            .map(|matrix| matrix.get(local))
            .unwrap_or(&[])
    }
    pub fn remove_tile(&mut self, dim_ref: DimensionRef, gpos: GlobalTilePos, tile_ent: Entity, size: SizeInTiles) {
        let size = size.inner();
        for y in 0..size.y {
            for x in 0..size.x {
                let curr_gpos = GlobalTilePos(gpos.0 + IVec2::new(x as i32, y as i32));
                let (chunk, local) = Self::chunk_and_local(curr_gpos);
                if let Some(matrix) = self.map.get_mut(&(dim_ref, chunk)) {
                    matrix.swap_remove(local, tile_ent);
                }
            }
        }
    }
    pub fn reserve_capacity(&mut self, additional: usize) {
        self.map.reserve(additional);
    }
    pub fn insert(&mut self, entity: Entity, dimension_ref: DimensionRef, gpos: GlobalTilePos, size: SizeInTiles) {
        let size = size.inner();
        for y in 0..size.y {
            for x in 0..size.x {
                let curr_gpos = GlobalTilePos(gpos.0 + IVec2::new(x as i32, y as i32));
                let (chunk, local) = Self::chunk_and_local(curr_gpos);
                let matrix = self
                    .map
                    .entry((dimension_ref, chunk))
                    .or_insert_with(ChunkEntityMatrix::new);
                matrix.push(local, entity);
            }
        }
    }
}

#[derive(Resource, Debug, Default)]
pub struct BeingsAtGpos {
    pub map: bevy::platform::collections::HashMap<(DimensionRef, ChunkPos), ChunkEntityMatrix>,
}
impl BeingsAtGpos {
    fn chunk_and_local(gpos: GlobalTilePos) -> (ChunkPos, UVec2) {
        let chunk = gpos.to_chunkpos();
        let chunk_origin = chunk.to_tilepos();
        let local = (gpos.0 - chunk_origin.0).as_uvec2();
        (chunk, local)
    }
    pub fn beings_at_pos(&self, dim_ref: DimensionRef, gpos: GlobalTilePos) -> &[Entity] {
        let (chunk, local) = Self::chunk_and_local(gpos);
        self.map
            .get(&(dim_ref, chunk))
            .map(|matrix| matrix.get(local))
            .unwrap_or(&[])
    }
    pub fn remove_being(&mut self, dim_ref: DimensionRef, gpos: GlobalTilePos, being_ent: Entity) {
        let (chunk, local) = Self::chunk_and_local(gpos);
        let Some(matrix) = self.map.get_mut(&(dim_ref, chunk)) else {
            return;
        };
        matrix.swap_remove(local, being_ent);
    }
    pub fn insert_being(&mut self, dim_ref: DimensionRef, gpos: GlobalTilePos, being_ent: Entity) {
        let (chunk, local) = Self::chunk_and_local(gpos);
        let matrix = self
            .map
            .entry((dim_ref, chunk))
            .or_insert_with(ChunkEntityMatrix::new);
        matrix.push(local, being_ent);
    }
}

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone)]
pub struct SpriteTile;

pub type ReturnedVec = SmallVec<[Entity; 16]>;

#[derive(Component, Debug, Clone, Default, )]
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


#[derive(SystemParam)]
#[allow(unused_parens, )]
/// system which uses this must be put .in_set(PreChunkDespawnReaders)
pub struct TileGatheringParamSet<'w, 's> {
    spritetiles_at_gpos: Res<'w, SpriteTilesAtGpos>,
    loaded_chunks: Res<'w, LoadedChunks>,
    chunk_children: Query<'w, 's, &'static Tilemaps>,
    pub tilemap_query: Query<'w, 's, (&'static mut TileStorage, &'static HashIdToTexIndex),>,
}
impl<'w, 's> TileGatheringParamSet<'w, 's> {
    pub fn gather_tiles_at(&self, vec_to_drain: &mut impl Extend<Entity>, dim: DimensionRef, gpos: GlobalTilePos) {
        let chunk_pos = gpos.to_chunkpos();
        vec_to_drain.extend(self.spritetiles_at_gpos.tiles_at_pos(dim, gpos).iter().copied());
        let Some(&chunk_ent) = self.loaded_chunks.0.get(&(dim, chunk_pos)) else {
            return;
        };
        if let Ok(tilemaps) = self.chunk_children.get(chunk_ent){
            for &tmap_ent in tilemaps.entities() {
                let Ok((storage, ..)) = self.tilemap_query.get(tmap_ent) else {
                    continue;
                };
                let tpos = gpos.to_tilepos();

                if let Some(tile_ent) = storage.get(&tpos) {
                    vec_to_drain.extend(std::iter::once(tile_ent));
                }
            }
        }
    }
}

#[derive(Component, Debug, Deserialize, Serialize, Copy, Clone, Reflect)]
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

#[derive(Component, Debug, Deserialize, Serialize, Copy, Clone, Reflect)]
pub struct TileCollisionMask {
    width: u8,
    height: u8,
    bits: u64,
}
impl TileCollisionMask {
    pub fn from_rows(rows: &[String], size_in_tiles: SizeInTiles) -> Result<Self, BevyError> {
        let width = size_in_tiles.inner().x as usize;
        let height = size_in_tiles.inner().y as usize;
        if rows.len() != height {
            return Err(BevyError::from(format!(
                "collision_mask row count ({}) does not match size_in_tiles.y ({})",
                rows.len(),
                height
            )));
        }

        let mut bits = 0u64;
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
                        // RON rows are authored top-to-bottom; local mask space is bottom-to-top.
                        let source_y = (height - 1) - y;
                        let bit_i = source_y * width + x;
                        bits |= 1u64 << bit_i;
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

        Ok(Self {
            width: width as u8,
            height: height as u8,
            bits,
        })
    }


    pub fn is_solid_at_world_pos_with_flip(
        &self,
        tile_origin: GlobalTilePos,
        target: GlobalTilePos,
        flip: TileFlip,
        direction: CardinalDirection,
    ) -> bool {
        let rel = target.0 - tile_origin.0;
        if rel.x < 0 || rel.y < 0 {
            return true;
        }
        let x = rel.x as u32;
        let y = rel.y as u32;
        self.is_solid_local_with_flip(x, y, flip, direction)
    }


    pub fn is_solid_local_with_flip(
        &self,
        x: u32,
        y: u32,
        flip: TileFlip,
        direction: CardinalDirection,
    ) -> bool {
        let (sx, sy) = self.target_to_source_local(x, y, flip, direction);
        let Some((sx, sy)) = sx.zip(sy) else {
            return true;
        };
        self.is_source_bit_set(sx, sy)
    }

    fn is_source_bit_set(&self, x: u32, y: u32) -> bool {
        let width = self.width as u32;
        let height = self.height as u32;
        if x >= width || y >= height {
            return true;
        }
        let bit_i = y * width + x;
        (self.bits & (1u64 << bit_i)) != 0
    }

    fn target_to_source_local(
        &self,
        x: u32,
        y: u32,
        flip: TileFlip,
        direction: CardinalDirection,
    ) -> (Option<u32>, Option<u32>) {
        let width = self.width as u32;
        let height = self.height as u32;

        let (mut w, mut h) = match direction {
            CardinalDirection::South | CardinalDirection::North => (width, height),
            CardinalDirection::West | CardinalDirection::East => (height, width),
        };
        if x >= w || y >= h {
            return (None, None);
        }

        // Inverse transform from target cell -> source cell to match tile rendering flips.
        let mut sx = x;
        let mut sy = y;

        if flip.y {
            sy = h - 1 - sy;
        }
        if flip.x {
            sx = w - 1 - sx;
        }
        if flip.d {
            std::mem::swap(&mut sx, &mut sy);
            std::mem::swap(&mut w, &mut h);
        }

        if sx >= w || sy >= h {
            return (None, None);
        }

        let (src_x, src_y) = match direction {
            CardinalDirection::South => (sx, sy),
            CardinalDirection::West => {
                if w != height || h != width {
                    return (None, None);
                }
                (sy, height - 1 - sx)
            }
            CardinalDirection::North => {
                if w != width || h != height {
                    return (None, None);
                }
                (width - 1 - sx, height - 1 - sy)
            }
            CardinalDirection::East => {
                if w != height || h != width {
                    return (None, None);
                }
                (width - 1 - sy, sx)
            }
        };

        if src_x >= width || src_y >= height {
            return (None, None);
        }
        (Some(src_x), Some(src_y))
    }
}




#[derive(Message, Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct RecheckTileAdjacency {
    pub dim: DimensionRef,
    pub gpos: GlobalTilePos,
}
impl RecheckTileAdjacency {
    pub fn append_all_adjacent_pos(msgs: &mut Vec<RecheckTileAdjacency>, dim: DimensionRef, base_pos: GlobalTilePos,) {
        for dir in DiagonalCardinalDirection::ALL_DIRS {
            msgs.push(RecheckTileAdjacency {
                dim,
                gpos: base_pos.adjacent_dir(dir),
            });
        }
    }
}

#[derive(Message, Debug, Clone, Copy, Hash, PartialEq, Eq)]
/// Despawn with removal from SpriteTilesAtGpos (if spritetile) and tile adjacency recheck
pub struct SafeDespawn(pub Entity);


#[derive(Component, Clone, Deserialize, Serialize, Debug,)]
/// interaction positions (offsets relative to the tile's anchor GlobalTilePos)
pub struct InteractionZones(
    pub HashIdMap<InteractionZone>
);
impl InteractionZones {
    pub fn new(map: HashMap<String, InteractionZoneSeri>) -> Self {
        let mut zones = HashIdMap::with_capacity(map.len());
        for (id, seri) in map {
            zones.overwrite(HashId::from(id), InteractionZone::new(seri));
        }
        Self(zones)
    }
    pub fn is_inside_interaction_zone(
        &self,
        zone_id: HashId,
        anchor_transf: Vec2,
        client_transf: Vec2,
        direction: CardinalDirection,
    ) -> bool {
        let zone = self.0.get(zone_id).ok();
        zone.is_some_and(|zone| zone.is_inside_any(direction, anchor_transf, client_transf))
    }
    pub const ENTER: HashId = HashId::hash("enter");
}

#[derive(Component, Clone, Deserialize, Serialize, Debug,)]
pub struct InteractionZone{
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
        direction: CardinalDirection,
        anchor_transf: Vec2,
        client_transf: Vec2,
    ) -> bool {
        for &offset_pos in &self.offset_positions {
            let anchor_gpos: GlobalTilePos = anchor_transf.into();
            let client_pos: GlobalTilePos = client_transf.into();
            let checked_pos = anchor_gpos + rotate_gpos_offset(offset_pos, direction);
            if checked_pos == client_pos {
                return true;
            }
        }
        for &(radius, offset) in &self.radius_paired_w_offsets {
            let pos = anchor_transf + rotate_vec2_offset(offset, direction);
            if pos.distance(client_transf) <= radius {
                return true;
            }
        }
        false
    }
}

#[derive(Component, Deserialize, TypePath, Clone, Default)]
pub struct InteractionZoneSeri{
    #[serde(default)]
    pub offset_positions: Vec<(i8, i8)>,
    #[serde(default)]
    pub radius_offset: Vec<(f32, (f32, f32))>,
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
