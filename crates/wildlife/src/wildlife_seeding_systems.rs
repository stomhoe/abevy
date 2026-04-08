use bevy::{ecs::system::SystemParam, prelude::*};

#[allow(unused_imports, )]
use ::being_shared::*;
use common::log_targets::WILDLIFE_SYSTEM;
use ::game_common::*;
use tilemap::terrain::biome::biome_components::CreatureSampler;
use tilemap::terrain::operation_list::operation_list_components::OperationList;
use tilemap::terrain::operation_list::operation_list_resources::OperationListEntityMap;
use tilemap::chunking::macro_chunk_components::{BiomeDistribution, MacrochunkPendingBiomeSamples};
use ::tilemap_shared::{DimensionEntityMap, *};
use tilemap::terrain::terrgen_messages::*;

use crate::wildlife_spawning_helpers::*;

#[derive(SystemParam)]
pub struct SeedQueries<'w, 's> {
    macro_chunk_query: Query<'w, 's, (&'static DimensionRef, &'static MacrochunkPos, &'static BiomeDistribution)>,
    biome_pack_samplers: Query<'w, 's, &'static CreatureSampler>,
    pack_min_sep_query: Query<'w, 's, &'static PackMinSepToPacksOrRaces>,
    pack_query: Query<'w, 's, (), (With<Pack>, With<Templ>)>,
    race_query: Query<'w, 's, (), (With<Race>, With<Templ>)>,
    bit_query: Query<'w, 's, (), (With<BeingInstTemplate>, With<Templ>)>,
}

#[derive(SystemParam)]
pub struct SeedLocals<'s> {
    occupied_pack_anchor_chunkpos: Local<'s, Vec<PackAnchorCpos>>,
}

#[allow(unused_parens, )]
pub fn request_macrochunk_biome_sampling(
    mut cmd: Commands,
    mut loaded_macrochunks: MessageReader<NewMacrochunkLoaded>,
    mut macro_chunk_query: Query<(&DimensionRef, &MacrochunkPos, &mut MacrochunkPendingBiomeSamples, ), (With<MacroChunk>, )>,
    dimension_map: Res<DimensionEntityMap>,
    dimension_query: Query<&DimensionRootOplist>,
    oplist_map: Res<OperationListEntityMap>,
    oplists: Query<&OplistSize, With<OperationList>>,
    mut pending_ops_writer: MessageWriter<PendingOp>,
    mut pending_ops: Local<Vec<PendingOp>>,
    mut sample_positions: Local<Vec<GlobalTilePos>>,
) {
    pending_ops.clear();
    for msg in loaded_macrochunks.read() {
        let macro_chunk_ent = msg.macro_chunk_ent;
        let Ok((&dim_ref, &macro_chunk_pos, mut biome_state)) = macro_chunk_query.get_mut(macro_chunk_ent) else {
            continue;
        };
        if biome_state.0 != 0 {
            continue;
        }
        let Some(dim_ent) = dimension_map.0.get_opt(dim_ref.0).copied() else {
            error!(target: WILDLIFE_SYSTEM, "No dimension entity for macrochunk {} in {:?}", macro_chunk_pos, dim_ref);
            continue;
        };
        let Ok(root_oplist) = dimension_query.get(dim_ent) else {
            error!(target: WILDLIFE_SYSTEM, "No root operation list for macrochunk {} in {:?}", macro_chunk_pos, dim_ref);
            continue;
        };
        let Ok(root_oplist_ent) = oplist_map.0.get_cloned(root_oplist.0) else {
            error!(target: WILDLIFE_SYSTEM, "No root operation list entity mapped for hash {:?}", root_oplist.0);
            continue;
        };
        let Ok(_) = oplists.get(root_oplist_ent) else {
            error!(target: WILDLIFE_SYSTEM, "No oplist size for root operation list {:?}", root_oplist);
            continue;
        };
        sample_positions.clear();
        let sample_positions = macro_chunk_pos.sample_macro_chunk_positions(3, &mut sample_positions);
        let expected_samples = sample_positions.len() as u32;
        if expected_samples == 0 {
            cmd.entity(macro_chunk_ent).try_remove::<MacrochunkPendingBiomeSamples>();
            debug!(target: WILDLIFE_SYSTEM, "Completed biome sampling for macrochunk {} in {:?} without pending samples", macro_chunk_pos, dim_ref);
            continue;
        }
        biome_state.0 = expected_samples;
        for &gpos in sample_positions {
            pending_ops.push(PendingOp {
                oplist: *root_oplist,
                input: PendingOpInput {
                    dimension_ref: dim_ref,
                    gpos,
                },
                purpose: PendingOpPurpose::MacroChunkBiomeSampling {
                    macro_chunk_ent,
                },
            });
        }
        trace!(target: WILDLIFE_SYSTEM, "Queued {} biome samples for macrochunk {} in {:?}", expected_samples, macro_chunk_pos, dim_ref);
    }
    pending_ops_writer.write_batch(pending_ops.drain(..));
}

#[allow(unused_parens, )]
pub fn seed_natural_wildlife_for_biomesampled_macrochunks(
    mut macrochunk_finished_biomesampling: RemovedComponents<MacrochunkPendingBiomeSamples>,
    mut instance_pack_writer: MessageWriter<InstantiateTemplPackEntity>,
    queries: SeedQueries,
    mut locs: SeedLocals,
    mut pending_instance_pack_messages: Local<Vec<InstantiateTemplPackEntity>>,
) {
    let mut rng = rand::rng();
    pending_instance_pack_messages.clear();
    for macro_chunk_ent in macrochunk_finished_biomesampling.read() {
        let Ok((&dim_ref, &macro_chunk_pos, distribution)) = queries.macro_chunk_query.get(macro_chunk_ent) else {
            continue;
        };
        let Some(biome_ent) = distribution.sample_biome_ent(&mut rng) else {
            warn!(target: WILDLIFE_SYSTEM, "Natural spawn found no weighted biome for macrochunk {} in {:?}", macro_chunk_pos, dim_ref);
            continue;
        };
        let number_of_packs: usize = distribution
            .averaged_pack_count_multiplier_stats(biome_ent)
            .sample_pack_count_int_multiplier(&mut rng);

        locs.occupied_pack_anchor_chunkpos.clear();
        locs.occupied_pack_anchor_chunkpos.reserve(number_of_packs);
        let mut spawned_packs = 0usize;

        for _ in 0..number_of_packs {
            let Ok(biome_pack_sampler) = queries.biome_pack_samplers.get(biome_ent) else {
                warn!(target: WILDLIFE_SYSTEM, "Natural spawn found no candidate wildlife sampler for macrochunk {} in {:?}", macro_chunk_pos, dim_ref);
                break;
            };
            let Some(sampled_pack_or_race_or_bit_ent) = biome_pack_sampler.sample_pack_or_race_or_bit_entity(&mut rng) else {
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
                biome_ent,
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
            spawned_packs += 1;
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

        if spawned_packs > 0 {
            debug!(target: WILDLIFE_SYSTEM, "Natural spawn seeded macrochunk {} in {:?} with biome {:?}, packs {}", macro_chunk_pos, dim_ref, biome_ent, spawned_packs);
        } else {
        }
    }
    instance_pack_writer.write_batch(pending_instance_pack_messages.drain(..));
}
