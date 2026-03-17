use bevy::platform::collections::HashSet;
use bevy::prelude::*;
use param_sets::BlockingTileParamSet;
use tilemap_shared::{ChunkPos, DimensionRef, GlobalTilePos, LoadedChunks};

pub fn find_nearest_overlap_resolution_gpos(
    blocking_tiles: &mut BlockingTileParamSet,
    loaded_chunks: &LoadedChunks,
    reserved_positions: &HashSet<(DimensionRef, GlobalTilePos)>,
    dim_ref: DimensionRef,
    anchor: GlobalTilePos,
    being_ent: Entity,
) -> Option<GlobalTilePos> {
    let home_chunk = anchor.to_chunkpos();
    let mut nearby_chunks = loaded_chunks
        .0
        .keys()
        .filter_map(|&(loaded_dim, chunk_pos)| {
            if loaded_dim == dim_ref {
                Some(chunk_pos)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    nearby_chunks.sort_unstable_by_key(|chunk_pos| {
        (chunk_pos.0.x - home_chunk.0.x).abs() + (chunk_pos.0.y - home_chunk.0.y).abs()
    });

    for chunk_pos in nearby_chunks {
        if let Some(found_gpos) = find_nearest_overlap_resolution_gpos_in_chunk(
            blocking_tiles,
            reserved_positions,
            dim_ref,
            anchor,
            chunk_pos,
            being_ent,
        ) {
            return Some(found_gpos);
        }
    }

    None
}

fn find_nearest_overlap_resolution_gpos_in_chunk(
    blocking_tiles: &mut BlockingTileParamSet,
    reserved_positions: &HashSet<(DimensionRef, GlobalTilePos)>,
    dim_ref: DimensionRef,
    anchor: GlobalTilePos,
    chunk_pos: ChunkPos,
    being_ent: Entity,
) -> Option<GlobalTilePos> {
    let min_tile = chunk_pos.to_tilepos().0;
    let clamped_anchor = chunk_pos.clamp_gpos_to_chunk(anchor);
    let local_anchor = clamped_anchor.0 - min_tile;
    let max_radius = (ChunkPos::CHUNK_SIZE.x.max(ChunkPos::CHUNK_SIZE.y) as i32).saturating_sub(1);

    for radius in 0..=max_radius {
        let min_local_x = (local_anchor.x - radius).max(0);
        let max_local_x = (local_anchor.x + radius).min(ChunkPos::CHUNK_SIZE.x as i32 - 1);
        let min_local_y = (local_anchor.y - radius).max(0);
        let max_local_y = (local_anchor.y + radius).min(ChunkPos::CHUNK_SIZE.y as i32 - 1);

        for local_y in min_local_y..=max_local_y {
            for local_x in min_local_x..=max_local_x {
                if radius != 0
                    && local_x != min_local_x
                    && local_x != max_local_x
                    && local_y != min_local_y
                    && local_y != max_local_y
                {
                    continue;
                }
                let candidate = GlobalTilePos(min_tile + IVec2::new(local_x, local_y));
                if reserved_positions.contains(&(dim_ref, candidate)) {
                    continue;
                }
                if blocking_tiles.is_blocked_at_tiles_only(dim_ref, candidate, being_ent) {
                    continue;
                }
                return Some(candidate);
            }
        }
    }

    None
}