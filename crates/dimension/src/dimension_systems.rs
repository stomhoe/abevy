use bevy::ecs::entity::EntityHashSet;
#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
use common::common_components::{DisplayName, EntityPrefix, StrId};
use crate::{
    dimension_components::*,
    dimension_resources::*,
/*
    dimension_events::*,
    dimension_layout::*,
*/
};



#[allow(unused_parens)]
pub fn dim_replace_string_ref_by_entity_ref(
    mut cmd: Commands, 
    dimension_entity_map: Res<DimensionEntityMap>,
    mut dimension_strid_query: Query<(Entity, &DimensionStrIdRef, Has<ChildOf>),(Changed<DimensionStrIdRef>,)>,
) {
    for (thing_ent, dimension_strid, child_of) in dimension_strid_query.iter_mut() {

        if let Ok(dimension_entity) = dimension_entity_map.0.get(&dimension_strid.0) {
            cmd.entity(thing_ent)
                .insert(DimensionRef(dimension_entity))
                .remove::<DimensionStrIdRef>();

            if child_of {
                warn!(target: "dimension_loading", "{} with added DimensionStrIdRef '{}' shouldn't have ChildOf component, the parent should be the one with the DimensionStrIdRef", thing_ent, dimension_strid.0);
            }
            cmd.entity(thing_ent).insert(ChildOf(dimension_entity));

        }
        else {
            warn!(target: "dimension_loading", "DimensionStrIdRef '{}' does not have a corresponding DimensionRef in the map.", dimension_strid.0);
        }
    }
}


pub fn replace_multiple_string_refs_by_entity_refs(
    mut cmd: Commands,
    dimension_entity_map: Res<DimensionEntityMap>,
    mut query: Query<(Entity, &MultipleDimensionStringRefs), Changed<MultipleDimensionStringRefs>>,
) {
    for (ent, string_refs) in query.iter_mut() {
        let mut entity_set = EntityHashSet::default();
        for str_id in &string_refs.0 {
            if let Ok(entity) = dimension_entity_map.0.get(str_id) {
                entity_set.insert(entity);
            } else {
                warn!(target: "dimension_loading", "MultipleDimensionStringRefs '{}' does not have a corresponding Entity in DimensionEntityMap.", str_id);
            }
        }
        cmd.entity(ent)
            .remove::<MultipleDimensionStringRefs>()
            .insert(MultipleDimensionRefs(entity_set));
    }
}
