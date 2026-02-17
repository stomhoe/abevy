#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
use common::common_components::*;

use std::mem::take;

use crate::terrain_gen::{
    opfilter::opfilter_resources::OpFilterEntityMap,
    terrain_probe::{
        terrain_probe_components::TerrainProbeTemplate,
        terrain_probe_resources::{EguiTerrainProbeTemplatesHolder, TerrainProbeSeri, TerrainProbeSerisHandles, TerrainProbeTemplateEntityMap},
    },
};

#[allow(unused_parens)]
pub fn init_terrain_probes(
    mut cmd: Commands,
    map: Res<TerrainProbeTemplateEntityMap>,
    mut seris_handles: ResMut<TerrainProbeSerisHandles>,
    mut assets: ResMut<Assets<TerrainProbeSeri>>,
    opfilter_entity_map: Res<OpFilterEntityMap>,
    egui_holder_query: Query<Entity, With<EguiTerrainProbeTemplatesHolder>>,
) {
    if !map.0.is_empty() { return; }

    let egui_ent = if let Ok(egui_ent) = egui_holder_query.single() {
        egui_ent
    } else {
        cmd.spawn(EguiTerrainProbeTemplatesHolder).id()
    };

    let mut comps = Vec::new();
    for handle in take(&mut seris_handles.handles).into_iter() {
        let Some(seri) = assets.remove(&handle) else { continue; };

        let Ok(str_id) = StrId::new_with_result(seri.id.clone(), 1) else {
            error!(target: "terrain_probe_init", "Failed to create StrId for terrain probe id '{}'", seri.id);
            continue;
        };
        let Ok(opfilter_ent) = opfilter_entity_map.0.get_cloned(&seri.opfilter_id) else {
            error!(target: "terrain_probe_init", "Failed to resolve opfilter '{}' for terrain probe '{}'", seri.opfilter_id, seri.id);
            continue;
        };

        let ent = cmd.spawn_empty().id();
        comps.push((ent, (
            str_id,
            Replicated,
            TerrainProbeTemplate {
                opfilter_ent,
                probe_pattern: seri.probe_pattern,
                step_size: seri.step_size.unwrap_or(1),
                max_batches: seri.max_batches.unwrap_or(1000),
                iterations_per_batch: seri.iterations_per_batch.unwrap_or(10000),
                max_emitted_results: seri.max_emitted_results.unwrap_or(1),
            },
            ChildOf(egui_ent),
        )));
    }

    cmd.try_insert_batch(comps);
}
