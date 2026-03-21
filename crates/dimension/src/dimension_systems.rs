use bevy::ecs::entity::EntityHashSet;
#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
use common::common_components::*;
use tilemap::tile::{tile_components::{PortalRecipe, TileStrId}, tile_resources::PortalSeri};
use tilemap::terrain::terrprobe::terrprobe_resources::TerrProbeTemplEntityMap;
use ::tilemap_shared::*;


#[allow(unused_parens)]
pub fn replace_dim_string_ref_by_entity_ref(
    mut cmd: Commands,
    dimension_entity_map: Res<DimensionEntityMap>,
    dimension_query: Query<&DimensionRootOplist>,
    dimension_strid_query: Query<(Entity, Option<&StrId>, &DimensionStrIdRef, Option<&ChildOf>),>,
) {
    for (thing_ent, ent_strid, dimension_strid, child_of) in dimension_strid_query.iter() {

        if let Ok(dimension_entity) = dimension_entity_map.0.get_cloned(&dimension_strid.0) {
            cmd.entity(thing_ent)
                .try_insert(DimensionRef(dimension_entity))
                .try_remove::<DimensionStrIdRef>();

            if let Some(child_of) = child_of {
                if dimension_query.get(child_of.parent()).is_err() {
                    warn!(target: "dimension_loading", "{} {} with added DimensionStrIdRef '{}' shouldn't have ChildOf component, the parent should be the one with the DimensionStrIdRef", ent_strid.cloned().unwrap_or_default(), thing_ent, dimension_strid.0);
                }
            }
            cmd.entity(thing_ent).try_insert(ChildOf(dimension_entity));
        }
        else {
            warn!(target: "dimension_loading", "DimensionStrIdRef '{}' does not have a corresponding Dimension entity in the map.", dimension_strid.0);
        }
    }
}

#[allow(unused_parens)]
pub fn replace_portal_tile_string_ref_by_entity_ref(
    mut cmd: Commands,
    dimension_entity_map: Res<DimensionEntityMap>,
    terrprobe_entity_map: Res<TerrProbeTemplEntityMap>,
    mut portal_tile_query: Query<(Entity, &TileStrId, &PortalSeri, &mut PortalRecipe),(common::AnyDisabling)>,
) {
    for (ent, ent_str_id, portal_seri, mut portal_template) in portal_tile_query.iter_mut() {
        let Ok(dimension_entity) = dimension_entity_map.0.get_cloned(&portal_seri.dest_dimension)
        else {
            error_once!(target: "dimension_loading", "Portal tile '{}' does not have a corresponding Dimension entity in the map.", ent_str_id);
            continue;
        };
        portal_template.dest_dimension = dimension_entity;

        let Ok(terrprobe_ent) = terrprobe_entity_map.0.get_cloned(&portal_seri.oe_terrprobe)
        else {
            error_once!(
                target: "dimension_loading",
                "Portal tile '{}' references unknown terrprobe '{}'.",
                ent_str_id,
                portal_seri.oe_terrprobe
            );
            continue;
        };
        portal_template.terrprobe_ent = terrprobe_ent;

        cmd.entity(ent).try_remove::<PortalSeri>();
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
            let Ok(dim_ent) = dimension_entity_map.0.get_cloned(str_ref)
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

#[allow(unused_parens)]
pub fn ensure_childof_for_enti_with_dimension_ref_and_readjust_if_parent_was_dimension(
    mut cmd: Commands,
    dimension_query: Query<(Entity),(With<Dimension>)>,
    query: Query<
        (Entity, &DimensionRef, Option<&ChildOf>),
        (
            Or<(Without<ChildOf>, Changed<DimensionRef>)>,
        ),
    >,
) {
    for (ent, dimension_ref, child_of) in query.iter() {
        let Some(child_of) = child_of else {
            cmd.entity(ent).try_insert(ChildOf(dimension_ref.0));
            continue;
        };
        if dimension_query.get(child_of.parent()).is_ok() {
            cmd.entity(ent).try_insert(ChildOf(dimension_ref.0));
        }
    }
}
