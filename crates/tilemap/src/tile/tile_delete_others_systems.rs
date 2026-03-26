use ::sprite_shared::*;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use common::common_tag_components::TagSet;
use game_common::game_common_components::*;
use ::tilemap_shared::{*, DeleteOtherTilesInSamePos};

#[derive(SystemParam)]
pub struct TileDeleteOthersParamSet<'w, 's> {
    pub tile_templ_ref_query: Query<'w, 's, &'static TemplEntiRef>,
    pub gpos_query: Query<'w, 's, &'static GlobalTilePos>,
    pub z_query: Query<'w, 's, &'static AcZ, common::AnyDisabling>,
    pub size_query: Query<'w, 's, &'static SizeInTiles, common::AnyDisabling>,
    pub templ_delete_query: Query<'w, 's, &'static DeleteOtherTilesInSamePos>,
    pub tag_set_query: Query<'w, 's, &'static TagSet, common::AnyDisabling>,
    pub checked_ents: Local<'s, Vec<Entity>>,
    pub msgs: Local<'s, Vec<SafeDespawn>>,
}

impl<'w, 's> TileDeleteOthersParamSet<'w, 's> {
    #[allow(clippy::too_many_arguments)]
    pub fn process_tile_despawns(
        &mut self,
        registered_positions: &ImportantRegisteredPositions,
        gather_params: &TileGatheringParamSet,
        newtile_ent: Entity,
        templ_ref: TemplEntiRef,
        dim: DimensionRef,
        gpos: GlobalTilePos,
        newtile_delete_others_excp: Option<&DeleteOtherTilesInSamePos>,
        newtile_tags: Option<&TagSet>,
    ) -> bool {
        let Ok(newtile_z) = self.z_query.get(templ_ref.0) else {
            warn_once!(target: common::DEBUG_TILE, "Failed to get AcZ for tile entity {:?}, skipping despawn check", newtile_ent);
            return false;
        };
        let Ok(newtile_size) = self.size_query.get(templ_ref.0) else {
            warn_once!(target: common::DEBUG_TILE, "Failed to get SizeInTiles for tile entity {:?}, skipping despawn check", newtile_ent);
            return false;
        };
        let scan_radius = newtile_delete_others_excp.map(|s| s.extra_radius as i32).unwrap_or_default();
        let scan_origin = gpos + newtile_delete_others_excp.map(|s| s.displacement).unwrap_or_default();
        let newtile_size = newtile_size.inner().as_ivec2();
        self.checked_ents.clear();
        for y in (scan_origin.0.y - scan_radius)..=(scan_origin.0.y + newtile_size.y - 1 + scan_radius) {
            for x in (scan_origin.0.x - scan_radius)..=(scan_origin.0.x + newtile_size.x - 1 + scan_radius) {
                gather_params.gather_tiles_extend(&mut * self.checked_ents, dim, GlobalTilePos::new(x, y));
            }
        }
        for otile_ent in self.checked_ents.drain(..) {
            if otile_ent == newtile_ent {
                continue;
            }
            let (Ok(otile_templ_ref), Ok(&otile_gpos)) = (
                self.tile_templ_ref_query.get(otile_ent),
                self.gpos_query.get(otile_ent),
            ) else {
                trace!(target: "tilemap", "Failed to get prev tile entity {:?}, skipping despawn check", otile_ent);
                continue;
            };
            let Ok(otile_z) = self.z_query.get(otile_templ_ref.0) else {
                trace!(target: "tilemap", "Failed to get AcZ for tile entity {:?}, skipping despawn check", otile_ent);
                continue;
            };
            let Ok(otile_size) = self.size_query.get(otile_templ_ref.0) else {
                trace!(target: "tilemap", "Failed to get SizeInTiles for tile entity {:?}, skipping despawn check", otile_ent);
                continue;
            };
            let templ_otile_delete_others_excp = self.templ_delete_query.get(otile_templ_ref.0).ok();
            if let Some(newtile_delete_others_excp) = newtile_delete_others_excp {
                let otile_tags = self.tag_set_query.get(otile_ent).ok().or_else(|| self.tag_set_query.get(otile_templ_ref.0).ok());
                if newtile_delete_others_excp.should_delete_tile_based_on_tag_sets(otile_z, otile_tags) {
                    trace!(target: "tilemap", "Despawning tile entity {:?} at gpos {:?} in dimension {:?} due to new tile entity {:?}", otile_ent, otile_gpos, dim, newtile_ent);
                    if !registered_positions.is_any_occupied_pos_registered(otile_templ_ref.0, dim, otile_gpos, otile_size.inner().as_ivec2()) && !registered_positions.get_exempted_tile_ents().contains(&otile_ent) {
                        self.msgs.push(SafeDespawn(otile_ent));
                    }
                    return true;
                }
            }
            let otile_delete_others_excp = self.templ_delete_query.get(otile_ent).ok().or(templ_otile_delete_others_excp);
            if let Some(otile_delete_others_excp) = otile_delete_others_excp {
                if otile_delete_others_excp.should_delete_tile_based_on_tag_sets(newtile_z, newtile_tags) {
                    trace!(target: "tilemap", "Despawning tile entity {:?} at gpos {:?} in dimension {:?} due to old tile entity {:?}", newtile_ent, gpos, dim, otile_ent);
                    if !registered_positions.is_any_occupied_pos_registered(templ_ref.0, dim, gpos, newtile_size) && !registered_positions.get_exempted_tile_ents().contains(&newtile_ent) {
                        self.msgs.push(SafeDespawn(newtile_ent));
                    }
                }
            }
        }
        false
    }
}

#[allow(clippy::too_many_arguments)]
pub fn process_tile_despawns_from_templ(
    paramset: &mut TileDeleteOthersParamSet,
    registered_positions: &ImportantRegisteredPositions,
    gather_params: &TileGatheringParamSet,
    newtile_ent: Entity,
    templ_ref: TemplEntiRef,
    dim: DimensionRef,
    gpos: GlobalTilePos,
) -> bool {
    let Some(newtile_delete_others_excp) = paramset.templ_delete_query.get(templ_ref.0).ok().cloned() else {
        return paramset.process_tile_despawns(registered_positions, gather_params, newtile_ent, templ_ref, dim, gpos, None, None);
    };
    let Some(newtile_tags) = paramset.tag_set_query.get(templ_ref.0).ok().map(|tags| tags as *const TagSet) else {
        return paramset.process_tile_despawns(registered_positions, gather_params, newtile_ent, templ_ref, dim, gpos, Some(&newtile_delete_others_excp), None);
    };
    unsafe {
        return paramset.process_tile_despawns(
            registered_positions,
            gather_params,
            newtile_ent,
            templ_ref,
            dim,
            gpos,
            Some(&newtile_delete_others_excp),
            Some(&*newtile_tags),
        );
    }
}

#[allow(clippy::too_many_arguments)]
pub fn process_tile_despawns_from_added_delete_others(
    paramset: &mut TileDeleteOthersParamSet,
    registered_positions: &ImportantRegisteredPositions,
    gather_params: &TileGatheringParamSet,
    newtile_ent: Entity,
    templ_ref: TemplEntiRef,
    dim: DimensionRef,
    gpos: GlobalTilePos,
    newtile_delete_others_excp: &DeleteOtherTilesInSamePos,
    newtile_tags: Option<&TagSet>,
) -> bool {
    paramset.process_tile_despawns(
        registered_positions,
        gather_params,
        newtile_ent,
        templ_ref,
        dim,
        gpos,
        Some(newtile_delete_others_excp),
        newtile_tags,
    )
}
