use bevy::prelude::*;
use bevy::ecs::entity::EntityHashMap;
use ac_input::player_action_requests::LocalItemPickupRequest;
use being_shared::Being;
use common::log_targets;
use game_common::game_common_components::{Dead, Templ, TemplEntiRef};
use ::item_shared::*;
use ::sprite_shared::{HeldSprites, ScsToBuild};
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
            for sprite_ent in held_sprites.iter() {
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
    held_query: Query<(Entity, &ItemHeldIn, Option<&ChildOf>), (Or<(Changed<ItemHeldIn>, Without<ChildOf>)>)>,
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

#[allow(unused_parens)]
pub fn sync_items_at_gpos(
    mut cmd: Commands,
    mut items_at_gpos: Option<ResMut<ItemsAtGpos>>,
    mut removed_items: RemovedComponents<Item>,
    mut tracked_pos: Local<EntityHashMap<(DimensionRef, GlobalTilePos)>>,
    query: Query<(Entity, &TemplEntiRef, Option<&DimensionRef>, Option<&Transform>, Option<&GlobalTilePos>, Has<ItemHeldIn>, Has<ScsToBuild>), (With<Item>, Without<Templ>)>,
    item_cfg_query: Query<&ItemSpritesConfig, (With<Item>, With<Templ>)>,
    sprite_cfg_hash_query: Query<&common::common_components::HashId, (With<::sprite_shared::SpriteConfig>, common::AnyDisabling)>,
) {
    let Some(items_at_gpos) = items_at_gpos.as_mut() else {
        tracked_pos.clear();
        return;
    };
    let mut gposes_to_insert = Vec::new();
    let mut gposes_to_remove = Vec::new();

    for item_ent in removed_items.read() {
        let Some((old_dim, old_gpos)) = tracked_pos.remove(&item_ent) else {
            continue;
        };
        items_at_gpos.remove_item(old_dim, old_gpos, item_ent);
    }

    for (item_ent, templ_ref, dim_ref, transform, curr_gpos, is_held, has_scs_to_build) in query.iter() {
        if is_held {
            let Some((old_dim, old_gpos)) = tracked_pos.remove(&item_ent) else {
                if curr_gpos.is_some() {
                    gposes_to_remove.push(item_ent);
                }
                continue;
            };
            items_at_gpos.remove_item(old_dim, old_gpos, item_ent);
            if curr_gpos.is_some() {
                gposes_to_remove.push(item_ent);
            }
            continue;
        }

        let Some(&dim_ref) = dim_ref else {
            let Some((old_dim, old_gpos)) = tracked_pos.remove(&item_ent) else {
                if curr_gpos.is_some() {
                    gposes_to_remove.push(item_ent);
                }
                continue;
            };
            items_at_gpos.remove_item(old_dim, old_gpos, item_ent);
            if curr_gpos.is_some() {
                gposes_to_remove.push(item_ent);
            }
            continue;
        };
        let Some(gpos) = curr_gpos
            .copied()
            .or_else(|| transform.map(|transform| GlobalTilePos::from(transform.translation.xy())))
        else {
            let Some((old_dim, old_gpos)) = tracked_pos.remove(&item_ent) else {
                continue;
            };
            items_at_gpos.remove_item(old_dim, old_gpos, item_ent);
            continue;
        };
        if !has_scs_to_build {
            if let Some(scs_to_build) = dropped_scs_to_build(&item_cfg_query, &sprite_cfg_hash_query, *templ_ref) {
                cmd.entity(item_ent).insert(scs_to_build);
            }
        }
        if curr_gpos.copied() != Some(gpos) {
            gposes_to_insert.push((item_ent, gpos));
        }
        let Some((old_dim, old_gpos)) = tracked_pos.get(&item_ent).copied() else {
            tracked_pos.insert(item_ent, (dim_ref, gpos));
            items_at_gpos.insert_item(dim_ref, gpos, item_ent);
            continue;
        };
        if old_dim == dim_ref && old_gpos == gpos {
            continue;
        }
        items_at_gpos.remove_item(old_dim, old_gpos, item_ent);
        items_at_gpos.insert_item(dim_ref, gpos, item_ent);
        tracked_pos.insert(item_ent, (dim_ref, gpos));
    }
    cmd.try_insert_batch(gposes_to_insert);
    for item_ent in gposes_to_remove {
        cmd.entity(item_ent).try_remove::<GlobalTilePos>();
    }
}

pub fn execute_item_operations(
    mut cmd: Commands,
    mut item_operations: MessageReader<ItemOperation>,
    dim_ref_query: Query<&DimensionRef>,
    item_instance_query: Query<&ItemHeldIn, (With<Item>, Without<Templ>)>,
    templ_item_query: Query<(), (With<Item>, With<Templ>)>,
    child_of_query: Query<&ChildOf>,
    mut materialize_params: ItemGroundMaterializeParamSet,
) {
    for &item_operation in item_operations.read() {
        match item_operation {
            ItemOperation::FromTempl(templ_ref, KnownItemDest::Holder(target)) => {
                let Ok(&dim_ref) = dim_ref_query.get(target) else {
                    warn!(
                        target: log_targets::ITEM_SYSTEM,
                        "Skipping item op for templ {:?}: target {:?} has no DimensionRef",
                        templ_ref.0,
                        target,
                    );
                    continue;
                };
                let item_instance = clone_item_from_templ(&mut cmd, templ_ref, dim_ref);
                cmd.entity(item_instance).try_insert((ItemHeldIn { holder: target }, ChildOf(target)));
            }
            ItemOperation::FromTempl(templ_ref, KnownItemDest::Ground(dim_ref, gpos)) => {
                materialize_params.materialize_item_on_ground_from_templ(&mut cmd, None, templ_ref, dim_ref, gpos);
            }
            ItemOperation::Preexisting(item, known_dest) => {
                if templ_item_query.get(item).is_ok() {
                    warn!(
                        target: log_targets::ITEM_SYSTEM,
                        "Skipping preexisting item op for templ {:?}; expected an item instance",
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
                        cmd.entity(item).try_insert((dim_ref, ItemHeldIn { holder: target }, ChildOf(target)));
                    }
                    Some(KnownItemDest::Ground(dim_ref, gpos)) => {
                        cmd.entity(item).insert((dim_ref, gpos));
                        materialize_params.materialize_item_on_ground(&mut cmd, item);
                    }
                    None => {
                        let mut current = held_in.holder;
                        let found_pos = loop {
                            let Ok(child_of) = child_of_query.get(current) else {
                                warn!(
                                    target: log_targets::ITEM_SYSTEM,
                                    "Skipping preexisting item op for {:?}: failed to inspect location chain at {:?}",
                                    item,
                                    current,
                                );
                                break None;
                            };
                            let Ok(&dim_ref) = dim_ref_query.get(current) else {
                                current = child_of.parent();
                                continue;
                            };
                            let Some(gpos) = materialize_params.get_gpos(current) else {
                                current = child_of.parent();
                                continue;
                            };
                            break Some((dim_ref, gpos));
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
            Option<&DimensionRef>,
            Option<&GlobalTransform>,
            Option<&GlobalTilePos>,
            Option<&ItemsGeneratedOnDeath>,
            Option<&TemplEntiRef>,
        ),
        (Without<Templ>, Added<Dead>),
    >,
    templ_drop_query: Query<&ItemsGeneratedOnDeath, With<Templ>>,
    mut item_operations: Local<Vec<ItemOperation>>,
) {
    let mut rng = rand::rng();
    for (dim_ref, global_transform, tile_pos, generated_on_death, templ_ref) in query.iter() {
        let generated_on_death = generated_on_death
            .cloned()
            .or_else(|| templ_ref.and_then(|templ_ref| templ_drop_query.get(templ_ref.0).ok()).cloned());
        let Some(generated_on_death) = generated_on_death else {
            continue;
        };
        let Some(item_counts) = generated_on_death.sampler.sample_with_rng(&mut rng) else {
            info!(
                target: log_targets::ITEM_SYSTEM,
                "Entity death ({:?}) produced no sampled drops (tile_pos={:?}, dim_ref={:?})",
                templ_ref.map(|e| e.0),
                tile_pos,
                dim_ref.map(|d| d.0),
            );
            continue;
        };
        info!(target: log_targets::ITEM_SYSTEM, "Entity death ({:?}) at {:?}: generating {} item types", templ_ref.map(|e| e.0), tile_pos, item_counts.len());
        let drop_pos = global_transform
            .map(|transform| transform.translation().xy())
            .or_else(|| tile_pos.map(GlobalTilePos::to_pixelpos))
            .unwrap_or_default();
        let drop_gpos = tile_pos
            .copied()
            .unwrap_or_else(|| GlobalTilePos::from(drop_pos));
        let count_multiplier = generated_on_death.count_multiplier.max(0.0);
        let dim_ref = dim_ref.copied();

        for (item_templ, base_count) in item_counts {
            let scaled_count = base_count as f32 * count_multiplier;
            let drop_count = scaled_count.round() as u32;
            if drop_count == 0 {
                info!(
                    target: log_targets::ITEM_SYSTEM,
                    " - Skipping {:?}: base_count={}, multiplier={}, rounded_drop_count=0",
                    item_templ,
                    base_count,
                    count_multiplier,
                );
                continue;
            }
            if drop_count > 0 {
                info!(target: log_targets::ITEM_SYSTEM, " - Spawning {} x {:?} at {:?}", drop_count, item_templ, drop_gpos);
            }
            for _ in 0..drop_count {
                let Some(dim_ref) = dim_ref else {
                    warn!(
                        target: log_targets::ITEM_SYSTEM,
                        "   -> missing DimensionRef on dead entity; templ {:?} cannot be spawned on ground",
                        item_templ,
                    );
                    continue;
                };
                info!(
                    target: log_targets::ITEM_SYSTEM,
                    "   -> queued item op dim={:?} gpos={:?} templ={:?}",
                    dim_ref.0,
                    drop_gpos,
                    item_templ,
                );
                item_operations.push(ItemOperation::spawn_on_ground(TemplEntiRef(item_templ), dim_ref, drop_gpos));
            }
        }
    }
    item_operation_writer.write_batch(item_operations.drain(..));
}
