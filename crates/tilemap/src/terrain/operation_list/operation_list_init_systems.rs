


use bevy::{ecs::entity::EntityHashSet, platform::collections::HashSet, prelude::*};

use bevy_replicon::shared::replication::Replicated;
use common::{common_components::*, common_tag_components::TagSet};

use crate::{
    chunking::macro_chunk_components::BiomeTagWeightAtMacrochunk,
    terrain::{
        biome::biome_resources::BiomeEntityMap,
        operation_list::operation_list_components::{Bifurcation, OperationList},
        operation_list::operation_list_resources::{EguiOperationListsHolder, OpListSeri, OperationListEntityMap, TgCompiledOpLists, load_op_list_seri_defs},
        operation_list::operation_list_script::load_tg_oplists,
        terrgen_components::FailedSearchOplistFilterHolder,
    },
    tile::{tile_resources::*, tile_sampler_resources::TileWeightedSamplerEntityMap},
};

#[allow(unused_parens)]
pub fn cache_tg_oplists(mut cmd: Commands) {
    cmd.insert_resource(TgCompiledOpLists(load_tg_oplists()));
}

fn merged_oplist_seris(tg_oplists: &TgCompiledOpLists) -> Vec<OpListSeri> {
    let mut seris = load_op_list_seri_defs();
    seris.extend(tg_oplists.0.iter().cloned());
    seris
}

#[allow(unused_parens)]
pub fn init_oplists_from_assets(
    mut cmd: Commands,
    samplers_map: Res<TileWeightedSamplerEntityMap>,
    tiles_map: Res<TileEntityMap>,
    biome_map: Res<BiomeEntityMap>,
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
    cmd.spawn((FailedSearchOplistFilterHolder, (Name::new("FailedPosSearches"), AssetScoped, ), ChildOf(egui_oplist_holder_ent)));

    let mut oplist_comps = Vec::new();
    let mut tags_to_insert = Vec::new();

    let mut seris = merged_oplist_seris(&tg_oplists);
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
                        let _ = sampler_ent;
                        Some(HashId::from(tile_str.as_str()))
                    } else if let Ok(tile_ent) = tiles_map.0.get_cloned(tile_str) {
                        let _ = tile_ent;
                        Some(HashId::from(tile_str.as_str()))
                    } else {
                        warn!(target: "oplist_init", "Tile {} not found in TileEntityMap or TileWeightedSamplerEntityMap", tile_str);
                        None
                    }
                })
                .collect::<Vec<HashId>>();

            let biome_tags = bif_seri
                .biome_tags
                .iter()
                .filter_map(|bt| {
                    if bt.tag.trim().is_empty() || !bt.weight.is_finite() || bt.weight <= 0.0 {
                        return None;
                    }
                    let Ok(_) = biome_map.0.get_cloned(bt.tag.trim()) else {
                        error!(target: "oplist_init", "Biome '{}' not found in BiomeEntityMap", bt.tag.trim());
                        return None;
                    };
                    Some(BiomeTagWeightAtMacrochunk {
                        biome: HashId::from(bt.tag.trim()),
                        weight: bt.weight,
                        pack_count_multiplier_mean: bt.pack_count_multiplier_mean.max(0.0),
                        pack_count_multiplier_std_dev: bt.pack_count_multiplier_std_dev.max(0.0),
                    })
                })
                .collect();
            let bifurcation = Bifurcation { oplist: None, tiles, biome_tags };
            oplist.bifurcations.push(bifurcation);
        }

        let oplist_hash = str_id.hash_id();
        if let Ok(ent) = oplist_map.0.get_cloned(oplist_hash) {
            error!(target: "oplist_init", "{} already in OperationListEntityMap : {}", str_id, ent);
            continue;
        }
        let spawned_oplist = cmd.spawn_empty().id();

        let expr_tree = seri.expr_tree.clone();
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
            let hid = debug_var.hash_id();
            let _ = oplist.hash_ids_mapped_to_strids.overwrite(hid, debug_var);
        }

        oplist_comps.push((spawned_oplist, (str_id.clone(), str_id.hash_id(), oplist, size, ReplicateIfServerStarts, ChildOf(egui_oplist_holder_ent))));

        if let Some(tags) = &seri.tags {
            tags_to_insert.push((spawned_oplist, TagSet::new(tags)));
        }
    }
    cmd.try_insert_batch(oplist_comps);
    cmd.try_insert_batch(tags_to_insert);
}

#[allow(unused_parens)]
pub fn validate_dimension_root_oplists(
    dimension_query: Query<(Entity, &StrId, &DimensionRootOplist), With<Dimension>>,
    oplist_map: Res<OperationListEntityMap>,
) {
    for (_, dim_id, root_oplist) in dimension_query.iter() {
        if oplist_map.0.get_cloned(root_oplist.0).is_ok() {
            continue;
        }
        error!(target: "oplist_init", "Dimension '{}' references missing root oplist hash {:?}", dim_id, root_oplist.0);
    }
}

#[allow(unused_parens)]
pub fn init_oplists_bifurcations(
    oplist_map: Res<OperationListEntityMap>,
    tg_oplists: Res<TgCompiledOpLists>,
    mut oplist_query: Query<(Entity, &mut OperationList, &OplistSize)>,
    dimension_root_query: Query<&DimensionRootOplist, With<Dimension>>,
) {
    let mut root_hashes: HashSet<HashId> = HashSet::default();
    for root_oplist in dimension_root_query.iter() {
        root_hashes.insert(root_oplist.0);
    }
    let seris = merged_oplist_seris(&tg_oplists);
    for seri in seris {
            let seri_hash = StrId::from(seri.id.as_str()).hash_id();
            let Ok(oplist_ent) = oplist_map.0.get_cloned(seri_hash) else {
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
                let bifurcation_hash = HashId::from(bifurcation_str);

                let Ok(bifurcation_ent) = oplist_map.0.get_cloned(bifurcation_hash) else {
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
                if root_hashes.contains(&bifurcation_hash) {
                    error!(target: "oplist_init", "bifurcation entity with id '{}' must not be a root oplist", bifurcation_str);
                    continue;
                }

                oplist.bifurcations[i].oplist = Some(bifurcation_hash);
            }
    }

}

#[allow(unused_parens, )]
pub fn cycle_detection(
    query: Query<(Entity, &OperationList, &StrId), ()>,
    dimension_root_query: Query<&DimensionRootOplist, With<Dimension>>,
    oplist_map: Res<OperationListEntityMap>,
) {
    let mut oplist_entities = HashIdMap::default();
    for (ent, _, str_id) in query.iter() {
        let _ = oplist_entities.overwrite(str_id.hash_id(), ent);
    }

    let mut roots = EntityHashSet::default();
    for root_oplist in dimension_root_query.iter() {
        let Ok(root_ent) = oplist_map.0.get_cloned(root_oplist.0) else {
            continue;
        };
        roots.insert(root_ent);
    }

    fn dfs(
        query: &Query<(Entity, &OperationList, &StrId), ()>,
        oplist_entities: &HashIdMap<Entity>,
        current: Entity,
        visited: &mut EntityHashSet,
        on_path: &mut EntityHashSet,
    ) -> bool {
        if on_path.contains(&current) {
            error!(target: "oplist_init", "Cycle detected, caused by oplist entity {:?}'s bifurcations", current);
            return true;
        }
        if !visited.insert(current) {
            return false;
        }
        on_path.insert(current);

        let Ok((_, oplist, _)) = query.get(current) else {
            on_path.remove(&current);
            return false;
        };

        for bifur in &oplist.bifurcations {
            let Some(child_hash) = bifur.oplist else { continue; };
            let Ok(child) = oplist_entities.get(child_hash) else {
                continue;
            };
            if dfs(query, oplist_entities, *child, visited, on_path) {
                return true;
            }
        }

        on_path.remove(&current);
        false
    }

    for root in roots {
        let mut visited = EntityHashSet::default();
        let mut on_path = EntityHashSet::default();
        if dfs(&query, &oplist_entities, root, &mut visited, &mut on_path) {
            error!(target: "oplist_init", "Cycle detected starting from root oplist {:?}", root);
        }
    }
}
