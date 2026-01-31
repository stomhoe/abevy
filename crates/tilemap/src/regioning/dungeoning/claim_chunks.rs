use bevy::platform::collections::HashSet;
#[allow(unused_imports)] use bevy::prelude::*;

use common::common_components::HashId;
use game_common::game_common_components::ArgsMap;
use dimension_shared::DimensionRef;
use rand::SeedableRng;
use rand_distr::{Distribution, Normal};
use ::tilemap_shared::*;

use crate::regioning::{
    regioning_components::*,
    regioning_messages::{ChunksClaim, OfferChunk},
    regioning_sgc_components::StructuredGenConfig,
};

const DRUNKWALK: HashId = HashId::hash("drunkwalk");
const CHAMBERS_CORRIDORS: HashId = HashId::hash("chamberscorridors");
const MAZE: HashId = HashId::hash("maze");
const SPIRAL: HashId = HashId::hash("spiral");
const ARCHI_SPIRAL: HashId = HashId::hash("archimedes_spiral");

/// Cache for Claim Chunks configuration
#[derive(Debug, Clone)]
pub struct ClaimChunksConfig {
    min_side_length: i32,
    max_side_length: i32,
    normal_mean: f32,
    normal_std_dev: f32,
}

impl ClaimChunksConfig {
    fn from_args(args: &ArgsMap) -> Self {
        let min_side_length: i32 = args.parse_arg("claim_square_min_side_length", 3);
        let max_side_length: i32 = args.parse_arg("claim_square_max_side_length", 9);
        let normal_mean: f32 = args.parse_arg("claim_square_normal_mean", 4.0);
        let normal_std_dev: f32 = args.parse_arg("claim_square_normal_std_dev", 2.0);

        Self {
            min_side_length: min_side_length.max(1),
            max_side_length: max_side_length.max(1),
            normal_mean,
            normal_std_dev: normal_std_dev.max(0.01),
        }
    }
}


const ADMITTED_STRUCTURE_IDS_FOR_CLAIMING: &[HashId] = &[
    DRUNKWALK,
    CHAMBERS_CORRIDORS,
    MAZE,
    SPIRAL,
    ARCHI_SPIRAL,   
];
// const ADMITTED_STRUCTURE_IDS_FOR_CLAIMING: &[HashId] = &[

#[allow(unused_parens, )]
pub fn claim_chunks_for_various_dungeon_types(
    mut offered_chunks: MessageReader<OfferChunk>,
    mut writer: MessageWriter<ChunksClaim>,
    region_dimension: Query<&DimensionRef>,
    structured_gens: Query<(&StructuredGenConfig,)>,
    dimension_hash: Query<&HashId>,
    settings: Single<&GlobalGenSettings>,
    mut config_cache: Local<Option<ClaimChunksConfig>>,
) {
    let mut claims_to_emit = Vec::new();
    let mut already_claimed: HashSet<ChunkPos> = HashSet::new();
    
    for offer in offered_chunks.read() {
        let Ok((structured_gen_cfg,)) = structured_gens.get(offer.structured_gen_cfg_ent)
        else { 
            error!(target: "dungeoning", "StructuredGenConfig entity {:?} not found when making DrunkwalkDungeon, skipping structure spawn", offer.structured_gen_cfg_ent);
            continue; };

        if !ADMITTED_STRUCTURE_IDS_FOR_CLAIMING.contains(&structured_gen_cfg.structure_hash_id()) {
            trace!(target: "dungeoning", "StructuredGenConfig entity {:?} is not in admitted structures, skipping", offer.structured_gen_cfg_ent);
            continue;
        }
        let center_chunk = offer.start_pos;

        let Ok(dimension_ref) = region_dimension.get(offer.region_ent) else {
            warn!(target: "dungeoning", "Region entity {:?} has no DimensionRef component when claiming chunks for structure spawn, skipping", offer.region_ent);
            continue;
        };

        let Ok(dimension_hash) = dimension_hash.get(dimension_ref.0) else {
            warn!(target: "dungeoning", "Dimension entity {:?} has no HashId component when claiming chunks for structure spawn, skipping", dimension_ref.0);
            continue;
        };

        let seed = center_chunk.hash_value(&settings, *dimension_hash, 0);
        let mut rng = rand_pcg::Pcg64Mcg::seed_from_u64(seed);

        // Cache config on first call
        let cfg = config_cache.get_or_insert_with(|| ClaimChunksConfig::from_args(&structured_gen_cfg.args));

        let min_side_length = cfg.min_side_length;
        let max_side_length = cfg.max_side_length;
        let normal_mean = cfg.normal_mean;
        let normal_std_dev = cfg.normal_std_dev;
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
            chunks_gpos: chunk_positions,
            partition_tolerant: false,
        });
        trace!(target: "dungeoning", "Emitting ClaimedChunks for ExampleStructure covering {} chunks around {:?}", chunk_count, center_chunk);
    }
    writer.write_batch(claims_to_emit);
}
