use bevy::{ecs::system::SystemParam, prelude::*};

#[allow(unused_imports, )]
use ::being_shared::*;
use common::log_targets::WILDLIFE_SYSTEM;
use ::game_common::*;
use tilemap::terrain::biome::biome_components::CreatureSampler;
use tilemap::terrain::biome::biome_resources::BiomeEntityMap;
use tilemap::terrain::operation_list::operation_list_components::OperationList;
use tilemap::terrain::operation_list::operation_list_resources::OperationListEntityMap;
use tilemap::chunking::macro_chunk_components::{BiomeDistribution, MacrochunkPendingBiomeSamples};
use ::tilemap_shared::{DimensionEntityMap, *};
use tilemap::terrain::terrgen_messages::*;

use crate::wildlife_spawning_helpers::*;


#[allow(unused_parens, )]
pub fn request_macrochunk_biome_sampling(
    mut cmd: Commands,
    mut macro_chunk_query: Query<(Entity, &DimensionRef, &MacrochunkPos, &mut MacrochunkPendingBiomeSamples, ), (With<MacroChunk>, Added<MacrochunkPos>)>,
    dimension_map: Res<DimensionEntityMap>,
    dimension_query: Query<&DimensionRootOplist>,
    oplist_map: Res<OperationListEntityMap>,
    mut pending_ops_writer: MessageWriter<PendingOp>,
    mut pending_ops: Local<Vec<PendingOp>>,
    mut sample_positions: Local<Vec<GlobalTilePos>>,
) {
    pending_ops.clear();
    for (macro_chunk_ent, &dim, &macro_chunk_pos, mut pending_samples) in macro_chunk_query.iter_mut() {
        if pending_samples.0 != 0 {
            continue;
        }
        let Some(dim_ent) = dimension_map.0.get_opt(dim.0).copied() else {
            error!(target: WILDLIFE_SYSTEM, "No dimension entity for macrochunk {} in {:?}", macro_chunk_pos, dim);
            continue;
        };
        let Ok(root_oplist) = dimension_query.get(dim_ent) else {
            error!(target: WILDLIFE_SYSTEM, "No root operation list for macrochunk {} in {:?}", macro_chunk_pos, dim);
            continue;
        };
        let Ok(_root_oplist_ent) = oplist_map.0.get_cloned(root_oplist.0) else {
            error!(target: WILDLIFE_SYSTEM, "No root operation list entity mapped for hash {:?}", root_oplist.0);
            continue;
        };
        sample_positions.clear();
        let sample_positions = macro_chunk_pos.gather_gpos_to_sample(&mut sample_positions, 3);
        let expected_samples = sample_positions.len() as u32;
        if expected_samples == 0 {
            cmd.entity(macro_chunk_ent).try_remove::<MacrochunkPendingBiomeSamples>();
            continue;
        }
        pending_samples.0 = expected_samples;
        for &gpos in sample_positions {
            pending_ops.push(PendingOp {
                oplist: *root_oplist,
                input: PendingOpInput {
                    dim,
                    gpos,
                },
                purpose: PendingOpPurpose::BiomeSampling {
                    macro_chunk_ent,
                },
            });
        }
        trace!(target: WILDLIFE_SYSTEM, "Queued {} biome samples for macrochunk {} in {:?}", expected_samples, macro_chunk_pos, dim);
    }
    pending_ops_writer.write_batch(pending_ops.drain(..));
}

#[derive(SystemParam)]
#[allow(non_camel_case_types, )]
pub struct init_natural_Queries<'w, 's> {
    macro_chunk_query: Query<'w, 's, (&'static DimensionRef, &'static MacrochunkPos, &'static BiomeDistribution)>,
    biome_pack_samplers: Query<'w, 's, &'static CreatureSampler>,
    pack_min_sep_query: Query<'w, 's, &'static PackMinSepToPacksOrRaces>,
    pack_query: Query<'w, 's, (), (With<Pack>, With<Templ>)>,
    race_query: Query<'w, 's, (), (With<Race>, With<Templ>)>,
    bit_query: Query<'w, 's, (), (With<BeingInstTemplate>, With<Templ>)>,
    biome_map: Res<'w, BiomeEntityMap>,
    pack_map: Res<'w, PackEntityMap>,
    race_map: Res<'w, RaceEntityMap>,
    bit_map: Res<'w, BeingInstTemplateEntityMap>,
}

#[derive(SystemParam)]
#[allow(non_camel_case_types, )]
pub struct init_naturalLocals<'s> {
    occupied_pack_anchor_chunkpos: Local<'s, Vec<PackAnchorCpos>>,
}
#[allow(unused_parens, )]
pub fn init_natural_wildlife_for_biomesampled_macrochunks(
    mut macrochunk_finished_biomesampling: RemovedComponents<MacrochunkPendingBiomeSamples>,
    mut instance_pack_writer: MessageWriter<InstantiateTemplPackEntity>,
    queries: init_natural_Queries,
    mut locs: init_naturalLocals,
    mut pending_instance_pack_messages: Local<Vec<InstantiateTemplPackEntity>>,
) {
    let mut rng = rand::rng();
    pending_instance_pack_messages.clear();
    for macro_chunk_ent in macrochunk_finished_biomesampling.read() {
        let Ok((&dim_ref, &macro_chunk_pos, distribution)) = queries.macro_chunk_query.get(macro_chunk_ent) else {
            continue;
        };
        let Some(biome_hash_id) = distribution.sample_biome_hash_id(&mut rng) else {
            warn!(target: WILDLIFE_SYSTEM, "Natural spawn found no weighted biome for macrochunk {} in {:?}", macro_chunk_pos, dim_ref);
            continue;
        };
        let Some(biome_ent) = queries.biome_map.0.get_opt(biome_hash_id).copied() else {
            warn!(target: WILDLIFE_SYSTEM, "Natural spawn found weighted biome hash {:?} with no biome entity for macrochunk {} in {:?}", biome_hash_id, macro_chunk_pos, dim_ref);
            continue;
        };
        let number_of_packs: usize = distribution
            .averaged_pack_count_multiplier_stats(biome_hash_id)
            .sample_pack_count_int_multiplier(&mut rng);

        locs.occupied_pack_anchor_chunkpos.clear();
        locs.occupied_pack_anchor_chunkpos.reserve(number_of_packs);

        for _ in 0..number_of_packs {
            let Ok(biome_pack_sampler) = queries.biome_pack_samplers.get(biome_ent) else {
                warn!(target: WILDLIFE_SYSTEM, "Natural spawn found no candidate wildlife sampler for macrochunk {} in {:?}", macro_chunk_pos, dim_ref);
                break;
            };
            let Some(sampled_pack_or_race_or_bit_ent) = biome_pack_sampler.sample_pack_or_race_or_bit_entity(
                &mut rng,
                &[&queries.pack_map.0, &queries.race_map.0, &queries.bit_map.0],
            ) else {
                warn!(target: WILDLIFE_SYSTEM, "Natural spawn found no available wildlife candidate for macrochunk {} in {:?} after affinity filtering", macro_chunk_pos, dim_ref);
                break;
            };
            let from_bit: bool = queries.bit_query.get(sampled_pack_or_race_or_bit_ent).is_ok();
            let from_race: bool = queries.race_query.get(sampled_pack_or_race_or_bit_ent).is_ok();
            let from_templ_pack: bool = queries.pack_query.get(sampled_pack_or_race_or_bit_ent).is_ok();

            if !from_bit && !from_race && !from_templ_pack {
                error!(target: WILDLIFE_SYSTEM, "Sampled entity to spawn {:?} for macrochunk {} in {:?} is not a BIT, Race, or Pack entity", sampled_pack_or_race_or_bit_ent, macro_chunk_pos, dim_ref);
                continue;
            }
            let pack_min_dists: Option<&PackMinSepToPacksOrRaces>
                = queries.pack_min_sep_query.get(sampled_pack_or_race_or_bit_ent).ok();
            let Some(pack_anchor_cpos) = choose_best_anchor_cpos_for_pack(
                distribution,
                biome_hash_id,
                sampled_pack_or_race_or_bit_ent,
                macro_chunk_pos,
                &locs.occupied_pack_anchor_chunkpos,
                pack_min_dists,
                &queries.pack_min_sep_query,
            ) else {
                trace!(target: WILDLIFE_SYSTEM, "Natural spawn found no affinity-valid pack center for target {:?} in macrochunk {} in {:?}", sampled_pack_or_race_or_bit_ent, macro_chunk_pos, dim_ref);
                break;
            };

            locs.occupied_pack_anchor_chunkpos.push(PackAnchorCpos {
                pack_ent: sampled_pack_or_race_or_bit_ent,
                center_chunk: pack_anchor_cpos,
            });
            let instance_pack_message = InstantiateTemplPackEntity::new(
                sampled_pack_or_race_or_bit_ent,
                None,
                None,
                None,
                dim_ref,
                [pack_anchor_cpos.center_gpos()],
            );
            pending_instance_pack_messages.push(instance_pack_message);
        }
    }
    instance_pack_writer.write_batch(pending_instance_pack_messages.drain(..));
}
