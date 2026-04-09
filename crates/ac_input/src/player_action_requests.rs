use bevy::prelude::*;
use bevy_replicon::prelude::ClientMessageAppExt;

crate::define_player_action_request_module!(
    base: MeleeAttack,
    extra_query: (),
    extra_binding: _,
    log_target: common::log_targets::BEING_SYSTEM,
);

crate::define_player_action_request_module!(
    base: ItemPickup,
    extra_query: (&tilemap_shared::DimensionRef, &tilemap_shared::GlobalTilePos),
    extra_binding: (_dim_ref, _gpos),
    log_target: common::log_targets::ITEM_SYSTEM,
);

crate::define_player_action_request_module!(
    base: DebugIncreaseSpeed,
    extra_query: (),
    extra_binding: _,
    log_target: common::log_targets::DEBUG,
    continuous: true,
);

crate::define_player_action_request_module!(
    base: DebugDecreaseSpeed,
    extra_query: (),
    extra_binding: _,
    log_target: common::log_targets::DEBUG,
    continuous: true,
);
