#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
use common::{common_components::{DisplayName, EntityPrefix, HashId, StrId}, common_tag_components::TagSet};
use crate::{
    dimension_resources::*,
/*
    dimension_events::*,
*/
};
use ::dimension_shared::*;

#[allow(unused_parens)]
pub fn init_dimensions(
    mut cmd: Commands, map: Option<Res<DimensionEntityMap>>,
    mut seris_handles: ResMut<DimensionSerisHandles>,
    mut assets: ResMut<Assets<DimensionSeri>>,
) {
    if map.is_some(){ return; }
    cmd.init_resource::<DimensionEntityMap>();

    let mut common_components = Vec::new();
    let mut tagsets_to_insert = Vec::new();
    let mut whitelisted_structure_gen_tags_to_insert = Vec::new();
    let mut blacklisted_structure_gen_tags_to_insert = Vec::new();

    for handle in std::mem::take(&mut seris_handles.handles) {
        let Some(seri) = assets.remove(&handle) else { continue };

        let str_id = match StrId::new_with_result(seri.id.clone(), 2) {
            Ok(id) => id,
            Err(e) => {
                let err = BevyError::from(format!("Failed to create StrId for dimension {}: {}", seri.id, e));
                error!(target: "dimension_loading", "{}", err);
                continue;
            }
        };

        info!(target: "dimension_loading", "Spawning dimension '{}' with id '{}' ", seri.name, str_id);

        let dim_ent = cmd.spawn_empty().id();

        if let Some(tags) = seri.tags {
            tagsets_to_insert.push((dim_ent, TagSet::new(tags)));
        }
        if let Some(whitelisted_structure_gen_tags) = seri.whitelisted_structure_gen_tags {
            let tag_set = WhitelistedStructureGenTags(TagSet::new(whitelisted_structure_gen_tags));
            whitelisted_structure_gen_tags_to_insert.push((dim_ent, tag_set));
        }
        if let Some(blacklisted_structure_gen_tags) = seri.blacklisted_structure_gen_tags {
            let tag_set = BlacklistedStructureGenTags(TagSet::new(blacklisted_structure_gen_tags));
            blacklisted_structure_gen_tags_to_insert.push((dim_ent, tag_set));
        }

        common_components.push((dim_ent, (
            HashId::from(str_id.as_ref()),
            str_id,
            Transform::default(),
            DisplayName::new(seri.name),
            Dimension,
            Visibility::Visible,
        )))
    }
    cmd.insert_batch(common_components);
    cmd.insert_batch(tagsets_to_insert);
    cmd.insert_batch(whitelisted_structure_gen_tags_to_insert);
    cmd.insert_batch(blacklisted_structure_gen_tags_to_insert);
}


pub fn add_dimensions_to_map(
    map: Option<ResMut<DimensionEntityMap>>,
    query: Query<(Entity, &EntityPrefix, &StrId), (With<Dimension>, Added<StrId>)>,
) {
    if let Some(mut map) = map {
        for (ent, prefix, str_id) in query.iter() {
            if let Err(err) = map.0.try_insert(str_id, ent, ) {
                error!(target: "dimension_loading", "{} {} already in DimensionEntityMap : {}", prefix, str_id, err);
            } else {
                info!(target: "dimension_loading", "Inserted Dimension'{}' {:?} into DimensionEntityMap  ", str_id, ent);
            }
        }
    } else {
        warn!(target: "dimension_loading", "DimensionEntityMap resource not found, cannot add dimensions to map.");
    }
}