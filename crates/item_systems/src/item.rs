use bevy::prelude::*;
use bevy_replicon::prelude::*;
use common::common_states::AssetLoading;
use game_common::game_common_components::EntityZeroRef;
use ::item_shared::*;

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
        .replicate::<Item>()
        .replicate::<ItemHeldIn>()
        .replicate::<DropHeldItemsOnDowned>()
        .replicate_filtered::<ChildOf, With<Item>>()
    ;
}
