#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
use common::common_components::*;

use crate::terrain::{
    opfilter::opfilter_resources::OpFilterEntityMap,
    terrprobe::{
        terrprobe_components::{ProbePatternSeri, TerrProbeTempl},
        terrprobe_resources::{EguiTptsHolder, TerrProbeTemplEntityMap, load_terrain_probe_seri_defs},
    },
};
#[allow(unused_parens)]
pub fn init_terrain_probes(
    mut cmd: Commands,
    map: Res<TerrProbeTemplEntityMap>,
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
    for seri in load_terrain_probe_seri_defs() {

        let Ok(str_id) = StrId::new_with_result(seri.id.clone(), 1) else {
            error!(target: "terrprobe_init", "Failed to create StrId for terrain probe id '{}'", seri.id);
            continue;
        };
        let Ok(opfilter_ent) = opfilter_entity_map.0.get_cloned(&seri.opfilter_id) else {
            error!(target: "terrprobe_init", "Failed to resolve opfilter '{}' for terrain probe '{}'", seri.opfilter_id, seri.id);
            continue;
        };

        let Some(parsed_probe_pattern) = ProbePatternSeri::parse(&seri.probe_pattern) else {
            error!(
                target: "terrprobe_init",
                "Invalid probe_pattern '{}' for terrain probe '{}'. Expected 'sun' or 'spiral'",
                seri.probe_pattern,
                seri.id
            );
            continue;
        };

        let ent = cmd.spawn_empty().id();
        comps.push((ent, (
            str_id,
            Replicated,
            AssetScoped,
            HotReload,
            TerrProbeTempl::from_seri(
                opfilter_ent,
                parsed_probe_pattern,
                seri.step_size,
                seri.max_batches,
                seri.iterations_per_batch,
                seri.max_emitted_results,
                seri.min_result_distance,
            ),
            ChildOf(egui_ent),
        )));
    }

    cmd.try_insert_batch(comps);
}
