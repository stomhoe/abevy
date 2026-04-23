use bevy::prelude::*;
use common::{
    common_components::{HashId, HashIdMap, StrId},
    common_tag_components::{HashedTagsVec, TagSet},
};
use std::sync::Arc;

use crate::{
    terrain::{
        operation_list::operation_list_components::*,
        terrgen_components::*,
        terrprobe::{opfilter::opfilter_components::OpFilter},
        terrgen_resources::{TerrGenSharedTaskData, TerrGenSharedTaskDataInner},
    },
};
use ::tilemap_shared::*;

#[allow(unused_parens, )]
pub(crate) fn init_terrgen_shared_task_data(
    mut commands: Commands,
    shared_task_data: Option<ResMut<TerrGenSharedTaskData>>,
    oplist_query: Query<
        (
            &HashId,
            &StrId,
            &OperationList,
            &OplistSize,
            Option<&HashedTagsVec>,
            Option<&TagSet>,
        ),
        (),
    >,
    fnl_noises: Query<(&HashId, &FnlNoiseComp), ()>,
    op_filters: Query<(&HashId, &OpFilter), ()>,
) {
    let oplists_count = oplist_query.iter().count();
    let noises_count = fnl_noises.iter().count();
    let filters_count = op_filters.iter().count();
    let any_oplist_has_tags = oplist_query
        .iter()
        .any(|(_, _, _, _, oplist_tags_opt, oplist_tagset_opt, )| {
            oplist_tags_opt.is_some() || oplist_tagset_opt.is_some()
        });
    if let Some(shared_task_data) = shared_task_data.as_ref() {
        let shared = &shared_task_data.0;
        if shared.oplists.len() == oplists_count
            && shared.noises.len() == noises_count
            && shared.filters.len() == filters_count
            && (!any_oplist_has_tags || !shared.oplist_tags.is_empty())
        {
            return;
        }
    }
    if oplists_count == 0 || noises_count == 0 {
        return;
    }

    let mut oplists: HashIdMap<OperationList> = HashIdMap::default();
    let mut oplist_ids: HashIdMap<StrId> = HashIdMap::default();
    let mut oplist_debug_var_ids: HashIdMap<Vec<HashId>> = HashIdMap::default();
    let mut oplist_sizes: HashIdMap<OplistSize> = HashIdMap::default();
    let mut oplist_tags: HashIdMap<HashedTagsVec> = HashIdMap::default();
    for (&oplist_hash, oplist_id, oplist, &oplist_size, oplist_tags_opt, oplist_tagset_opt) in
        oplist_query.iter()
    {
        let _ = oplists.overwrite(oplist_hash, oplist.clone());
        let _ = oplist_ids.overwrite(oplist_hash, oplist_id.clone());
        let _ = oplist_debug_var_ids.overwrite(
            oplist_hash,
            oplist.hash_ids_mapped_to_strids.keys().copied().collect::<Vec<_>>(),
        );
        let _ = oplist_sizes.overwrite(oplist_hash, oplist_size);
        let tags_to_cache = oplist_tags_opt
            .cloned()
            .or_else(|| oplist_tagset_opt.map(HashedTagsVec::from));
        if let Some(tags) = tags_to_cache {
            let _ = oplist_tags.overwrite(oplist_hash, tags);
        };
    }

    let mut noises: HashIdMap<FnlNoiseComp> = HashIdMap::default();
    for (noise_hash, noise) in fnl_noises.iter() {
        let _ = noises.overwrite(*noise_hash, noise.clone());
    }

    let mut filters: HashIdMap<OpFilter> = HashIdMap::default();
    for (&filter_hash, filter) in op_filters.iter() {
        let _ = filters.overwrite(filter_hash, filter.clone());
    }

    let new_shared_data = Arc::new(TerrGenSharedTaskDataInner {
        oplists,
        oplist_ids,
        oplist_debug_var_ids,
        oplist_sizes,
        oplist_tags,
        noises,
        filters,
    });

    if let Some(mut shared_task_data) = shared_task_data {
        shared_task_data.0 = Arc::clone(&new_shared_data);
    } else {
        commands.insert_resource(TerrGenSharedTaskData(new_shared_data));
    }
}
