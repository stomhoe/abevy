use crate::{
    tile::{tile_bundles::TileMassSpawnBundle, tile_components::*},
    tilemap_resources::*,
};
use ::sprite_shared::*;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use common::common_tag_components::TagSet;
use game_common::game_common_components::*;
use ::tilemap_shared::*;

pub fn temp_tile_mass_spawn_bundle(ezero_ref: EntityZeroRef, dim_ref: DimensionRef, gpos: GlobalTilePos) -> TileMassSpawnBundle {
    TileMassSpawnBundle {
        ezero_ref,
        gpos,
        snap_to_gpos: SnapTransformToGpos::OnChange,
        dim_ref,
        tile_bundle: TileBundle::default(),
        initial_pos: InitialPos::default(),
    }
}

#[derive(SystemParam)]
pub struct TileDeleteOthersParamSet<'w, 's> {
    pub tile_ezero_ref_query: Query<'w, 's, &'static EntityZeroRef>,
    pub gpos_query: Query<'w, 's, &'static GlobalTilePos>,
    pub z_query: Query<'w, 's, &'static AcZ, common::AnyDisabling>,
    pub size_query: Query<'w, 's, &'static SizeInTiles, common::AnyDisabling>,
    pub ezero_delete_query: Query<'w, 's, &'static DeleteOtherTilesInSamePos>,
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
        bundle: &TileMassSpawnBundle,
        newtile_delete_others_excp: Option<&DeleteOtherTilesInSamePos>,
        newtile_tags: Option<&TagSet>,
    ) {
        let dim = bundle.dim_ref;
        let gpos = bundle.gpos;
        let ezero_ref = bundle.ezero_ref;
        let Ok(newtile_z) = self.z_query.get(ezero_ref.0) else {
            warn_once!(target: common::DEBUG_TILE, "Failed to get AcZ for tile entity {:?}, skipping despawn check", newtile_ent);
            return;
        };
        let Ok(newtile_size) = self.size_query.get(ezero_ref.0) else {
            warn_once!(target: common::DEBUG_TILE, "Failed to get SizeInTiles for tile entity {:?}, skipping despawn check", newtile_ent);
            return;
        };
        let scan_radius = newtile_delete_others_excp.map(|s| s.extra_radius as i32).unwrap_or_default();
        let scan_origin = gpos + newtile_delete_others_excp.map(|s| s.displacement).unwrap_or_default();
        let newtile_size = newtile_size.inner().as_ivec2();
        self.checked_ents.clear();
        for y in (scan_origin.0.y - scan_radius)..=(scan_origin.0.y + newtile_size.y - 1 + scan_radius) {
            for x in (scan_origin.0.x - scan_radius)..=(scan_origin.0.x + newtile_size.x - 1 + scan_radius) {
                gather_params.gather_tiles_at(&mut * self.checked_ents, dim, GlobalTilePos::new(x, y));
            }
        }
        self.checked_ents.drain(..).for_each(|otile_ent| {
            if otile_ent == newtile_ent {
                return;
            }
            let (Ok(otile_ezero_ref), Ok(&otile_gpos)) = (
                self.tile_ezero_ref_query.get(otile_ent),
                self.gpos_query.get(otile_ent),
            ) else {
                trace!(target: "tilemap", "Failed to get prev tile entity {:?}, skipping despawn check", otile_ent);
                return;
            };
            let Ok(otile_z) = self.z_query.get(otile_ezero_ref.0) else {
                trace!(target: "tilemap", "Failed to get AcZ for tile entity {:?}, skipping despawn check", otile_ent);
                return;
            };
            let Ok(otile_size) = self.size_query.get(otile_ezero_ref.0) else {
                trace!(target: "tilemap", "Failed to get SizeInTiles for tile entity {:?}, skipping despawn check", otile_ent);
                return;
            };
            let ezero_otile_delete_others_excp = self.ezero_delete_query.get(otile_ezero_ref.0).ok();
            if let Some(newtile_delete_others_excp) = newtile_delete_others_excp {
                let otile_tags = self.tag_set_query.get(otile_ent).ok().or_else(|| self.tag_set_query.get(otile_ezero_ref.0).ok());
                if should_delete_tile_based_on_tag_sets(newtile_delete_others_excp, otile_z, otile_tags) {
                    trace!(target: "tilemap", "Despawning tile entity {:?} at gpos {:?} in dimension {:?} due to new tile entity {:?}", otile_ent, otile_gpos, dim, newtile_ent);
                    if !is_any_occupied_pos_registered(registered_positions, *otile_ezero_ref, dim, otile_gpos, otile_size.inner().as_ivec2()) && !registered_positions.exempted.contains(&otile_ent) {
                        self.msgs.push(SafeDespawn(otile_ent));
                    }
                    return;
                }
            }
            let otile_delete_others_excp = self.ezero_delete_query.get(otile_ent).ok().or(ezero_otile_delete_others_excp);
            if let Some(otile_delete_others_excp) = otile_delete_others_excp {
                if should_delete_tile_based_on_tag_sets(otile_delete_others_excp, newtile_z, newtile_tags) {
                    trace!(target: "tilemap", "Despawning tile entity {:?} at gpos {:?} in dimension {:?} due to old tile entity {:?}", newtile_ent, gpos, dim, otile_ent);
                    if !is_any_occupied_pos_registered(registered_positions, ezero_ref, dim, gpos, newtile_size) && !registered_positions.exempted.contains(&newtile_ent) {
                        self.msgs.push(SafeDespawn(newtile_ent));
                    }
                }
            }
        });
    }
}

fn is_any_occupied_pos_registered(
    registered_positions: &ImportantRegisteredPositions,
    ezero_ref: EntityZeroRef,
    dim: DimensionRef,
    anchor_gpos: GlobalTilePos,
    size: IVec2,
) -> bool {
    for y in anchor_gpos.0.y..(anchor_gpos.0.y + size.y) {
        for x in anchor_gpos.0.x..(anchor_gpos.0.x + size.x) {
            if registered_positions.is_pos_registered(ezero_ref, dim, GlobalTilePos::new(x, y)) {
                return true;
            }
        }
    }
    false
}

#[allow(clippy::too_many_arguments)]
pub fn process_tile_despawns_from_ezero(
    paramset: &mut TileDeleteOthersParamSet,
    registered_positions: &ImportantRegisteredPositions,
    gather_params: &TileGatheringParamSet,
    newtile_ent: Entity,
    bundle: &TileMassSpawnBundle,
) {
    let Some(newtile_delete_others_excp) = paramset.ezero_delete_query.get(bundle.ezero_ref.0).ok().cloned() else {
        paramset.process_tile_despawns(registered_positions, gather_params, newtile_ent, bundle, None, None);
        return;
    };
    let Some(newtile_tags) = paramset.tag_set_query.get(bundle.ezero_ref.0).ok().map(|tags| tags as *const TagSet) else {
        paramset.process_tile_despawns(registered_positions, gather_params, newtile_ent, bundle, Some(&newtile_delete_others_excp), None);
        return;
    };
    unsafe {
        paramset.process_tile_despawns(
            registered_positions,
            gather_params,
            newtile_ent,
            bundle,
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
    bundle: &TileMassSpawnBundle,
    newtile_delete_others_excp: &DeleteOtherTilesInSamePos,
    newtile_tags: Option<&TagSet>,
) {
    paramset.process_tile_despawns(
        registered_positions,
        gather_params,
        newtile_ent,
        bundle,
        Some(newtile_delete_others_excp),
        newtile_tags,
    );
}

pub fn tile_is_pending_despawn(msgs: &Local<Vec<SafeDespawn>>, ent: Entity) -> bool {
    msgs.iter().any(|msg| msg.0 == ent)
}

fn should_delete_tile_based_on_tag_sets(
    spec: &DeleteOtherTilesInSamePos,
    target_z: &AcZ,
    target_tags: Option<&TagSet>,
) -> bool {
    if !spec.targeted_z.is_empty() {
        if !spec.targeted_z.contains(target_z) {
            return false;
        }
        if let Some(tags) = target_tags {
            if spec.spared_tags.intersects(tags) {
                return false;
            }
        }
        return true;
    }
    if !spec.targeted_tags.is_empty() {
        let Some(tags) = target_tags else {
            return false;
        };
        if !spec.targeted_tags.intersects(tags) {
            return false;
        }
        if spec.spared_tags.intersects(tags) {
            return false;
        }
        if spec.spared_z.contains(target_z) {
            return false;
        }
        return true;
    }
    if spec.spared_z.contains(target_z) {
        return false;
    }
    if let Some(tags) = target_tags {
        if spec.spared_tags.intersects(tags) {
            return false;
        }
    }
    true
}
