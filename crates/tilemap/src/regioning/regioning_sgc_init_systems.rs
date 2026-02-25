use std::cmp::Ordering;

use bevy::{platform::collections::HashSet, prelude::*};
use common::{common_components::*, common_tag_components::TagSet, log_targets::SGC_INIT};
use game_common::{game_common_components::ArgsDict, game_common_samplers::EntityWeightedSampler};
use ::tilemap_shared::*;

use crate::{
    regioning::{
        regioning_resources::*,
        regioning_sgc_components::*,
        StructuredGenConfigEntityMap,
    },
    terrain::terrprobe::opfilter::opfilter_resources::OpFilterEntityMap,
};

struct PrioritySgcDef {
    ent: Entity,
    priority: f32,
    tags: HashSet<String>,
    run_before_tags: HashSet<String>,
    run_after_tags: HashSet<String>,
}

fn build_prioritized_entities(defs: &[PrioritySgcDef]) -> Vec<Entity> {
    if defs.is_empty() {
        return Vec::new();
    }

    let mut invalid_defs: HashSet<usize> = HashSet::new();
    let mut edges: Vec<HashSet<usize>> = vec![HashSet::default(); defs.len()];
    let mut indegree = vec![0_usize; defs.len()];

    for i in 0..defs.len() {
        for tag in &defs[i].run_before_tags {
            let mut found = false;
            for j in 0..defs.len() {
                if i == j || !defs[j].tags.contains(tag) {
                    continue;
                }
                found = true;
                if edges[i].insert(j) {
                    indegree[j] += 1;
                }
            }
            if !found {
                error!(
                    target: SGC_INIT,
                    "Skipping prioritized SGC {:?}: run_before_sgcs_with_tags contains unmatched tag '{}'",
                    defs[i].ent,
                    tag
                );
                invalid_defs.insert(i);
            }
        }
        for tag in &defs[i].run_after_tags {
            let mut found = false;
            for j in 0..defs.len() {
                if i == j || !defs[j].tags.contains(tag) {
                    continue;
                }
                found = true;
                if edges[j].insert(i) {
                    indegree[i] += 1;
                }
            }
            if !found {
                error!(
                    target: SGC_INIT,
                    "Skipping prioritized SGC {:?}: run_after_sgcs_with_tags contains unmatched tag '{}'",
                    defs[i].ent,
                    tag
                );
                invalid_defs.insert(i);
            }
        }
    }

    for &invalid_i in &invalid_defs {
        for &next in &edges[invalid_i] {
            indegree[next] = indegree[next].saturating_sub(1);
        }
        edges[invalid_i].clear();
    }

    let mut remaining: Vec<usize> = (0..defs.len())
        .filter(|i| !invalid_defs.contains(i))
        .collect();
    let mut ordered = Vec::with_capacity(defs.len());
    while !remaining.is_empty() {
        let Some((selected_pos, selected_i)) = remaining
            .iter()
            .enumerate()
            .filter(|(_, i)| indegree[**i] == 0)
            .max_by(|(_, a), (_, b)| {
                defs[**a]
                    .priority
                    .total_cmp(&defs[**b].priority)
                    .then(Ordering::Equal)
            })
            .map(|(pos, i)| (pos, *i))
        else {
            for i in &remaining {
                error!(
                    target: SGC_INIT,
                    "Skipping prioritized SGC {:?}: impossible ordering (cyclic run_before_sgcs_with_tags/run_after_sgcs_with_tags)",
                    defs[*i].ent
                );
            }
            break;
        };

        remaining.swap_remove(selected_pos);
        ordered.push(defs[selected_i].ent);
        for &next in &edges[selected_i] {
            indegree[next] -= 1;
        }
    }

    ordered
}

#[allow(unused_parens)]
pub fn init_structured_gen_configs(
    mut cmd: Commands,
    map: Res<StructuredGenConfigEntityMap>,
    dimension_entity_map: Res<DimensionEntityMap>,
    egui_holder_query: Query<Entity, With<EguiSgcsHolder>>,
    _opfilter_entity_map: Res<OpFilterEntityMap>,
) {
    if !map.0.is_empty() {
        return;
    }

    let mut ent_w_sampler = EntityWeightedSampler::default();
    let mut sgcs_comps = Vec::new();
    let mut exclusive_for_dims = Vec::new();
    let mut priority_defs = Vec::new();

    let egui_ent = if let Ok(egui_ent) = egui_holder_query.single() {
        egui_ent
    } else {
        cmd.spawn(EguiSgcsHolder).id()
    };

    for structured_gen_seri in load_sgc_seri_defs() {
        let seri_id = structured_gen_seri.id.clone();
        let main_ent = cmd.spawn_empty().id();

        let mut gen_cfg = StructuredGenConfig::new(structured_gen_seri.structure_id);
        let sgc_id = StrId::trunc(seri_id.clone());

        gen_cfg.max_per_region = structured_gen_seri.max_per_region;
        if !structured_gen_seri.args.is_empty() {
            gen_cfg.args = ArgsDict::with_capacity(structured_gen_seri.args.len());
            for (key, val_vec) in structured_gen_seri.args.clone() {
                gen_cfg.args.insert(key, val_vec);
            }
        }

        let mut tags_vec = structured_gen_seri.tags.clone();
        if tags_vec.iter().all(|tag| tag != &seri_id) {
            tags_vec.push(seri_id.clone());
        }
        let tags_set = HashSet::from_iter(tags_vec.iter().cloned());
        cmd.entity(main_ent).try_insert(TagSet::new(tags_vec));

        if !structured_gen_seri.whitelisted_filters.is_empty() {
            for _opfilter_id in structured_gen_seri.whitelisted_filters {}
        }
        if !structured_gen_seri.exclusive_for_dimensions.is_empty() {
            let mut dim_refs = MultipleDimensionRefs::default();
            for dim_strid in &structured_gen_seri.exclusive_for_dimensions {
                let Ok(dim_ent) = dimension_entity_map.0.get_cloned(dim_strid) else {
                    error!(target: SGC_INIT, "Failed to find Dimension with StrId: {}", dim_strid);
                    continue;
                };
                dim_refs.0.insert(dim_ent);
            }
            if dim_refs.0.is_empty() {
                error!(target: SGC_INIT, "StructureSeri with id: {} has no valid dimension references.", seri_id);
                continue;
            }
            exclusive_for_dims.push((main_ent, dim_refs));
        }
        if !structured_gen_seri.pdisk_mindist_and_tag.is_empty() {
            let Ok(poisson_disk) =
                PoissonDisk::multiple_tagged(structured_gen_seri.pdisk_mindist_and_tag.clone(), 3, 16)
            else {
                error!(target: SGC_INIT, "Failed to create PoissonDisk for StructureSeri with id: {}, skipping PoissonDisk creation.", seri_id);
                continue;
            };
            cmd.entity(main_ent).insert(poisson_disk);
        }

        if structured_gen_seri.priority > 0.0 {
            priority_defs.push(PrioritySgcDef {
                ent: main_ent,
                priority: structured_gen_seri.priority,
                tags: tags_set,
                run_before_tags: structured_gen_seri.run_before_sgcs_with_tags,
                run_after_tags: structured_gen_seri.run_after_sgcs_with_tags,
            });
        }

        ent_w_sampler.insert(main_ent, structured_gen_seri.weight);
        sgcs_comps.push((main_ent, (sgc_id, gen_cfg)));
    }

    let prioritized_ents = build_prioritized_entities(&priority_defs);

    cmd.insert_batch(exclusive_for_dims);
    cmd.insert_batch(sgcs_comps);
    cmd.insert_resource(Prioritized(prioritized_ents));
    cmd.spawn((SgcsWeightedSampler, ent_w_sampler, ChildOf(egui_ent)));
}
