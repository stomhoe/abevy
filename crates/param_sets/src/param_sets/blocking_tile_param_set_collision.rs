use super::*;
use std::borrow::Cow;

pub(super) fn resolve_collision_context_for_entity<'a, 'w, 's>(
    this: &'a BlockingTileParamSet<'w, 's>,
    entity: Entity,
) -> (Cow<'a, InteractionZone>, CardinalDirection) {
    let collision_zone = this
        .interaction_zones
        .get(entity)
        .ok()
        .and_then(|zones| zones.get_collision_mask())
        .map(Cow::Borrowed)
        .or_else(|| {
            this.bit_ref_query
                .get(entity)
                .ok()
                .and_then(|bit_ref| this.interaction_zones.get(bit_ref.0).ok())
                .and_then(|zones| zones.get_collision_mask())
                .map(Cow::Borrowed)
        })
        .or_else(|| {
            this.race_ref_query
                .get(entity)
                .ok()
                .and_then(|race_ref| this.interaction_zones.get(race_ref.0).ok())
                .and_then(|zones| zones.get_collision_mask())
                .map(Cow::Borrowed)
        })
        .unwrap_or_else(|| Cow::Owned(InteractionZone::collision_default_zone()));
    let facing_dir = this
        .tile_gathering_params
        .cardinal_direction_query
        .get(entity)
        .cloned()
        .unwrap_or_default();
    (collision_zone, facing_dir)
}
