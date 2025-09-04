use bevy::ecs::{entity::EntityHashSet, entity_disabling::Disabled};
#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
use common::common_components::{DisplayName, EntityPrefix, StrId};
use ::dimension_shared::*;
use tilemap::{terrain_gen::{terrgen_oplist_components::OperationList, terrgen_resources::OpListEntityMap}, tile::{tile_components::PortalTemplate, tile_resources::PortalSeri}};
use crate::{
    dimension_resources::*,
/*
    dimension_events::*,
*/
};



#[allow(unused_parens)]
pub fn dim_replace_string_ref_by_entity_ref(
    mut cmd: Commands, 
    dimension_entity_map: Res<DimensionEntityMap>,
    dimension_query: Query<Option<&DimensionRootOplist>>,
    dimension_strid_query: Query<(Entity, Option<&StrId>, &DimensionStrIdRef, Option<&ChildOf>),>,
    mut portal_tile_query: Query<(Entity, &StrId, &PortalSeri, &mut PortalTemplate),(With<Disabled>)>,
    oplist_map: Res<OpListEntityMap>,
) {
    for (thing_ent, ent_strid, dimension_strid, child_of) in dimension_strid_query.iter() {

        if let Ok(dimension_entity) = dimension_entity_map.0.get(&dimension_strid.0) {
            cmd.entity(thing_ent)
                .insert(DimensionRef(dimension_entity))
                .remove::<DimensionStrIdRef>();

            if let Some(child_of) = child_of {
                if dimension_query.get(child_of.parent()).is_err() {
                    warn!(target: "dimension_loading", "{} {} with added DimensionStrIdRef '{}' shouldn't have ChildOf component, the parent should be the one with the DimensionStrIdRef", ent_strid.cloned().unwrap_or_default(), thing_ent, dimension_strid.0);
                }
            }
            cmd.entity(thing_ent).insert(ChildOf(dimension_entity));
        }
        else {
            warn!(target: "dimension_loading", "DimensionStrIdRef '{}' does not have a corresponding Dimension entity in the map.", dimension_strid.0);
        }
    }
    for (ent, ent_str_id, portal_seri, mut portal_template) in portal_tile_query.iter_mut() {
        let Ok(dimension_entity) = dimension_entity_map.0.get(&portal_seri.dest_dimension)
        else {
            error!(target: "dimension_loading", "Portal tile '{}' does not have a corresponding Dimension entity in the map.", ent_str_id);
            continue;
        };
        let Ok(root_oplist) = dimension_query.get(dimension_entity) else {
            error!(target: "dimension_loading", "PortalTemplate {} references a Dimension that doesn't exist: {:?}", ent_str_id, portal_template.root_oplist);
            continue;
        };
        let Some(root_oplist) = root_oplist else {
            continue;
        };

        portal_template.root_oplist = root_oplist.0;

        let Ok(oplist_ent) = oplist_map.0.get(&portal_seri.oplist) else {
            error!(target: "dimension_loading", "Portal tile '{}' does not have a corresponding OperationList entity in the map.", ent_str_id);
            continue;
        };

        portal_template.checked_oplist = oplist_ent;
        cmd.entity(ent).remove::<PortalSeri>();
    }

}

#[allow(unused_parens, )]
pub fn replace_multiple_string_refs_by_entity_refs(
    mut cmd: Commands,
    query: Query<(Entity, Option<&StrId>, &MultipleDimensionStringRefs, ), Changed<MultipleDimensionStringRefs>>,
    dimension_entity_map: Res<DimensionEntityMap>,
) {
    for (ent, ent_str_id, string_refs, ) in query.iter() {
        let mut entity_set = EntityHashSet::default();
        for str_ref in string_refs.iter() {
            let Ok(dim_ent) = dimension_entity_map.0.get(str_ref) 
            else {
                error!(target: "dimension_loading", "{}'s MultipleDimensionStringRefs '{}' does not have a corresponding Entity in DimensionEntityMap.", ent_str_id.cloned().unwrap_or_default(), str_ref);
                continue;
            };

            entity_set.insert(dim_ent);
        }
        cmd.entity(ent)
            .remove::<MultipleDimensionStringRefs>()
            .insert(MultipleDimensionRefs(entity_set));
    }
}


