use std::cmp::Ordering;

use bevy::{platform::collections::HashSet, prelude::*};
use common::{common_components::*, common_tag_components::TagSet, log_targets::SGC_INIT};
use tilemap_shared::tilemap_shared_samplers::HashIdWeightedSampler;
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
    hash_id: HashId,
    priority: f32,
    tags: TagSet,
    run_before_tags: HashSet<String>,
    run_after_tags: HashSet<String>,
}

fn build_prioritized_hash_ids(defs: &[PrioritySgcDef]) -> Vec<HashId> {
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
                if i == j || !defs[j].tags.contains(tag.as_str()) {
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
                if i == j || !defs[j].tags.contains(tag.as_str()) {
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
        ordered.push(defs[selected_i].hash_id);
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
    sgc_command_registry: Res<SgcCommandRegistry>,
    dimension_entity_map: Res<DimensionEntityMap>,
    dimension_hash_query: Query<&HashId, With<Dimension>>,
    egui_holder_query: Query<Entity, With<EguiSgcsHolder>>,
    _opfilter_entity_map: Res<OpFilterEntityMap>,
) {
    if !map.0.is_empty() {
        return;
    }

    let mut hashid_sampler = HashIdWeightedSampler::default();
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
        let structure_id = structured_gen_seri.structure_id.clone();
        let main_ent = cmd.spawn_empty().id();

        let mut sgc = StructuredGenConfig::new(structure_id.as_str());
        let sgc_id = match StrId::new_with_result(seri_id.as_str(), 0) {
            Ok(sgc_id) => sgc_id,
            Err(err) => {
                error!(
                    target: SGC_INIT,
                    "Skipping StructuredGenConfig with invalid id '{}': {}",
                    seri_id,
                    err,
                );
                continue;
            }
        };

        sgc.max_per_region = structured_gen_seri.max_per_region;
        sgc.max_being_count = structured_gen_seri.max_being_count;
        sgc.whitelisted_tags = TagSet::new(structured_gen_seri.whitelisted_tags.iter().map(String::as_str));
        sgc.blacklisted_tags = TagSet::new(structured_gen_seri.blacklisted_tags.iter().map(String::as_str));
        sgc.typed_args = structured_gen_seri.args.clone();
        sgc.args = sgc.typed_args.to_legacy_args_dict();
        let configured_room_spawn_shapes = sgc.typed_args.room_spawn_shape_keys();
        if !configured_room_spawn_shapes.is_empty()
            && let Some(allowed_room_spawn_shapes) = sgc_command_registry
                .allowed_room_spawn_shapes_for(structure_id.as_str())
        {
            for configured_shape in configured_room_spawn_shapes {
                if allowed_room_spawn_shapes.contains(configured_shape.as_str()) {
                    continue;
                }
            }
        }

        let mut tags_vec = structured_gen_seri.tags.clone();
        if tags_vec.iter().all(|tag| tag != &seri_id) {
            tags_vec.push(seri_id.clone());
        }
        let tags_set = TagSet::new(tags_vec.iter());
        cmd.entity(main_ent).try_insert(TagSet::new(tags_vec));

        if !structured_gen_seri.exclusive_for_dimensions.is_empty() {
            let mut dim_refs = MultipleDimensionRefs::default();
            for dim_strid in &structured_gen_seri.exclusive_for_dimensions {
                let Ok(dim_ent) = dimension_entity_map.0.get_cloned(dim_strid) else {
                    error!(target: SGC_INIT, "Failed to find Dimension with StrId: {}", dim_strid);
                    continue;
                };
                let Ok(&dim_hash) = dimension_hash_query.get(dim_ent) else {
                    error!(target: SGC_INIT, "Dimension '{}' resolved to entity {:?} but it is missing HashId", dim_strid, dim_ent);
                    continue;
                };
                dim_refs.0.insert(dim_hash);
            }
            if dim_refs.0.is_empty() {
                continue;
            }
            exclusive_for_dims.push((main_ent, dim_refs));
        }
        if !structured_gen_seri.pdisk_mindist_and_tag.is_empty() {
            let Ok(poisson_disk) =
                PoissonDisk::multiple_tagged(structured_gen_seri.pdisk_mindist_and_tag.clone(), 3, 16)
            else {
                continue;
            };
            cmd.entity(main_ent).insert(poisson_disk);
        }

        if structured_gen_seri.priority > 0.0 {
            priority_defs.push(PrioritySgcDef {
                ent: main_ent,
                hash_id: HashId::hash(structured_gen_seri.structure_id.as_str()),
                priority: structured_gen_seri.priority,
                tags: tags_set,
                run_before_tags: structured_gen_seri.run_before_sgcs_with_tags,
                run_after_tags: structured_gen_seri.run_after_sgcs_with_tags,
            });
        }

        if structured_gen_seri.weight != f32::NEG_INFINITY {
            if let Err(negative_item) = hashid_sampler.insert(sgc_id.hash_id(), structured_gen_seri.weight) {
                error!(target: "regioning_sgc_init", "Weighted sampler {} encountered a negative weight for value {:?}; rejected", sgc_id.as_str(), negative_item);
            }
        }
        sgcs_comps.push((main_ent, (sgc_id.hash_id(), sgc_id, sgc, ReplicateIfServerStarts)));
    }

    let prioritized_hash_ids = build_prioritized_hash_ids(&priority_defs);

    cmd.insert_batch(exclusive_for_dims);
    cmd.insert_batch(sgcs_comps);
    cmd.spawn((PrioritizedSgs(prioritized_hash_ids), ReplicateIfServerStarts, Name::new("PrioritizedSgs"), ChildOf(egui_ent)));
    cmd.spawn((SgcsWeightedSampler, ReplicateIfServerStarts, hashid_sampler, ChildOf(egui_ent)));
}
