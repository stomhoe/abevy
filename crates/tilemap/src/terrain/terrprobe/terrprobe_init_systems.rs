#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
use common::common_components::*;
use common::common_tag_components::HashedTagsVec;
use common::log_targets::TERRPROBE_INIT;
use game_common::game_common_components::{Templ, TemplEntiRef};

use crate::terrain::{
    terrprobe::opfilter::opfilter_resources::{OpFilterEntityMap, OpFilterRef},
    terrprobe::opfilter::opfilter_components::OpFilter,
    terrprobe::{
        terrprobe_components::{ProbePatternSeri, TerrProbeTempl},
        terrprobe_resources::{EguiTptsHolder, TerrProbeTemplEntityMap, load_terrain_probe_seri_defs},
    },
};
use crate::{regioning::regioning_resources::StructuredGenConfigEntityMap, tile::tile_resources::TemplTileEntsWithinTag};
#[allow(unused_parens, )]
pub fn init_terrain_probes(
    mut cmd: Commands,
    map: Res<TerrProbeTemplEntityMap>,
    sgc_entity_map: Res<StructuredGenConfigEntityMap>,
    opfilter_entity_map: Res<OpFilterEntityMap>,
    opfilter_query: Query<(&OpFilter, &StrId, ), (),>,
    tile_ents_with_tag: Res<TemplTileEntsWithinTag>,
    entity_zeroes: Query<(&Templ, ), (),>,
    egui_holder_query: Query<(Entity, ), (With<EguiTptsHolder>, ),>,
) {
    if !map.0.is_empty() { return; }

    let egui_ent = if let Ok((egui_ent,)) = egui_holder_query.single() {
        egui_ent
    } else {
        cmd.spawn(EguiTptsHolder).id()
    };

    let mut comps = Vec::new();
    for def in load_terrain_probe_seri_defs() {
        if def.is_abstract {
            continue;
        }
        let seri = def.seri;
        let str_id = match StrId::new_with_result(seri.id.clone(), 1) {
            Ok(str_id) => str_id,
            Err(err) => {
                error!(
                    target: TERRPROBE_INIT,
                    "Failed to create StrId for terrain probe id '{}': {}",
                    seri.id,
                    err,
                );
                continue;
            }
        };
        let opfilter_id = seri.opfilter_id.trim();
        let opfilter_var_name = seri.opfilter_var_name.trim();
        let opfilter_ent = if !opfilter_id.is_empty() {
            let Ok(opfilter_ent) = opfilter_entity_map.0.get_cloned(opfilter_id) else {
                error!(target: TERRPROBE_INIT, "Failed to resolve opfilter '{}' for terrain probe '{}'", seri.opfilter_id, seri.id);
                continue;
            };
            Some(opfilter_ent)
        } else {
            None
        };

        let has_inline_overrides = !seri.opfilter_tags.is_empty()
            || !opfilter_var_name.is_empty()
            || seri.opfilter_min_val != f32::NEG_INFINITY
            || seri.opfilter_max_val != f32::INFINITY;
        if opfilter_ent.is_none() && !has_inline_overrides {
            error!(target: TERRPROBE_INIT, "Terrain probe '{}' requires either opfilter_id or inline opfilter_* fields", seri.id);
            continue;
        }
        let opfilter_ref = match (opfilter_ent, has_inline_overrides) {
            (Some(opfilter_ent), false) => {
                let Ok((_, opfilter_str_id, )) = opfilter_query.get(opfilter_ent) else {
                    error!(target: TERRPROBE_INIT, "OpFilter entity {:?} missing OpFilter component for terrain probe '{}'", opfilter_ent, seri.id);
                    continue;
                };
                OpFilterRef(HashId::from(opfilter_str_id.as_str()))
            }
            (opfilter_ent, _) => {
                let mut opfilter = if let Some(opfilter_ent) = opfilter_ent {
                    let Ok((opfilter, _, )) = opfilter_query.get(opfilter_ent) else {
                        error!(target: TERRPROBE_INIT, "OpFilter entity {:?} missing OpFilter component for terrain probe '{}'", opfilter_ent, seri.id);
                        continue;
                    };
                    opfilter.clone()
                } else {
                    OpFilter {
                        tags: HashedTagsVec::new(seri.opfilter_tags.iter()),
                        var_name_hash: (!opfilter_var_name.is_empty()).then_some(HashId::hash(opfilter_var_name)),
                        min_val: seri.opfilter_min_val,
                        max_val: seri.opfilter_max_val,
                    }
                };
                if !seri.opfilter_tags.is_empty() {
                    opfilter.tags = HashedTagsVec::new(seri.opfilter_tags.iter());
                }
                if !opfilter_var_name.is_empty() {
                    opfilter.var_name_hash = Some(HashId::hash(opfilter_var_name));
                }
                if seri.opfilter_min_val != f32::NEG_INFINITY {
                    opfilter.min_val = seri.opfilter_min_val;
                }
                if seri.opfilter_max_val != f32::INFINITY {
                    opfilter.max_val = seri.opfilter_max_val;
                }
                let inline_opfilter_id = format!("{}_inline_opfilter", seri.id);
                let inline_opfilter_strid = match StrId::new_with_result(inline_opfilter_id.as_str(), 0) {
                    Ok(inline_opfilter_strid) => inline_opfilter_strid,
                    Err(err) => {
                        error!(
                            target: TERRPROBE_INIT,
                            "Terrain probe '{}' could not build inline opfilter StrId '{}': {}",
                            seri.id,
                            inline_opfilter_id,
                            err,
                        );
                        continue;
                    }
                };
                let _ = cmd.spawn((
                    inline_opfilter_strid.clone(),
                    AddHashIdFromStrId,
                    Replicated,
                    AssetScoped,
                    SelectedForHotReload,
                    Templ,
                    opfilter,
                ));
                OpFilterRef(HashId::from(inline_opfilter_strid.as_str()))
            }
        };

        let mut structuregen_whitelist = Vec::with_capacity(seri.structuregen_whitelist.len());
        for sgc_id in &seri.structuregen_whitelist {
            let Ok(sgc_ent) = sgc_entity_map.0.get_cloned(sgc_id) else {
                error!(target: TERRPROBE_INIT, "Failed to resolve SGC '{}' for terrain probe '{}'", sgc_id, seri.id);
                continue;
            };
            structuregen_whitelist.push(sgc_ent);
        }
        let mut structuregen_blacklist = Vec::with_capacity(seri.structuregen_blacklist.len());
        for sgc_id in &seri.structuregen_blacklist {
            let Ok(sgc_ent) = sgc_entity_map.0.get_cloned(sgc_id) else {
                error!(target: TERRPROBE_INIT, "Failed to resolve SGC '{}' for terrain probe '{}'", sgc_id, seri.id);
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
                let Ok((_,)) = entity_zeroes.get(tile_ent) else {
                    continue;
                };
                let templ_ref = TemplEntiRef(tile_ent);
                if sgc_admitted_tiles_as_found_pos.iter().all(|existing| *existing != templ_ref) {
                    sgc_admitted_tiles_as_found_pos.push(templ_ref);
                }
            }
        }

        let Some(parsed_probe_pattern) = ProbePatternSeri::parse(&seri.probe_pattern) else {
                error!(
                    target: TERRPROBE_INIT,
                    "Invalid probe_pattern '{}' for terrain probe '{}'. Expected 'concentric' (alias: 'conc'), 'chunk' or 'region'",
                    seri.probe_pattern,
                    seri.id
            );
            continue;
        };

        let ent = cmd.spawn_empty().id();
        let templ = TerrProbeTempl::from_seri(
            opfilter_ref,
            structuregen_whitelist,
            structuregen_blacklist,
            seri.required_tile_tags.clone(),
            sgc_admitted_tiles_as_found_pos,
            parsed_probe_pattern,
            seri.concentric_sample_spacing,
            seri.step_size,
            seri.region_multiplier,
            seri.max_batches,
            seri.iterations_per_batch,
            seri.max_emitted_results,
            seri.min_result_distance,
            seri.collect,
        );
        comps.push((ent, (
            str_id,
            Replicated,
            AssetScoped,
            SelectedForHotReload,
            templ,
            ChildOf(egui_ent),
        )));
    }

    cmd.try_insert_batch(comps);
}
