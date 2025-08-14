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



// ----------------------> NO OLVIDARSE DE AGREGARLO AL Plugin DEL MÓDULO <-----------------------------
//                                                       ^^^^
#[allow(unused_parens)]
pub fn replace_string_ref_by_entity_ref(
    mut cmd: Commands, 
    dimension_entity_map: Res<DimensionEntityMap>,
    mut dimension_strid_query: Query<(Entity, &DimensionStrIdRef,),(Changed<DimensionStrIdRef>,)>,
) {
    for (ent, dimension_strid) in dimension_strid_query.iter_mut() {
        cmd.entity(ent).remove::<DimensionStrIdRef>();
        if let Ok(entity) = dimension_entity_map.0.get(&dimension_strid.0) {
            cmd.entity(ent).insert(DimensionRef(entity));
        }
        else {
            warn!(target: "dimension_loading", "DimensionStrIdRef '{}' does not have a corresponding DimensionRef in the map.", dimension_strid.0);
        }
    }
}
