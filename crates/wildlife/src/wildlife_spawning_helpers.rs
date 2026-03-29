use bevy::{
    platform::collections::HashSet,
    prelude::*,
};
use being::pack::pack_components::*;
use ::being_shared::*;
use tilemap_shared::*;

const SEPARATION_BETWEEN_PACKS_ANCHOR_CPOS: u8 = 1;


#[derive(Clone, Copy)]
pub struct PackAnchorCpos {
    pub pack_ent: Entity,
    pub center_chunk: ChunkPos,
}



#[derive(Component, Debug, Clone, Copy)]
pub struct NaturalSpawnOrigin(pub ChunkPos);

pub fn choose_best_anchor_cpos_for_pack(
    distribution: &BiomeDistribution,
    biome_ent: Entity,
    current_target: Entity,
    macro_chunk_pos: MacrochunkPos,
    prev_selected_anchors: &[PackAnchorCpos],
    pack_min_dists: Option<&PackMinSepToPacksOrRaces>,
    pack_min_dists_query: &Query<&PackMinSepToPacksOrRaces>,
) -> Option<ChunkPos> {
    let mut seen_candidates = HashSet::<ChunkPos>::default();
    let sorted_candidates = distribution.sorted_chunk_candidates_for_biome(biome_ent);
    for candidate in sorted_candidates {
        if !macro_chunk_pos.contains_chunkpos(candidate) || !seen_candidates.insert(candidate) {
            continue;
        }
        if is_pack_center_candidate_valid(
            candidate,
            current_target,
            prev_selected_anchors,
            pack_min_dists,
            pack_min_dists_query,
        ) {
            return Some(candidate);
        }
    }
    None
}

fn is_pack_center_candidate_valid(
    candidate: ChunkPos,
    current_target: Entity,
    prev_selected_anchors: &[PackAnchorCpos],
    pack_min_dists: Option<&PackMinSepToPacksOrRaces>,
    pack_min_dists_query: &Query<&PackMinSepToPacksOrRaces>,
) -> bool {
    prev_selected_anchors.iter().all(|prev_anchor| {
        let mut min_sep_inbetween_chunks = pack_min_dists
            .map(|pack_min_dists| pack_min_dists.min_inbetween_chunks(prev_anchor.pack_ent))
            .unwrap_or(SEPARATION_BETWEEN_PACKS_ANCHOR_CPOS);
        if let Ok(existing_pack_min_dists) = pack_min_dists_query.get(prev_anchor.pack_ent) {
            if let Some(reciprocal_required_inbetween_chunks) = existing_pack_min_dists
                .configured_min_inbetween_chunks(current_target)
            {
                min_sep_inbetween_chunks = min_sep_inbetween_chunks.max(reciprocal_required_inbetween_chunks);
            }
        }
        if prev_anchor.pack_ent == current_target {
            min_sep_inbetween_chunks = min_sep_inbetween_chunks.max(1);
        }
        (prev_anchor.center_chunk.0 - candidate.0)
            .abs()
            .max_element()
            >= i32::from(min_sep_inbetween_chunks) + 1
    })
}
