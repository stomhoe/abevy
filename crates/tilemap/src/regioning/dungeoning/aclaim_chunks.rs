use bevy::platform::collections::HashSet;
#[allow(unused_imports)] use bevy::prelude::*;

use common::common_components::HashId;
use rand::SeedableRng;
use rand_distr::{Distribution, Normal};
use ::tilemap_shared::*;

use crate::regioning::{    dungeoning::dungeoning_ids::{ARCHI, CHAMBERS_CORRIDORS, DRUNKWALK, MAZE, SPIRAL},
    regioning_components::*, regioning_messages::{ChunksClaim, OfferChunk}, regioning_sgc_components::StructuredGenConfig
};


pub const ADMITTED_STRUCTURE_IDS_FOR_CLAIMING: &[HashId] = &[
    DRUNKWALK,
    CHAMBERS_CORRIDORS,
    MAZE,
    SPIRAL,
    ARCHI,
];
// pub const ADMITTED_STRUCTURE_IDS_FOR_CLAIMING: &[HashId] = &[

#[allow(unused_parens, )]
pub fn claim_chunks_for_various_dungeon_types(
    mut offered_chunks: MessageReader<OfferChunk>,
    mut writer: MessageWriter<ChunksClaim>,
    region_dimension: Query<&DimensionRef>,
    structured_gens: Query<(&StructuredGenConfig,)>,
    dimension_hash: Query<&HashId>,
    settings: Query<&GlobalGenSettings>,
    mut claimlists: Query<&mut ClaimList>,
) {
    let Ok(settings) = settings.single() else {
        error_once!("Failed to get global gen settings");
        return;
    };
    let mut claims_to_emit = Vec::new();
    let mut already_claimed: HashSet<ChunkPos> = HashSet::new();

    let mut mark_skipped = |region_ent: Entity, i: u64| {
        if let Ok(mut claimlist) = claimlists.get_mut(region_ent) {
            claimlist.skipped_is.insert(i as usize);
        }
    };

    for offer in offered_chunks.read() {
        let Ok((structured_gen_cfg,)) = structured_gens.get(offer.structured_gen_cfg_ent)
        else {
            error!(target: "dungeoning", "StructuredGenConfig entity {:?} not found when making DrunkwalkDungeon, skipping structure spawn", offer.structured_gen_cfg_ent);
            mark_skipped(offer.region_ent, offer.i);
            continue; };

        if !ADMITTED_STRUCTURE_IDS_FOR_CLAIMING.contains(&structured_gen_cfg.structure_hash_id()) {
            trace!(target: "dungeoning", "StructuredGenConfig entity {:?} is not in admitted structures, skipping", offer.structured_gen_cfg_ent);
            continue;
        }
        let center_chunk = offer.start_pos;

        let Ok(dimension_ref) = region_dimension.get(offer.region_ent) else {
            warn!(target: "dungeoning", "Region entity {:?} has no DimensionRef component when claiming chunks for structure spawn, skipping", offer.region_ent);
            mark_skipped(offer.region_ent, offer.i);
            continue;
        };

        let Ok(dimension_hash) = dimension_hash.get(dimension_ref.0) else {
            warn!(target: "dungeoning", "Dimension entity {:?} has no HashId component when claiming chunks for structure spawn, skipping", dimension_ref.0);
            mark_skipped(offer.region_ent, offer.i);
            continue;
        };

        let seed = center_chunk.hash_value(&settings, *dimension_hash, 0);
        let mut rng = rand_pcg::Pcg64Mcg::seed_from_u64(seed);

        let min_side_length = structured_gen_cfg
            .args
            .parse_arg("claim_square_min_side_length", 5)
            .max(1);
        let max_side_length = structured_gen_cfg
            .args
            .parse_arg("claim_square_max_side_length", 9)
            .max(1);
        let normal_mean: f32 = structured_gen_cfg
            .args
            .parse_arg("claim_square_normal_mean", 6.0);
        let normal_std_dev: f32 = structured_gen_cfg
            .args
            .parse_arg("claim_square_normal_std_dev", 4.0);
        let normal_std_dev = normal_std_dev.max(0.01);
        let (min_side_length, max_side_length) = if min_side_length <= max_side_length {
            (min_side_length, max_side_length)
        } else {
            (max_side_length, min_side_length)
        };
        let side_length = {
            let normal = Normal::new(normal_mean, normal_std_dev).unwrap();
            (normal.sample(&mut rng) as i32).clamp(min_side_length, max_side_length)
        };
        let half_spread = side_length / 2;
        let region_pos = center_chunk.to_region_pos();

        let start_offset = -half_spread;
        let end_offset = start_offset + side_length - 1;

        let mut chunk_positions: Vec<ChunkPos> = Vec::new();
        let mut full_square_available = true;
        for dy in start_offset..=end_offset {
            for dx in start_offset..=end_offset {
                let candidate = center_chunk + IVec2::new(dx, dy);
                if !region_pos.contains_chunkpos(candidate) || already_claimed.contains(&candidate) {
                    full_square_available = false;
                    break;
                }
                chunk_positions.push(candidate);
            }
            if !full_square_available {
                break;
            }
        }

        if !full_square_available {
            trace!(target: "dungeoning", "Square around {:?} would be partial; skipping claim", center_chunk);
            mark_skipped(offer.region_ent, offer.i);
            continue;
        }

        for &chunk_pos in &chunk_positions {
            already_claimed.insert(chunk_pos);
        }

        chunk_positions.sort_unstable_by_key(|chunk| (chunk.y(), chunk.x()));
        let chunk_count = chunk_positions.len();
        claims_to_emit.push(ChunksClaim {
            i: offer.i,
            region_ent: offer.region_ent,
            sgc_ent: offer.structured_gen_cfg_ent,
            chunks_pos: chunk_positions,
            partition_tolerant: false,
        });
        trace!(target: "dungeoning", "Emitting ClaimedChunks for ExampleStructure covering {} chunks around {:?}", chunk_count, center_chunk);
    }
    writer.write_batch(claims_to_emit);
}
