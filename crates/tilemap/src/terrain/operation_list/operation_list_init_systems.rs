


use bevy::{ecs::entity::EntityHashMap, prelude::*};

use common::{common_components::{ StrId}, common_tag_components::TagSet};

use crate::{
    chunking::macro_chunk_components::BiomeTagWeightAtMacroChunk,
    terrain::{
        biome::biome_resources::BiomeEntityMap,
        operation_list::operation_list_components::{Bifurcation, CompiledBranch, CompiledBranchNode, OperationList},
        operation_list::operation_list_resources::{EguiOperationListsHolder, OperationListEntityMap, TgCompiledOpLists, load_op_list_seri_defs},
        operation_list::operation_list_script::load_tg_oplists,
        terrgen_components::FailedSearchOplistFilterHolder,
        terrgen_expression,
        terrgen_resources::TerrgenEntityMap,
    },
    tile::{tile_resources::*, tile_sampler_resources::TileWeightedSamplerEntityMap},
};
use ::tilemap_shared::*;

use std::collections::{HashMap, HashSet};

#[allow(unused_parens)]
pub fn cache_tg_oplists(mut cmd: Commands) {
    cmd.insert_resource(TgCompiledOpLists(load_tg_oplists()));
}

/// Resolve NoiseByName variants in expression tree to actual Noise entities
fn resolve_noise_names_in_expr(
    expr: &mut terrgen_expression::Expr,
    terr_gen_map: &TerrgenEntityMap,
) {
    use terrgen_expression::Expr;

    match expr {
        Expr::NoiseByName { name, sample_range, complement, seed_offset } => {
            if let Ok(entity) = terr_gen_map.0.get_cloned(*name) {
                *expr = Expr::Noise {
                    entity,
                    sample_range: *sample_range,
                    complement: *complement,
                    seed_offset: *seed_offset,
                };
            } else {
                error!(target: "oplist_init", "Noise entity not found while resolving TG expr hash: {:?}", name);
            }
        }
        Expr::Add { left, right }
        | Expr::Subtract { left, right }
        | Expr::Multiply { left, right }
        | Expr::Divide { left, right }
        | Expr::MultiplyNormalized { left, right }
        | Expr::MultiplyNormalizedAbs { left, right } => {
            resolve_noise_names_in_expr(left, terr_gen_map);
            resolve_noise_names_in_expr(right, terr_gen_map);
        }
        Expr::MultiplyOpo { value }
        | Expr::Abs { value }
        | Expr::Complement { value } => {
            resolve_noise_names_in_expr(value, terr_gen_map);
        }
        Expr::Min { values }
        | Expr::Max { values }
        | Expr::Average { values }
        | Expr::IndexMax { values }
        | Expr::Linear { values } => {
            for v in values {
                resolve_noise_names_in_expr(v, terr_gen_map);
            }
        }
        Expr::IndexNorm { value, multiplier } => {
            resolve_noise_names_in_expr(value, terr_gen_map);
            resolve_noise_names_in_expr(multiplier, terr_gen_map);
        }
        Expr::Clamp { value, min, max } => {
            resolve_noise_names_in_expr(value, terr_gen_map);
            resolve_noise_names_in_expr(min, terr_gen_map);
            resolve_noise_names_in_expr(max, terr_gen_map);
        }
        _ => {}
    }
}

#[allow(unused_parens)]
pub fn init_oplists_from_assets(
    mut cmd: Commands,
    terr_gen_map: Res<TerrgenEntityMap>,
    samplers_map: Res<TileWeightedSamplerEntityMap>,
    tiles_map: Res<TileEntityMap>,
    biome_map: Res<BiomeEntityMap>,
    dimension_map: Res<DimensionEntityMap>,
    oplist_map: Res<OperationListEntityMap>,
    tg_oplists: Res<TgCompiledOpLists>,
    egui_holder: Query<Entity, With<EguiOperationListsHolder>>,
) {
    if !oplist_map.0.is_empty() { return ; }

    let egui_oplist_holder_ent = if let Ok(egui_holder_ent) = egui_holder.single() {
        egui_holder_ent
    } else {
        let egui_oplist_holder_ent = cmd.spawn(EguiOperationListsHolder).id();
        egui_oplist_holder_ent
    };
    cmd.spawn((FailedSearchOplistFilterHolder, ChildOf(egui_oplist_holder_ent)));

    let mut oplist_comps = Vec::new();
    let mut oplist_multiple_dimension_refs = Vec::new();
    let mut tags_to_insert = Vec::new();

    let mut seris = load_op_list_seri_defs();
    seris.extend(tg_oplists.0.iter().cloned());
    for seri in seris.iter_mut() {
        let str_id = match StrId::new_with_result(seri.id.clone(), 1) {
            Ok(str_id) => str_id,
            Err(err) => {
                error!(target: "oplist_init", "Failed to create StrId for oplist {}: {:?}", seri.id, err);
                continue;
            }
        };
        let size = if let Some(size) = seri.size {
            if let Ok(size) = OplistSize::new(size) {
                size
            } else {
                error!(target: "oplist_init", "Invalid oplist_size for {}, must be in [1,4] for each vec component", seri.id);
                continue;
            }
        } else {
            OplistSize::default()
        };

        let mut oplist = OperationList::default();

        // Build bifurcations from seri
        oplist.bifurcations = Vec::with_capacity(seri.bifs.len());
        for bif_seri in seri.bifs.iter() {
            let tiles = bif_seri
                .tiles
                .iter()
                .filter(|tile_str| !tile_str.is_empty())
                .filter_map(|tile_str| {
                    if let Ok(sampler_ent) = samplers_map.0.get_cloned(tile_str) {
                        Some(sampler_ent)
                    } else if let Ok(tile_ent) = tiles_map.0.get_cloned(tile_str) {
                        Some(tile_ent)
                    } else {
                        warn!(target: "oplist_init", "Tile {} not found in TilingEntityMap or TileWeightedSamplerEntityMap", tile_str);
                        None
                    }
                })
                .collect::<Vec<Entity>>();

            let biome_tags = bif_seri
                .biome_tags
                .iter()
                .filter_map(|bt| {
                    if bt.tag.trim().is_empty() || !bt.weight.is_finite() || bt.weight <= 0.0 {
                        return None;
                    }
                    let Ok(biome_ent) = biome_map.0.get_cloned(bt.tag.trim()) else {
                        error!(target: "oplist_init", "Biome '{}' not found in BiomeEntityMap", bt.tag.trim());
                        return None;
                    };
                    Some(BiomeTagWeightAtMacroChunk {
                        biome: biome_ent,
                        weight: bt.weight,
                        pack_count_multiplier_mean: bt.pack_count_multiplier_mean.max(0.0),
                        pack_count_multiplier_std_dev: bt.pack_count_multiplier_std_dev.max(0.0),
                    })
                })
                .collect();
            let bifurcation = Bifurcation { oplist: None, tiles, biome_tags };
            oplist.bifurcations.push(bifurcation);
        }

        if let Ok(ent) = oplist_map.0.get_cloned(&str_id) {
            error!(target: "oplist_init", "{} already in OperationListEntityMap : {}", str_id, ent);
            continue;
        }
        let spawned_oplist = cmd.spawn_empty().id();

        let mut expr_tree = seri.expr_tree.clone();
        for assignment in expr_tree.assignments.iter_mut() {
            resolve_noise_names_in_expr(&mut assignment.expr, &terr_gen_map);
        }
        resolve_noise_names_in_expr(&mut expr_tree.output, &terr_gen_map);
        oplist.expr_tree = expr_tree;

        let debug_vars = seri.debug_vars.iter().filter_map(|debug_var| {
            match StrId::new_with_result(debug_var.clone(), 1) {
                Ok(str_id) => Some(str_id),
                Err(err) => {
                    error!(
                        target: "oplist_init",
                        "Failed to create StrId for debug var '{}' in oplist {}: {:?}",
                        debug_var,
                        seri.id,
                        err
                    );
                    None
                }
            }
        }).collect::<Vec<_>>();
        for debug_var in debug_vars {
            let hid = common::common_components::HashId::from(debug_var.as_str());
            let _ = oplist.hash_ids_mapped_to_strids.overwrite(hid, debug_var);
        }

        oplist_comps.push((spawned_oplist, (str_id, oplist, size, ChildOf(egui_oplist_holder_ent))));
        if seri.is_root() {
            let mut dim_refs = MultipleDimensionRefs::default();
            for dim_id in seri.root_in_dimensions.iter() {
                if dim_id.trim().is_empty() { continue; }
                let Ok(dim_entity) = dimension_map.0.get_cloned(&dim_id) else {
                    error!(target: "oplist_init", "Dimension '{}' not found in DimensionEntityMap for root oplist '{}'", dim_id, seri.id);
                    continue;
                };
                dim_refs.0.insert(dim_entity);
            }
            oplist_multiple_dimension_refs.push((spawned_oplist, dim_refs));
        }

        if let Some(tags) = &seri.tags {
            tags_to_insert.push((spawned_oplist, TagSet::new(tags)));
        }
    }
    cmd.try_insert_batch(oplist_comps);
    cmd.try_insert_batch(oplist_multiple_dimension_refs);
    cmd.try_insert_batch(tags_to_insert);
}

#[allow(unused_parens)]
pub fn init_oplists_bifurcations(
    mut cmd: Commands,
    oplist_map: Res<OperationListEntityMap>,
    tg_oplists: Res<TgCompiledOpLists>,
    mut oplist_query: Query<(Entity, &mut OperationList, &OplistSize)>,
    is_root: Query<&MultipleDimensionRefs>,
) -> Result {
    let mut seris = load_op_list_seri_defs();
    seris.extend(tg_oplists.0.iter().cloned());
    for seri in seris {
            let Ok(oplist_ent) = oplist_map.0.get_cloned(&seri.id) else {
                error!(
                    target: "oplist_init",
                    "oplist entity with id '{}' not found in OperationListEntityMap",
                    seri.id
                );
                continue;
            };
            let Ok((_, mut oplist, _)) = oplist_query.get_mut(oplist_ent) else {
                error!(target: "oplist_init", "oplist entity '{}' missing OperationList component", seri.id);
                continue;
            };

            for (i, seri_bifurcation) in seri.bifs.iter().enumerate() {
                let bifurcation_str = seri_bifurcation.oplist.trim();
                if bifurcation_str.is_empty() { continue; }

                let Ok(bifurcation_ent) = oplist_map.0.get_cloned(&bifurcation_str.to_string()) else {
                    error!(
                        target: "oplist_init",
                        "bifurcation entity with id '{}' not found in OperationListEntityMap",
                        bifurcation_str
                    );
                    continue;
                };
                if oplist_ent == bifurcation_ent {
                    error!(target: "oplist_init", "bifurcation entity with id '{}' would make parent diverge into itself ", bifurcation_str);
                    continue;
                }
                if is_root.get(bifurcation_ent).is_ok() {
                    error!(target: "oplist_init", "bifurcation entity with id '{}' must not be a root oplist", bifurcation_str);
                    continue;
                }

                cmd.entity(bifurcation_ent).insert(ChildOf(oplist_ent));
                oplist.bifurcations[i].oplist = Some(bifurcation_ent);
            }
    }

    let mut snapshots: HashMap<Entity, (crate::terrain::terrgen_expression::ExprOpList, Vec<Bifurcation>, OplistSize)> =
        HashMap::new();
    for (ent, oplist, &size) in oplist_query.iter() {
        snapshots.insert(ent, (oplist.expr_tree.clone(), oplist.bifurcations.clone(), size));
    }

    let mut cache: HashMap<Entity, CompiledBranchNode> = HashMap::new();
    for ent in snapshots.keys().copied() {
        let mut stack = HashSet::new();
        let _ = compile_branch_node(ent, &snapshots, &mut cache, &mut stack);
    }

    for (ent, mut oplist, _) in oplist_query.iter_mut() {
        oplist.compiled_branch_ast = cache.get(&ent).cloned();
    }
    Ok(())
}

fn compile_branch_node(
    ent: Entity,
    snapshots: &HashMap<Entity, (crate::terrain::terrgen_expression::ExprOpList, Vec<Bifurcation>, OplistSize)>,
    cache: &mut HashMap<Entity, CompiledBranchNode>,
    stack: &mut HashSet<Entity>,
) -> Option<CompiledBranchNode> {
    if let Some(found) = cache.get(&ent) {
        return Some(found.clone());
    }
    if !stack.insert(ent) {
        error!(target: "oplist_init", "Cycle detected while compiling branch AST at entity {:?}", ent);
        return None;
    }
    let Some((expr_tree, bifurcations, _)) = snapshots.get(&ent) else {
        stack.remove(&ent);
        return None;
    };

    let mut branches = Vec::with_capacity(bifurcations.len());
    for bif in bifurcations {
        let (child, child_size) = if let Some(child_ent) = bif.oplist {
            let child_size = snapshots.get(&child_ent).map(|(_, _, size)| *size);
            let child = compile_branch_node(child_ent, snapshots, cache, stack).map(Box::new);
            (child, child_size)
        } else {
            (None, None)
        };
        branches.push(CompiledBranch {
            tiles: bif.tiles.clone(),
            biome_tags: bif.biome_tags.clone(),
            child_size,
            child,
        });
    }
    let compiled = CompiledBranchNode {
        source_oplist: ent,
        expr_tree: expr_tree.clone(),
        branches,
    };
    cache.insert(ent, compiled.clone());
    stack.remove(&ent);
    Some(compiled)
}

#[allow(unused_parens)]
pub fn cycle_detection(
    query: Query<(Entity, &OperationList, Has<MultipleDimensionStringRefs>)>,
) {
    let roots: Vec<Entity> = query
    .iter()
    .filter_map(|(ent, _, is_root)| if is_root { Some(ent) } else { None })
    .collect();
    fn dfs(
        query: &Query<(Entity, &OperationList, Has<MultipleDimensionStringRefs>)>,
        current: Entity,
        visited: &mut HashSet<Entity>,
        stack: &mut Vec<Entity>,
    ) -> bool {
        if stack.contains(&current) {
            error!(target: "oplist_init", "Cycle detected, caused by oplist entity {:?}'s bifurcations", current);
            return true;
        }
        if !visited.insert(current) {
            return false;
        }
        stack.push(current);

        let Ok((_, oplist, _)) = query.get(current) else {
            stack.pop();
            return false;
        };
        for bifur in &oplist.bifurcations {
            if let Some(child) = bifur.oplist {
                if dfs(query, child, visited, stack) {
                    return true;
                }
            }
        }
        stack.pop();
        false
    }
    for root in roots {
        let mut visited = HashSet::new();
        let mut stack = Vec::new();
        if dfs(&query, root, &mut visited, &mut stack) {
            error!(target: "oplist_init", "Cycle detected starting from root oplist {:?}", root);
        }
    }
}
#[allow(unused_parens)]
pub fn assign_rootoplist_to_dimensions(mut cmd: Commands,
    oplist_query: Query<(Entity, &StrId, &MultipleDimensionRefs),(With<OperationList>, )>,
    dimension_query: Query<(&StrId, Option<&DimensionRootOplist>), With<Dimension>>,
) {
    let mut assignments: EntityHashMap<DimensionRootOplist> = EntityHashMap::new();


    for (oplist_ent, my_oplist_id, dim_refs) in oplist_query.iter() {
        for &dim_ent in dim_refs.0.iter() {
            let Ok((dim_str_id, root_op_list)) = dimension_query.get(dim_ent) else {
                error!(target: "oplist_init", "Dimension entity '{}' referenced by DimensionEntityMap is not spawned in world", dim_ent);
                continue;
            };

            match (assignments.get(&dim_ent), root_op_list) {
                (Some(&other_ent), _) => {
                    if other_ent.0 == oplist_ent { trace!(target: "oplist_init", "self is already dimoplist"); continue; }
                    let Ok((_, other_id, _, )) = oplist_query.get(other_ent.0) else {
                        continue;
                    };
                    error!(target: "oplist_init", "Dimension {} already has root operation list {}; couldn't assign {} as its root oplist", dim_str_id, other_id, my_oplist_id);
                    continue;
                },
                (_, Some(&DimensionRootOplist(other_ent))) => {
                    if other_ent == oplist_ent { trace!(target: "oplist_init", "self is already dimoplist"); continue; }

                    let Ok((_, other_id, _, )) = oplist_query.get(other_ent) else {
                        continue;
                    };
                    error!(target: "oplist_init", "Dimension {} already has root operation list {}; couldn't assign {} as its root oplist", dim_str_id, other_id, my_oplist_id);
                    continue;
                },
                (None, None) => {
                    assignments.insert(dim_ent, DimensionRootOplist(oplist_ent));
                },
            }
        }
    }
    cmd.try_insert_batch(assignments);
}
