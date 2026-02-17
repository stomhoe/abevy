#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
use common::{common_components::*, common_tag_components::HashedTagsVec};

use std::{f32::{INFINITY, NEG_INFINITY}, mem::take};

use crate::terrain_gen::opfilter::{
    opfilter_components::OpFilter,
    opfilter_resources::{EguiOpFiltersHolder, OpFilterEntityMap, OpFilterSeri, OpFilterSerisHandles},
};

#[allow(unused_parens)]
pub fn init_opfilters(
    mut cmd: Commands,
    map: Res<OpFilterEntityMap>,
    mut seris_handles: ResMut<OpFilterSerisHandles>,
    mut assets: ResMut<Assets<OpFilterSeri>>,
    egui_holder_query: Query<Entity, With<EguiOpFiltersHolder>>,
) {
    if !map.0.is_empty() { return; }

    let egui_ent = if let Ok(egui_ent) = egui_holder_query.single() {
        egui_ent
    } else {
        cmd.spawn(EguiOpFiltersHolder).id()
    };

    let mut comps = Vec::new();
    for handle in take(&mut seris_handles.handles).into_iter() {
        let Some(seri) = assets.remove(&handle) else { continue; };

        let Ok(str_id) = StrId::new_with_result(seri.id.clone(), 1) else {
            error!(target: "opfilter_init", "Failed to create StrId for opfilter id '{}'", seri.id);
            continue;
        };

        let ent = cmd.spawn_empty().id();
        comps.push((ent, (
            str_id,
            Replicated,
            OpFilter {
                tags: HashedTagsVec::new(seri.tags.iter()),
                op_i: seri.op_i,
                min_val: seri.min_val.unwrap_or(NEG_INFINITY),
                max_val: seri.max_val.unwrap_or(INFINITY),
            },
            ChildOf(egui_ent),
        )));
    }

    cmd.try_insert_batch(comps);
}
