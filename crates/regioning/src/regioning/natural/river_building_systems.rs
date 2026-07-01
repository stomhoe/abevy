use bevy::{
    platform::collections::{HashMap, HashSet},
    prelude::*,
};
use common::{common_components::HashId, log_targets::RIVER_BUILD_SYSTEM};
use ::tilemap_shared::*;

use tilemap::tile::tile_resources::{TileEntityMap, TileRef};

use super::river_components::*;

const RIVER: HashId = HashId::hash("river");

#[allow(unused_parens)]
pub fn river_structure_building_system(
    mut reader: MessageReader<SgcPrepareTilesOrder>,
    structured_gens: Query<&StructuredGenConfig>,
    tiles_map: Res<TileEntityMap>,
    loaded_regions: Res<LoadedRegions>,
    region_plans: Query<&RiverRegionPlan>,
    mut writer: MessageWriter<StructureBuildCompliance>,
) {
    let mut compliances_to_emit = Vec::new();
    let mut total_orders = 0_u32;
    let mut river_orders = 0_u32;
    let mut emitted_chunks_total = 0_usize;
    let mut emitted_tiles_total = 0_usize;
    let mut generated_tiles_total = 0_usize;
    let mut blocked_gpos_total = 0_usize;
    for order in reader.read() {
        total_orders = total_orders.saturating_add(1);
        let Ok(cfg) = structured_gens.get(order.structured_gen_cfg_ent) else {
            error!(target: RIVER_BUILD_SYSTEM, "Order {}: missing StructuredGenConfig {:?}", order.i, order.structured_gen_cfg_ent);
            continue;
        };
        if cfg.structure_hash_id() != RIVER {
            continue;
        }
        river_orders = river_orders.saturating_add(1);
        info!(target: RIVER_BUILD_SYSTEM, "Order {}: river build start for region {:?} dim {:?}, claimed chunks={}", order.i, order.region_pos, order.dimension_ref, order.chunks_pos.len());

        let mut compliance = StructureBuildCompliance {
            i: order.i,
            structure_gen_cfg_ent: order.structured_gen_cfg_ent,
            dimension_ref: order.dimension_ref,
            chunk_tiles: Vec::new(),
            terrgen_disabled_gpos_for_chunks: TerrGenDisabledGposForChunks::default(),
            forced_chunk_biomes: Vec::new(),
        };

        let river_tile_id = cfg.args
            .get("river_tile_id")
            .and_then(|v| v.first())
            .map(|s| HashId::hash(s))
            .unwrap_or_else(|| HashId::hash("blue"));
        let Ok(_river_tile_ent) = tiles_map.0.get_cloned(river_tile_id) else {
            error!(target: RIVER_BUILD_SYSTEM, "CRITICAL!!!!: MISSING river_tile_id {:?} for cfg {:?}", river_tile_id, order.structured_gen_cfg_ent);
            compliances_to_emit.push(compliance);
            continue;
        };
        let river_tile_ref = TileRef(river_tile_id);
        let river_tile_size = SizeInTiles::default().inner();
        let gravel_tile_id = cfg.args
            .get("river_gravel_tile_id")
            .and_then(|v| v.first())
            .map(|s| HashId::hash(s))
            .unwrap_or_else(|| HashId::hash("gravel"));
        let gravel_tile_ref = tiles_map.0.get_cloned(gravel_tile_id).ok().map(|_| TileRef(gravel_tile_id));

        let Some(&region_ent) = loaded_regions.0.get(&(order.dimension_ref, order.region_pos)) else {
            error!(target: RIVER_BUILD_SYSTEM, "Order {}: no loaded region entity for region {:?} dim {:?}", order.i, order.region_pos, order.dimension_ref);
            compliances_to_emit.push(compliance);
            continue;
        };
        let Ok(plan) = region_plans.get(region_ent) else {
            error!(target: RIVER_BUILD_SYSTEM, "Order {}: no cached RiverRegionPlan for region {:?} dim {:?}", order.i, order.region_pos, order.dimension_ref);
            compliances_to_emit.push(compliance);
            continue;
        };
        let generated_count = plan.river_tiles.values().map(ChunkGposMask::count_set).sum::<usize>();
        let gravel_generated_count = plan.gravel_tiles.values().map(ChunkGposMask::count_set).sum::<usize>();
        generated_tiles_total = generated_tiles_total.saturating_add(generated_count.saturating_add(gravel_generated_count));

        let claimed_chunks: HashSet<ChunkPos> = order.chunks_pos.iter().copied().collect();
        let mut tiles_by_chunk: HashMap<ChunkPos, Vec<(GlobalTilePos, HashId, Option<tilemap_shared::DeleteOtherTilesInSamePos>)>> = HashMap::with_capacity(plan.claimed_chunks.len());
        let mut terrgen_disabled_gpos_for_chunks = TerrGenDisabledGposForChunks::default();
        let emitted_river_tile_count = append_mask_tiles_by_chunk(
            &mut tiles_by_chunk,
            &plan.river_tiles,
            &claimed_chunks,
            river_tile_ref,
        );
        let emitted_gravel_tile_count = if let Some(gravel_tile_ref) = gravel_tile_ref {
            append_mask_tiles_by_chunk(
                &mut tiles_by_chunk,
                &plan.gravel_tiles,
                &claimed_chunks,
                gravel_tile_ref,
            )
        } else {
            0
        };

        let mut chunks: Vec<_> = tiles_by_chunk.into_iter().collect();
        chunks.sort_by_key(|(chunk, _)| (chunk.y(), chunk.x()));
        let emitted_chunk_count = chunks.len();
        let emitted_tile_count = emitted_river_tile_count.saturating_add(emitted_gravel_tile_count);
        let tiles_outside_claimed = generated_count.saturating_sub(emitted_river_tile_count);
        let gravel_tiles_outside_claimed = gravel_generated_count.saturating_sub(emitted_gravel_tile_count);
        for (chunk_pos, tiles) in &chunks {
            let mut blocked_gpos = ChunkGposMask::default();
            for (tile_pos, _, _) in tiles {
                mark_occupied_gpos(&mut blocked_gpos, *chunk_pos, *tile_pos, river_tile_size);
            }
            blocked_gpos_total = blocked_gpos_total.saturating_add(blocked_gpos.count_set());
            terrgen_disabled_gpos_for_chunks.insert_for_chunk(*chunk_pos, blocked_gpos);
        }
        emitted_chunks_total = emitted_chunks_total.saturating_add(emitted_chunk_count);
        emitted_tiles_total = emitted_tiles_total.saturating_add(emitted_tile_count);
        if emitted_chunk_count == 0 {
            error!(target: RIVER_BUILD_SYSTEM, "Order {}: generated {} river tiles and {} gravel tiles but emitted 0 chunks (claimed chunks={}, outside_claimed_river_tiles={}, outside_claimed_gravel_tiles={})", order.i, generated_count, gravel_generated_count, claimed_chunks.len(), tiles_outside_claimed, gravel_tiles_outside_claimed);
        } else {
            info!(target: RIVER_BUILD_SYSTEM, "Order {}: emitted {} chunks and {} river tiles plus {} gravel tiles (generated before clip: river={}, gravel={}, outside_claimed_river_tiles={}, outside_claimed_gravel_tiles={})", order.i, emitted_chunk_count, emitted_river_tile_count, emitted_gravel_tile_count, generated_count, gravel_generated_count, tiles_outside_claimed, gravel_tiles_outside_claimed);
        }
        compliance.chunk_tiles = chunks.into_iter().flat_map(|(_, tiles)| tiles).collect();
        compliance.terrgen_disabled_gpos_for_chunks = terrgen_disabled_gpos_for_chunks;

        compliances_to_emit.push(compliance);
    }
    if river_orders > 0 {
        info!(target: RIVER_BUILD_SYSTEM, "river build summary: total_orders={}, river_orders={}, generated_tiles_total={}, emitted_chunks_total={}, emitted_tiles_total={}, blocked_gpos_total={}", total_orders, river_orders, generated_tiles_total, emitted_chunks_total, emitted_tiles_total, blocked_gpos_total);
    }
    writer.write_batch(compliances_to_emit);
}

fn mark_occupied_gpos(
    blocked_gpos: &mut ChunkGposMask,
    chunk_pos: ChunkPos,
    anchor_gpos: GlobalTilePos,
    size: UVec2,
) {
    for y in anchor_gpos.0.y..(anchor_gpos.0.y + size.y as i32) {
        for x in anchor_gpos.0.x..(anchor_gpos.0.x + size.x as i32) {
            blocked_gpos.set_gpos(chunk_pos, GlobalTilePos::new(x, y));
        }
    }
}

fn append_mask_tiles_by_chunk(
    tiles_by_chunk: &mut HashMap<ChunkPos, StructureBuilderTiles>,
    masks: &HashMap<ChunkPos, ChunkGposMask>,
    claimed_chunks: &HashSet<ChunkPos>,
    tile_ref: TileRef,
) -> usize {
    let mut emitted_tile_count = 0_usize;
    for (chunk_pos, mask) in masks {
        if !claimed_chunks.contains(chunk_pos) {
            continue;
        }
        let tiles = tiles_by_chunk
            .entry(*chunk_pos)
            .or_insert_with(|| Vec::with_capacity(mask.count_set()));
        tiles.reserve(mask.count_set());
        let chunk_tile_origin = chunk_pos.to_tilepos();
        for bit_idx in 0..ChunkPos::CHUNK_AREA {
            if !mask.is_set(bit_idx) {
                continue;
            }
            let x = (bit_idx % ChunkPos::CHUNK_SIZE.x as usize) as i32;
            let y = (bit_idx / ChunkPos::CHUNK_SIZE.x as usize) as i32;
            tiles.push((GlobalTilePos(chunk_tile_origin.0 + IVec2::new(x, y)), tile_ref.0, None));
            emitted_tile_count = emitted_tile_count.saturating_add(1);
        }
    }
    emitted_tile_count
}