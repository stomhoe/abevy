use super::*;

pub(super) fn gather_tile_templs_via_idxs<'a>(
    this: &BlockingTileParamSet<'_, '_>,
    dim_ref: DimensionRef,
    gpos: GlobalTilePos,
    out: &'a mut Vec<Entity>,
) -> &'a [Entity] {
    out.clear();
    let macro_chunk_pos = gpos.to_chunkpos().to_macrochunk_pos();
    let Some(&macro_chunk_ent) = this.loaded_macro_chunks.0.get(&(dim_ref, macro_chunk_pos)) else {
        return out.as_slice();
    };
    let Ok(macro_chunk_tile_indices) = this.macro_chunk_tile_indices.get(macro_chunk_ent) else {
        return out.as_slice();
    };
    let Ok(tile_indexing) = this.tile_indexing_query.single() else {
        return out.as_slice();
    };
    let macro_chunk_anchor = macro_chunk_pos.to_chunkpos().to_tilepos();
    let Some(tile_indices) = macro_chunk_tile_indices.tile_indices_at_gpos(macro_chunk_anchor, gpos) else {
        return out.as_slice();
    };
    out.reserve(tile_indices.len());
    for &tile_index in tile_indices.iter() {
        let Some(tile_hash_id) = tile_indexing.hash_id_for_index(tile_index) else {
            continue;
        };
        let Ok(tile_templ_ent) = this.tile_map.0.get_cloned(tile_hash_id) else {
            continue;
        };
        out.push(tile_templ_ent);
    }
    out.as_slice()
}

pub(super) fn gather_collision_tile_samples(
    this: &mut BlockingTileParamSet<'_, '_>,
    dim_ref: DimensionRef,
    occupied_gpos: GlobalTilePos,
) {
    this.collision_tile_samples.clear();
    this.tile_gathering_params.gather_tiles_to_drain(dim_ref, occupied_gpos);
    if !this.tile_gathering_params.to_drain.is_empty() {
        let tile_entities_len = this.tile_gathering_params.to_drain.len();
        let tile_entities_ptr = this.tile_gathering_params.to_drain.as_ptr();
        this.collision_tile_samples.reserve(tile_entities_len);
        for tile_entity_idx in 0..tile_entities_len {
            let tile_entity = unsafe { *tile_entities_ptr.add(tile_entity_idx) };
            let Ok(templ_ref) = this.templ_ref_query.get(tile_entity) else {
                continue;
            };
            let Ok(tile_origin) = this.gpos_query.get(tile_entity) else {
                continue;
            };
            let fallback_direction = this
                .tile_gathering_params
                .cardinal_direction_query
                .get(tile_entity)
                .cloned()
                .unwrap_or_default();
            this.collision_tile_samples.push(CollisionTileSample {
                templ_ent: templ_ref.0,
                tile_origin: *tile_origin,
                direction: this.card_at_gpos.resolve_tile_direction(&this.hash_id_query, templ_ref.0, *tile_origin, fallback_direction),
                dead_despawning: this.will_despawn_query.get(tile_entity).is_ok(),
            });
        }
        this.tile_gathering_params.to_drain.clear();
    }
    if !this.collision_tile_samples.is_empty() {
        return;
    }

    {
        let this_ptr = std::ptr::addr_of!(*this);
        let to_drain = std::ptr::addr_of_mut!(this.tile_gathering_params.to_drain);
        let templ_ents = unsafe {
            gather_tile_templs_via_idxs(&*this_ptr, dim_ref, occupied_gpos, &mut *to_drain)
        };
        let templ_ents_len = templ_ents.len();
        let templ_ents_ptr = templ_ents.as_ptr();
        this.collision_tile_samples.reserve(templ_ents_len);
        for templ_ent_idx in 0..templ_ents_len {
            let templ_ent = unsafe { *templ_ents_ptr.add(templ_ent_idx) };
            this.collision_tile_samples.push(CollisionTileSample {
                templ_ent,
                tile_origin: occupied_gpos,
                direction: this.card_at_gpos.resolve_tile_direction(&this.hash_id_query, templ_ent, occupied_gpos, CardinalDirection::default()),
                dead_despawning: false,
            });
        }
    }
}
