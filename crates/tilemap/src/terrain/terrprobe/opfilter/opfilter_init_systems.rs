#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
use common::{common_components::*, common_tag_components::HashedTagsVec};

use crate::terrain::terrprobe::opfilter::{
    opfilter_components::OpFilter,
    opfilter_resources::{EguiOpFiltersHolder, OpFilterEntityMap, load_op_filter_seri_defs},
};

#[allow(unused_parens)]
pub fn init_opfilters(
    mut cmd: Commands,
    map: Res<OpFilterEntityMap>,
    egui_holder_query: Query<Entity, With<EguiOpFiltersHolder>>,
) {
    if !map.0.is_empty() { return; }

    let egui_ent = if let Ok(egui_ent) = egui_holder_query.single() {
        egui_ent
    } else {
        cmd.spawn(EguiOpFiltersHolder).id()
    };

    let mut comps = Vec::new();
    for seri in load_op_filter_seri_defs() {

        let Ok(str_id) = StrId::new_with_result(seri.id.clone(), 1) else {
            error!(target: "opfilter_init", "Failed to create StrId for opfilter id '{}'", seri.id);
            continue;
        };

        let ent = cmd.spawn_empty().id();
        comps.push((ent, (
            str_id,
            Replicated,
            AssetScoped,
            HotReload,
            OpFilter {
                tags: HashedTagsVec::new(seri.tags.iter()),
                op_i: if seri.op_i == u16::MAX { None } else { Some(seri.op_i) },
                min_val: seri.min_val,
                max_val: seri.max_val,
            },
            ChildOf(egui_ent),
        )));
    }

    cmd.try_insert_batch(comps);
}
