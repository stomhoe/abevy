use bevy::{ecs::entity_disabling::Disabled, prelude::*};

use ::being_shared::*;
use ::tilemap_shared::*;

#[allow(unused_parens, )]
pub fn on_pending_natural_spawn_unfreeze_despawn(
    trigger: On<Despawn, (Being, Disabled, NaturalSpawnOrigin)>,
    being_query: Query<((&DimensionRef, &NaturalSpawnOrigin, ), )>,
    mut pending_wildlife_by_chunk: ResMut<BeingsToEnableOnChunkLoad>,
) {
    let Ok(((&dim_ref, &NaturalSpawnOrigin(home_chunk), ), )) = being_query.get(trigger.entity) else {
        return;
    };
    pending_wildlife_by_chunk.remove_being(trigger.entity, dim_ref, home_chunk);
}
