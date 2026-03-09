use bevy::prelude::*;
use bevy_replicon::prelude::*;
use bevy::ecs::entity::EntityHashMap;
use common::common_states::AssetLoading;
use game_common::game_common::GameplaySystems;
use game_common::game_common_components::{EntityZero, EntityZeroRef};
use ::item_shared::*;
use sprite::prelude::ScsToBuild;
use tilemap_shared::{DimensionRef, GlobalTilePos, ItemsAtGpos};

use crate::item_init_systems::*;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct ItemSystems;

pub fn clone_item_from_ezero(cmd: &mut Commands, ezero_ref: EntityZeroRef) -> Entity {
    let item_instance = cmd
        .entity(ezero_ref.0)
        .clone_and_spawn_with_opt_out(|builder| {
            builder.deny::<ToDenyOnItemClone>();
        })
        .id();
    cmd.entity(item_instance)
        .insert((Item, EntityZeroRef(ezero_ref.0)));
    item_instance
}

pub fn plugin(app: &mut App) {
    app.add_plugins((plugin_item,))
        .add_systems(
            OnEnter(AssetLoading::SpawnReplicatedEntities),
            (init_items, map_item_id_to_entity).chain().in_set(ItemSystems),
        )
        .add_systems(
            Update,
            sync_items_at_gpos.in_set(GameplaySystems),
        )
        .replicate::<Item>()
        .replicate::<ItemHeldIn>()
        .replicate::<DropHeldItemsOnDowned>()
        .replicate_filtered::<ChildOf, With<Item>>()
    ;
}

#[allow(unused_parens)]
pub fn sync_items_at_gpos(
    mut cmd: Commands,
    mut items_at_gpos: Option<ResMut<ItemsAtGpos>>,
    mut removed_items: RemovedComponents<Item>,
    mut tracked_pos: Local<EntityHashMap<(DimensionRef, GlobalTilePos)>>,
    query: Query<(Entity, &EntityZeroRef, Option<&DimensionRef>, Option<&Transform>, Option<&GlobalTilePos>, Has<ItemHeldIn>, Has<ScsToBuild>), (With<Item>, Without<EntityZero>)>,
    item_cfg_query: Query<&ItemSpritesConfig, (With<Item>, With<EntityZero>)>,
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

    for (item_ent, ezero_ref, dim_ref, transform, curr_gpos, is_held, has_scs_to_build) in query.iter() {
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
        let Some(transform) = transform else {
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

        let gpos = GlobalTilePos::from(transform.translation.xy());
        if !has_scs_to_build {
            let dropped_sprite_cfg = item_cfg_query
                .get(ezero_ref.0)
                .ok()
                .and_then(|cfg| {
                    if cfg.dropped_sprite_cfg.0 != Entity::PLACEHOLDER {
                        return Some(cfg.dropped_sprite_cfg.0);
                    }
                    if cfg.icon_sprite_cfg.0 != Entity::PLACEHOLDER {
                        return Some(cfg.icon_sprite_cfg.0);
                    }
                    None
                });
            if let Some(cfg_ent) = dropped_sprite_cfg {
                let mut scs_to_build = ScsToBuild::with_capacity(1);
                scs_to_build.0.insert(cfg_ent);
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
