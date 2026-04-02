use bevy::ecs::schedule::IntoScheduleConfigs;
use bevy::prelude::*;
use bevy::ecs::schedule::common_conditions::on_message;
use bevy_replicon::prelude::AppRuleExt;
use ac_input::player_action_requests::LocalItemPickupRequest;
use common::common_states::AssetLoading;
use game_common::HostSystems;
use game_common::game_common::GameplaySystems;
use ::item_shared::*;

use crate::item_init_systems::*;
use crate::item_systems::*;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct ItemSystems;

pub fn plugin(app: &mut App) {
    app.add_plugins((plugin_item,))
        .add_systems(
            OnEnter(AssetLoading::SpawnReplicatedEntities),
            (init_items, map_item_id_to_entity).chain().in_set(ItemSystems),
        )
        .add_systems(
            Update,
            (
                execute_item_operations
                    .run_if(on_message::<ItemOperation>),
                pick_up_locally_requested_items
                    .run_if(on_message::<LocalItemPickupRequest>)
                    .in_set(HostSystems)
                ,
                readjust_child_of_for_items,
                sync_items_at_gpos,
                on_being_held_items_changed,
                generate_items_on_deaths,
            ).in_set(GameplaySystems),
        )
        .add_message::<ItemOperation>()
        .replicate::<Item>()
        .replicate::<ItemHeldIn>()
        .replicate::<DropHeldItemsOnDowned>()
        .replicate::<SlotableIn>()
        .replicate_filtered::<ChildOf, With<Item>>()
    ;
}
