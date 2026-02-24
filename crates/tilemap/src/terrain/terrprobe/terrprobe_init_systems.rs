#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
use common::common_components::*;
use common::common_tag_components::HashedTagsVec;
use game_common::game_common_components::{EntityZero, EntityZeroRef};

use crate::terrain::{
    terrprobe::opfilter::opfilter_resources::OpFilterEntityMap,
    terrprobe::opfilter::opfilter_components::OpFilter,
    terrprobe::{
        terrprobe_components::{ProbePatternSeri, TerrProbeTempl},
        terrprobe_resources::{EguiTptsHolder, TerrProbeTemplEntityMap, load_terrain_probe_seri_defs},
    },
};
use crate::{regioning::regioning_resources::StructuredGenConfigEntityMap, tile::tile_resources::EzeroTileEntsWithinTag};
#[allow(unused_parens)]
pub fn init_terrain_probes(
    mut cmd: Commands,
    map: Res<TerrProbeTemplEntityMap>,
    sgc_entity_map: Res<StructuredGenConfigEntityMap>,
    opfilter_entity_map: Res<OpFilterEntityMap>,
    tile_ents_with_tag: Res<EzeroTileEntsWithinTag>,
    entity_zeroes: Query<&EntityZero>,
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
        let opfilter_ent = if !seri.opfilter_id.trim().is_empty() {
            let Ok(opfilter_ent) = opfilter_entity_map.0.get_cloned(&seri.opfilter_id) else {
                error!(target: "terrprobe_init", "Failed to resolve opfilter '{}' for terrain probe '{}'", seri.opfilter_id, seri.id);
                continue;
            };
            Some(opfilter_ent)
        } else {
            None
        };

        let opfilter_override = (!seri.opfilter_tags.is_empty()
            || seri.opfilter_op_i != u16::MAX
            || seri.opfilter_min_val != f32::NEG_INFINITY
            || seri.opfilter_max_val != f32::INFINITY)
            .then(|| OpFilter {
                tags: HashedTagsVec::new(seri.opfilter_tags.iter()),
                op_i: (seri.opfilter_op_i != u16::MAX).then_some(seri.opfilter_op_i),
                min_val: seri.opfilter_min_val,
                max_val: seri.opfilter_max_val,
            });
        if opfilter_ent.is_none() && opfilter_override.is_none() {
            error!(target: "terrprobe_init", "Terrain probe '{}' requires either opfilter_id or inline opfilter_* fields", seri.id);
            continue;
        }

        let mut structuregen_whitelist = Vec::with_capacity(seri.structuregen_whitelist.len());
        for sgc_id in &seri.structuregen_whitelist {
            let Ok(sgc_ent) = sgc_entity_map.0.get_cloned(sgc_id) else {
                error!(target: "terrprobe_init", "Failed to resolve SGC '{}' for terrain probe '{}'", sgc_id, seri.id);
                continue;
            };
            structuregen_whitelist.push(sgc_ent);
        }
        let mut structuregen_blacklist = Vec::with_capacity(seri.structuregen_blacklist.len());
        for sgc_id in &seri.structuregen_blacklist {
            let Ok(sgc_ent) = sgc_entity_map.0.get_cloned(sgc_id) else {
                error!(target: "terrprobe_init", "Failed to resolve SGC '{}' for terrain probe '{}'", sgc_id, seri.id);
                continue;
            };
            structuregen_blacklist.push(sgc_ent);
        }
        let mut sgc_admitted_tiles_as_found_pos = Vec::new();
        for tag in &seri.required_tile_tags {
            let my_tag = Tag::trunc(tag);
            let Some(tile_ents) = tile_ents_with_tag.0.get(&my_tag) else {
                continue;
            };
            for &tile_ent in tile_ents {
                let Ok(_) = entity_zeroes.get(tile_ent) else {
                    continue;
                };
                let ezero_ref = EntityZeroRef(tile_ent);
                if sgc_admitted_tiles_as_found_pos.iter().all(|existing| *existing != ezero_ref) {
                    sgc_admitted_tiles_as_found_pos.push(ezero_ref);
                }
            }
        }

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
                opfilter_override,
                structuregen_whitelist,
                structuregen_blacklist,
                seri.required_tile_tags.clone(),
                sgc_admitted_tiles_as_found_pos,
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
