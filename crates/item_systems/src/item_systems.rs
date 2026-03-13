use bevy::ecs::entity::EntityHashMap;
use bevy::prelude::*;
use ac_input::player_action_requests::LocalItemPickupRequest;
use being_shared::Being;
use common::log_targets;
use game_common::game_common_components::{Dead, EntityZero, EntityZeroRef};
use item_shared::{DroppedItem, HeldItems, Item, ItemHeldIn, ItemsGeneratedOnDeath};
use modifier_shared::modifier_components::{CurrEffectiveValue, ModifierTarget};
use modifier_shared::modifier_item_types::StackLimit;
use param_sets::BlockingTileParamSet;
use sprite_shared::prelude::{AcZ, HeldSprites, ScsToBuild};
use bevy::ecs::entity::EntityHashSet;
use std::collections::HashSet;
use tilemap_shared::{DimensionRef, GlobalTilePos, ItemsAtGpos, TileGatheringParamSet};

use crate::{clone_item_from_ezero, item_helpers::*, item_messages::*};

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

pub fn generate_items_from_messages(
    mut cmd: Commands,
    mut generate_items: MessageReader<GenerateItem>,
    mut removed_item_held_in: RemovedComponents<ItemHeldIn>,
    tile_gathering: TileGatheringParamSet,
    item_cfg_query: Query<&item_shared::ItemSpritesConfig, (With<Item>, With<EntityZero>)>,
    dropped_lookup: Query<(Entity, &EntityZeroRef, &DimensionRef, &GlobalTilePos), DroppedItem>,
    ac_z_query: Query<&AcZ>,
    mut to_drain: Local<Vec<Entity>>,
) {
    for &GenerateItem { ezero_ref, dim_ref, dest } in generate_items.read() {
        match dest {
            GenerateItemDest::Gpos(gpos) => {
                let item_instance = clone_item_from_ezero(&mut cmd, ezero_ref, dim_ref);
                materialize_item_on_ground(
                    &mut cmd,
                    &item_cfg_query,
                    &tile_gathering,
                    &ac_z_query,
                    &mut to_drain,
                    item_instance,
                    ezero_ref,
                    (dim_ref, gpos),
                );
            }
            GenerateItemDest::Entity(target) => {
                let item_instance = clone_item_from_ezero(&mut cmd, ezero_ref, dim_ref);
                cmd.entity(item_instance).insert((
                    ItemHeldIn { holder: target },
                    ChildOf(target),
                ));
            }
        }
    }

    for item_ent in removed_item_held_in.read() {
        let Ok((item_ent, ezero_ref, dim_ref, gpos)) = dropped_lookup.get(item_ent) else {
            continue;
        };
        materialize_item_on_ground(
            &mut cmd,
            &item_cfg_query,
            &tile_gathering,
            &ac_z_query,
            &mut to_drain,
            item_ent,
            *ezero_ref,
            (*dim_ref, *gpos),
        );
    }
}

pub fn generate_items_on_deaths(
    blocking_tiles: BlockingTileParamSet,
    mut generate_item_writer: MessageWriter<GenerateItem>,
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
    stack_limit_query: Query<(&ModifierTarget, &CurrEffectiveValue), With<StackLimit>>,
    mut generate_item_messages: Local<Vec<GenerateItem>>,
    mut to_drain: Local<Vec<Entity>>,
    mut occupied_nonstackable: Local<HashSet<GlobalTilePos>>,
) {
    let mut stack_limit_by_item = EntityHashMap::default();
    for (&ModifierTarget(target), &CurrEffectiveValue(limit)) in stack_limit_query.iter() {
        stack_limit_by_item.insert(target, limit.max(1.0));
    }
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
        occupied_nonstackable.clear();
        let count_multiplier = generated_on_death.count_multiplier.max(0.0);
        let dim_ref = dim_ref.copied();
        let stackable_drop_gpos = find_stackable_drop_gpos(&blocking_tiles, &mut to_drain, dim_ref, drop_gpos);

        for (item_ezero, base_count) in item_counts {
            let stack_limit = stack_limit_by_item.get(&item_ezero).copied().unwrap_or(1.0);
            let is_nonstackable = stack_limit <= 1.0;
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
                let spawn_gpos = if is_nonstackable {
                    find_nonstackable_drop_gpos(&blocking_tiles, &mut to_drain, &mut occupied_nonstackable, dim_ref, drop_gpos)
                } else {
                    stackable_drop_gpos
                };
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
                    "   -> queued GenerateItem dim={:?} gpos={:?} ezero={:?}",
                    dim_ref.0,
                    spawn_gpos,
                    item_ezero,
                );
                generate_item_messages.push(GenerateItem::on_ground(EntityZeroRef(item_ezero), dim_ref, spawn_gpos));
            }
        }
    }
    generate_item_writer.write_batch(generate_item_messages.drain(..));
}
