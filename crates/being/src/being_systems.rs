use ::being_shared::*;
use ::tilemap_shared::GlobalTilePos;
use bevy::{ecs::{entity::EntityHashSet, entity_disabling::Disabled}, prelude::*};
use common::common_components::StrId;
use common::log_targets::BEING_SYSTEM;

use crate::{
    being_components::*,
};

#[allow(unused_parens, )]
pub fn validate_added_beings_have_gpos(
    query: Query<(Entity, Option<&StrId>, Has<GlobalTilePos>, ), (LoadedBeing, ),>,
    added_being: Query<Entity, (Added<Being>, )>,
    mut removed_disabled: RemovedComponents<Disabled>,
    mut removed_unloaded: RemovedComponents<Unloaded>,
    mut to_iter: Local<EntityHashSet>,
) {
    to_iter.extend(added_being.iter());
    to_iter.extend(removed_disabled.read());
    to_iter.extend(removed_unloaded.read());
    for (ent, str_id, has_gpos, ) in query.iter_many(to_iter.drain()) {
        if has_gpos {
            continue;
        }
        error_once!(
            target: BEING_SYSTEM,
            "Added Being {:?} {} missing required components: GlobalTilePos={}",
            ent,
            str_id.map(StrId::as_str).unwrap_or("<no-strid>"),
            has_gpos,
        );
    }
}
