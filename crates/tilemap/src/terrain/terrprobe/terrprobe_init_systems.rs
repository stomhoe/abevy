#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
use common::common_components::*;

use std::mem::take;

use crate::terrain::{
    opfilter::opfilter_resources::OpFilterEntityMap,
    terrprobe::{
        terrprobe_components::TerrProbeTempl,
        terrprobe_resources::{EguiTptsHolder, TerrainProbeSeri, TerrainProbeSerisHandles, TerrProbeTemplEntityMap},
    },
};

#[allow(unused_parens)]
pub fn init_terrain_probes(
    mut cmd: Commands,
    map: Res<TerrProbeTemplEntityMap>,
    mut seris_handles: ResMut<TerrainProbeSerisHandles>,
    mut assets: ResMut<Assets<TerrainProbeSeri>>,
    opfilter_entity_map: Res<OpFilterEntityMap>,
    egui_holder_query: Query<Entity, With<EguiTptsHolder>>,
) {
    if !map.0.is_empty() { return; }

    let egui_ent = if let Ok(egui_ent) = egui_holder_query.single() {
        egui_ent
    } else {
        cmd.spawn(EguiTptsHolder).id()
    };

    let mut comps = Vec::new();
    for handle in take(&mut seris_handles.handles).into_iter() {
        let Some(seri) = assets.remove(&handle) else { continue; };

        let Ok(str_id) = StrId::new_with_result(seri.id.clone(), 1) else {
            error!(target: "terrprobe_init", "Failed to create StrId for terrain probe id '{}'", seri.id);
            continue;
        };
        let Ok(opfilter_ent) = opfilter_entity_map.0.get_cloned(&seri.opfilter_id) else {
            error!(target: "terrprobe_init", "Failed to resolve opfilter '{}' for terrain probe '{}'", seri.opfilter_id, seri.id);
            continue;
        };

        let ent = cmd.spawn_empty().id();
        comps.push((ent, (
            str_id,
            Replicated,
            TerrProbeTempl::from_seri(
                opfilter_ent,
                seri.probe_pattern,
                seri.step_size.unwrap_or(1),
                seri.max_batches.unwrap_or(1000),
                seri.iterations_per_batch.unwrap_or(10000),
                seri.max_emitted_results.unwrap_or(1),
                seri.min_result_distance.unwrap_or(0),
            ),
            ChildOf(egui_ent),
        )));
    }

    cmd.try_insert_batch(comps);
}
