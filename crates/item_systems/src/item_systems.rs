use bevy::prelude::*;
use ac_input::player_action_requests::LocalItemPickupRequest;
use being_shared::Being;
use common::log_targets;
use game_common::game_common_components::{Dead, EntityZero, EntityZeroRef};
use item_shared::{clone_item_from_ezero, DroppedItem, HeldItems, Item, ItemHeldIn, ItemOperation, ItemsGeneratedOnDeath, KnownItemDest};
use sprite_shared::prelude::{HeldSprites, ScsToBuild};
use tilemap_shared::{DimensionRef, GlobalTilePos, ItemsAtGpos};

use crate::ItemGroundMaterializeParamSet;
pub fn on_being_held_items_changed(//chequear si caben todos en slots
    _query: Query<(), (Changed<HeldItems>, With<Being>)>,
) {
}

pub fn pick_up_locally_requested_items(
    mut cmd: Commands,
    mut pickup_requests: MessageReader<LocalItemPickupRequest>,
    items_at_gpos: Res<ItemsAtGpos>,
    being_query: Query<(&DimensionRef, &GlobalTilePos), With<Being>>,
    held_sprites_query: Query<&HeldSprites>,
    item_query: Query<(), DroppedItem>,
) {
    for &LocalItemPickupRequest { being_ent } in pickup_requests.read() {
        let Ok((&dim_ref, &gpos)) = being_query.get(being_ent) else {
            trace!(target: log_targets::ITEM_SYSTEM, "Skipping pickup request: missing being position for {:?}", being_ent);
            continue;
        };
        let Some(&item_ent) = items_at_gpos
            .items_at_pos(dim_ref, gpos)
            .iter()
            .find(|&&item_ent| item_query.get(item_ent).is_ok())
        else {
            trace!(target: log_targets::ITEM_SYSTEM, "Pickup request found no dropped item at dim={:?} gpos={:?} for {:?}", dim_ref.0, gpos, being_ent);
            continue;
        };
        if let Ok(held_sprites) = held_sprites_query.get(item_ent) {
            for &sprite_ent in held_sprites.entities() {
                cmd.entity(sprite_ent).try_despawn();
            }
        }
        cmd.entity(item_ent).try_insert(ItemHeldIn { holder: being_ent });
        cmd.entity(item_ent).try_remove::<(GlobalTilePos, ScsToBuild)>();
        debug!(target: log_targets::ITEM_SYSTEM, "Picked up item {:?} into being {:?} at dim={:?} gpos={:?}", item_ent, being_ent, dim_ref.0, gpos);
    }
}
#[allow(unused_parens, )]
pub fn readjust_child_of_for_items(
    mut cmd: Commands,
    held_query: Query<(Entity, &ItemHeldIn, Option<&ChildOf>), (With<Item>, Or<(Changed<ItemHeldIn>, Without<ChildOf>)>)>,
    dropped_query: Query<
        (Entity, &DimensionRef, Option<&ChildOf>),
        (DroppedItem, Or<(Changed<DimensionRef>, Without<ChildOf>)>),
    >,
) {
    let mut child_ofs_to_insert = Vec::new();
    for (item_ent, held_in, child_of) in held_query.iter() {
        if child_of.is_some_and(|child_of| child_of.parent() == held_in.holder) {
            continue;
        }
        child_ofs_to_insert.push((item_ent, ChildOf(held_in.holder)));
    }
    for (item_ent, dim_ref, child_of) in dropped_query.iter() {
        if let Some(child_of) = child_of {
            if child_of.parent() == dim_ref.0 {
                continue;
            }
        }
        child_ofs_to_insert.push((item_ent, ChildOf(dim_ref.0)));
    }
    cmd.try_insert_batch(child_ofs_to_insert);
}

pub fn execute_item_operations(
    mut cmd: Commands,
    mut item_operations: MessageReader<ItemOperation>,
    dim_ref_query: Query<&DimensionRef>,
    item_instance_query: Query<Option<&ItemHeldIn>, (With<Item>, Without<EntityZero>)>,
    ezero_item_query: Query<(), (With<Item>, With<EntityZero>)>,
    location_query: Query<(Option<&DimensionRef>, Option<&GlobalTilePos>, Option<&ChildOf>)>,
    mut materialize_params: ItemGroundMaterializeParamSet,
) {
    for &item_operation in item_operations.read() {
        match item_operation {
            ItemOperation::FromEzero(ezero_ref, KnownItemDest::Holder(target)) => {
                let Ok(&dim_ref) = dim_ref_query.get(target) else {
                    warn!(
                        target: log_targets::ITEM_SYSTEM,
                        "Skipping item op for ezero {:?}: target {:?} has no DimensionRef",
                        ezero_ref.0,
                        target,
                    );
                    continue;
                };
                let item_instance = clone_item_from_ezero(&mut cmd, ezero_ref, dim_ref);
                cmd.entity(item_instance).insert((ItemHeldIn { holder: target }, ChildOf(target)));
            }
            ItemOperation::FromEzero(ezero_ref, KnownItemDest::Ground(dim_ref, gpos)) => {
                materialize_params.materialize_item_on_ground_from_ezero(&mut cmd, None, ezero_ref, dim_ref, gpos);
            }
            ItemOperation::Preexisting(item, known_dest) => {
                if ezero_item_query.get(item).is_ok() {
                    warn!(
                        target: log_targets::ITEM_SYSTEM,
                        "Skipping preexisting item op for ezero {:?}; expected an item instance",
                        item,
                    );
                    continue;
                }
                let Ok(held_in) = item_instance_query.get(item) else {
                    warn!(
                        target: log_targets::ITEM_SYSTEM,
                        "Skipping preexisting item op: entity {:?} is not a valid item instance",
                        item,
                    );
                    continue;
                };
                match known_dest {
                    Some(KnownItemDest::Holder(target)) => {
                        let Ok(&dim_ref) = dim_ref_query.get(target) else {
                            warn!(
                                target: log_targets::ITEM_SYSTEM,
                                "Skipping preexisting item op for {:?}: holder {:?} has no DimensionRef",
                                item,
                                target,
                            );
                            continue;
                        };
                        cmd.entity(item).insert((dim_ref, ItemHeldIn { holder: target }, ChildOf(target)));
                    }
                    Some(KnownItemDest::Ground(dim_ref, gpos)) => {
                        cmd.entity(item).insert((dim_ref, gpos));
                        materialize_params.materialize_item_on_ground(&mut cmd, item);
                    }
                    None => {
                        let mut current = held_in.map(|held_in| held_in.holder).unwrap_or(item);
                        let found_pos = loop {
                            let Ok((dim_ref, gpos, child_of)) = location_query.get(current) else {
                                warn!(
                                    target: log_targets::ITEM_SYSTEM,
                                    "Skipping preexisting item op for {:?}: failed to inspect location chain at {:?}",
                                    item,
                                    current,
                                );
                                break None;
                            };
                            if let Some((&dim_ref, &gpos)) = dim_ref.zip(gpos) {
                                break Some((dim_ref, gpos));
                            }
                            let Some(child_of) = child_of else {
                                warn!(
                                    target: log_targets::ITEM_SYSTEM,
                                    "Skipping preexisting item op for {:?}: no DimensionRef/GlobalTilePos found in holder chain",
                                    item,
                                );
                                break None;
                            };
                            current = child_of.parent();
                        };
                        let Some((dim_ref, gpos)) = found_pos else {
                            continue;
                        };
                        cmd.entity(item).insert((dim_ref, gpos));
                        materialize_params.materialize_item_on_ground(&mut cmd, item);
                    }
                }
            }
        }
    }
}

pub fn generate_items_on_deaths(
    mut item_operation_writer: MessageWriter<ItemOperation>,
    query: Query<
        (
            Has<Dead>,
            Option<&DimensionRef>,
            Option<&GlobalTransform>,
            Option<&GlobalTilePos>,
            Option<&ItemsGeneratedOnDeath>,
            Option<&EntityZeroRef>,
        ),
        (Without<EntityZero>, Added<Dead>),
    >,
    ezero_drop_query: Query<&ItemsGeneratedOnDeath, With<EntityZero>>,
    mut item_operations: Local<Vec<ItemOperation>>,
) {
    let mut rng = rand::rng();
    for (is_dead, dim_ref, global_transform, tile_pos, generated_on_death, ezero_ref) in query.iter() {
        if !is_dead {
            continue;
        }
        let generated_on_death = generated_on_death
            .cloned()
            .or_else(|| ezero_ref.and_then(|ezero_ref| ezero_drop_query.get(ezero_ref.0).ok()).cloned());
        let Some(generated_on_death) = generated_on_death else {
            continue;
        };
        let Some(item_counts) = generated_on_death.sampler.sample_with_rng(&mut rng) else {
            info!(
                target: log_targets::ITEM_SYSTEM,
                "Entity death ({:?}) produced no sampled drops (tile_pos={:?}, dim_ref={:?})",
                ezero_ref.map(|e| e.0),
                tile_pos,
                dim_ref.map(|d| d.0),
            );
            continue;
        };
        info!(target: log_targets::ITEM_SYSTEM, "Entity death ({:?}) at {:?}: generating {} item types", ezero_ref.map(|e| e.0), tile_pos, item_counts.len());
        let drop_pos = global_transform
            .map(|transform| transform.translation().xy())
            .or_else(|| tile_pos.map(GlobalTilePos::to_pixelpos))
            .unwrap_or_default();
        let drop_gpos = tile_pos
            .copied()
            .unwrap_or_else(|| GlobalTilePos::from(drop_pos));
        let count_multiplier = generated_on_death.count_multiplier.max(0.0);
        let dim_ref = dim_ref.copied();

        for (item_ezero, base_count) in item_counts {
            let scaled_count = base_count as f32 * count_multiplier;
            let drop_count = scaled_count.round() as u32;
            if drop_count == 0 {
                info!(
                    target: log_targets::ITEM_SYSTEM,
                    " - Skipping {:?}: base_count={}, multiplier={}, rounded_drop_count=0",
                    item_ezero,
                    base_count,
                    count_multiplier,
                );
                continue;
            }
            if drop_count > 0 {
                info!(target: log_targets::ITEM_SYSTEM, " - Spawning {} x {:?} at {:?}", drop_count, item_ezero, drop_gpos);
            }
            for _ in 0..drop_count {
                let Some(dim_ref) = dim_ref else {
                    warn!(
                        target: log_targets::ITEM_SYSTEM,
                        "   -> missing DimensionRef on dead entity; ezero {:?} cannot be spawned on ground",
                        item_ezero,
                    );
                    continue;
                };
                info!(
                    target: log_targets::ITEM_SYSTEM,
                    "   -> queued item op dim={:?} gpos={:?} ezero={:?}",
                    dim_ref.0,
                    drop_gpos,
                    item_ezero,
                );
                item_operations.push(ItemOperation::spawn_on_ground(EntityZeroRef(item_ezero), dim_ref, drop_gpos));
            }
        }
    }
    item_operation_writer.write_batch(item_operations.drain(..));
}
