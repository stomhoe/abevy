use bevy::platform::collections::HashSet;
#[allow(unused_imports)] use bevy::prelude::*;

use rand::{Rng, SeedableRng};
use rand_distr::{Distribution, Normal};
use ::tilemap_shared::*;

use crate::regioning::{dungeoning::dungeoning_ids::{admitted_structure_ids_for_claiming, ARCHI, SPIRAL}, regioning_components::*, regioning_messages::{ChunksClaim, OfferChunk}, regioning_sgc_components::StructuredGenConfig};

const MIN_CLAIM_SIDE_LENGTH: i32 = 4;
const MIN_CLAIM_AREA: i32 = 16;

#[allow(unused_parens, )]
pub fn claim_chunks_for_various_dungeon_types(
    mut offered_chunks: MessageReader<OfferChunk>,
    mut writer: MessageWriter<ChunksClaim>,
    region_dimension: Query<&DimensionRef>,
    structured_gens: Query<(&StructuredGenConfig,)>,
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

        if !admitted_structure_ids_for_claiming().contains(&structured_gen_cfg.structure_hash_id()) {
            trace!(target: "dungeoning", "StructuredGenConfig entity {:?} is not in admitted structures, skipping", offer.structured_gen_cfg_ent);
            continue;
        }
        let center_chunk = offer.start_pos;

        let Ok(dimension_ref) = region_dimension.get(offer.region_ent) else {
            warn!(target: "dungeoning", "Region entity {:?} has no DimensionRef component when claiming chunks for structure spawn, skipping", offer.region_ent);
            mark_skipped(offer.region_ent, offer.i);
            continue;
        };

        let dimension_hash = dimension_ref.0;

        let seed = center_chunk.hash_value(&settings, dimension_hash, 0);
        let mut rng = rand_pcg::Pcg64Mcg::seed_from_u64(seed);

        let claim_is_square = matches!(structured_gen_cfg.structure_hash_id(), SPIRAL | ARCHI);
        let region_pos = center_chunk.to_region_pos();

        let (start_offset_x, end_offset_x, start_offset_y, end_offset_y) = if claim_is_square {
            let min_side_length = structured_gen_cfg
                .args
                .parse_arg("claim_square_min_side_length", 5)
                .max(5);
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
            let start_offset = -half_spread;
            let end_offset = start_offset + side_length - 1;
            (start_offset, end_offset, start_offset, end_offset)
        } else {
            let min_side_length = structured_gen_cfg
                .args
                .parse_arg("claim_rectangle_min_side_length", structured_gen_cfg.args.parse_arg("claim_square_min_side_length", 2))
                .max(2);
            let max_side_length = structured_gen_cfg
                .args
                .parse_arg("claim_rectangle_max_side_length", structured_gen_cfg.args.parse_arg("claim_square_max_side_length", 22))
                .max(min_side_length);
            let normal_mean: f32 = structured_gen_cfg
                .args
                .parse_arg("claim_rectangle_normal_mean", structured_gen_cfg.args.parse_arg("claim_square_normal_mean", 6.0));
            let normal_std_dev: f32 = structured_gen_cfg
                .args
                .parse_arg("claim_rectangle_normal_std_dev", structured_gen_cfg.args.parse_arg("claim_square_normal_std_dev", 4.0));
            let normal_std_dev = normal_std_dev.max(0.01);
            let normal = Normal::new(normal_mean, normal_std_dev).unwrap();
            let short_side = (normal.sample(&mut rng) as i32).clamp(min_side_length, max_side_length);
            let long_extra = (normal.sample(&mut rng) as i32).abs().clamp(0, max_side_length);
            let long_side = (short_side + long_extra).max(min_side_length);
            let (width, mut height) = if rng.random_bool(0.5) {
                (short_side, long_side)
            } else {
                (long_side, short_side)
            };
            let claim_area = width * height;
            if claim_area < MIN_CLAIM_AREA {
                height = ((MIN_CLAIM_AREA + width - 1) / width).max(MIN_CLAIM_SIDE_LENGTH);
            }
            let half_width = width / 2;
            let half_height = height / 2;
            let start_offset_x = -half_width;
            let end_offset_x = start_offset_x + width - 1;
            let start_offset_y = -half_height;
            let end_offset_y = start_offset_y + height - 1;
            (start_offset_x, end_offset_x, start_offset_y, end_offset_y)
        };

        let mut chunk_positions: Vec<ChunkPos> = Vec::new();
        let mut full_claim_available = true;
        for dy in start_offset_y..=end_offset_y {
            for dx in start_offset_x..=end_offset_x {
                let candidate = center_chunk + IVec2::new(dx, dy);
                if !region_pos.contains_chunkpos(candidate) || already_claimed.contains(&candidate) {
                    full_claim_available = false;
                    break;
                }
                chunk_positions.push(candidate);
            }
            if !full_claim_available {
                break;
            }
        }

        if !full_claim_available {
            trace!(target: "dungeoning", "Claim around {:?} would be partial; skipping claim", center_chunk);
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
        trace!(target: "dungeoning", "Emitting ClaimedChunks covering {} chunks around {:?}", chunk_count, center_chunk);
    }
    writer.write_batch(claims_to_emit);
}
