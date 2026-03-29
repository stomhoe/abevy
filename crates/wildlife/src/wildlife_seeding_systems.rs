use bevy::ecs::entity_disabling::Disabled;
use bevy::{ecs::system::SystemParam, prelude::*};
use being::being_bundles::BeingBundle;
use being::pack::pack_components::*;
use ::being_shared::*;
use common::file_logging::file_log;
use common::log_targets::WILDLIFE_SYSTEM;
use ::game_common::*;
use tilemap::terrain::biome::biome_components::CreatureSampler;
use tilemap::chunking::macro_chunk_components::{BiomeDistribution, MacrochunkPendingBiomeSamples};
use ::tilemap_shared::*;

use crate::wildlife_spawning_helpers::*;

#[derive(SystemParam)]
pub struct SeedQueries<'w, 's> {
    macro_chunk_query: Query<'w, 's, (&'static DimensionRef, &'static MacrochunkPos, &'static BiomeDistribution)>,
    biome_pack_samplers: Query<'w, 's, &'static CreatureSampler>,
    being_samplers: Query<'w, 's, &'static BeingTemplateSampler>,
    pack_min_sep_query: Query<'w, 's, &'static PackMinSepToPacksOrRaces>,
    pack_query: Query<'w, 's, (), (With<Pack>, With<Templ>)>,
    race_query: Query<'w, 's, (), (With<Race>, With<Templ>)>,
    bit_query: Query<'w, 's, (), (With<BeingInstTemplate>, With<Templ>)>,
    no_spawn_group_query: Query<'w, 's, (), With<NoSpawnGroup>>,
    spawn_pack_size_query: Query<'w, 's, &'static PackInitialSizeSampler>,
}

#[derive(SystemParam)]
pub struct SeedLocals<'s> {
    sampled_beings: Local<'s, Vec<Entity>>,
    occupied_pack_anchor_chunkpos: Local<'s, Vec<PackAnchorCpos>>,
}

#[derive(SystemParam)]
pub struct SeedRes<'w> {
    pending_wildlife_by_chunk: ResMut<'w, BeingsToEnableOnChunkLoad>,
}

#[allow(unused_parens, )]
pub fn seed_natural_wildlife_for_new_macro_chunks(
    mut cmd: Commands,
    mut macrochunk_finished_biomesampling: RemovedComponents<MacrochunkPendingBiomeSamples>,
    queries: SeedQueries,
    mut res: SeedRes,
    mut locs: SeedLocals,
) {
    let mut rng = rand::rng();
    for macro_chunk_ent in macrochunk_finished_biomesampling.read() {
        let Ok((&dim_ref, &macro_chunk_pos, distribution)) = queries.macro_chunk_query.get(macro_chunk_ent) else {
            continue;
        };
        let Some(biome_ent) = distribution.sample_biome_ent(&mut rng) else {
            warn!(target: WILDLIFE_SYSTEM, "Natural spawn found no weighted biome for macrochunk {} in {:?}", macro_chunk_pos, dim_ref);
            continue;
        };
        let rng_sampled_pack_count: usize = distribution
            .averaged_pack_count_multiplier_stats(biome_ent)
            .sample_pack_count_multiplier(&mut rng);

        locs.occupied_pack_anchor_chunkpos.clear();
        locs.occupied_pack_anchor_chunkpos.reserve(rng_sampled_pack_count);
        let mut spawned_packs = 0usize;
        let mut spawned_beings = 0usize;

        for _ in 0..rng_sampled_pack_count {
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
            let pack_sampled_being_count: usize = queries
                .spawn_pack_size_query
                .get(sampled_pack_or_race_or_bit_ent).ok()
                .map(|dist| dist.sample_count(&mut rng))
                .unwrap_or(1);
            let spawns_squad: bool = queries.no_spawn_group_query.get(sampled_pack_or_race_or_bit_ent).is_err();

            if from_templ_pack {
                let Ok(being_sampler) = queries.being_samplers.get(sampled_pack_or_race_or_bit_ent) else {
                    error!(target: WILDLIFE_SYSTEM, "Natural spawn found no being sampler for pack {:?} in macrochunk {} in {:?}", sampled_pack_or_race_or_bit_ent, macro_chunk_pos, dim_ref);
                    continue;
                };
                being_sampler.0.sample_n_with_rng(pack_sampled_being_count, &mut rng, &mut *locs.sampled_beings);
                if locs.sampled_beings.is_empty() {
                    error!(target: WILDLIFE_SYSTEM, "Natural spawn found no beings to sample from pack {:?} in macrochunk {} in {:?}", sampled_pack_or_race_or_bit_ent, macro_chunk_pos, dim_ref);
                    continue;
                }
            } else {
                // fill sampled_beings with the bit/race entity, so we can spawn beings from it
                locs.sampled_beings.resize(pack_sampled_being_count, sampled_pack_or_race_or_bit_ent);
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
            let beings_in_pack_count: usize = locs.sampled_beings.len();
            let mut assigned_cpos_for_each_being_in_pack = Pack::select_chunk_positions_around_anchor_cpos(
                macro_chunk_pos,
                pack_anchor_cpos,
                beings_in_pack_count,
                &mut rng,
            );
            if assigned_cpos_for_each_being_in_pack.is_empty() {
                trace!(target: WILDLIFE_SYSTEM, "Natural spawn found no target chunks for {:?} in macrochunk {} in {:?}", sampled_pack_or_race_or_bit_ent, macro_chunk_pos, dim_ref);
                continue;
            }

            locs.occupied_pack_anchor_chunkpos.push(PackAnchorCpos {
                pack_ent: sampled_pack_or_race_or_bit_ent,
                center_chunk: pack_anchor_cpos,
            });
            spawned_packs += 1;
            let squad_ent = spawns_squad.then(|| {
                let pack_entity = cmd.spawn((Pack, )).id();
                if from_templ_pack {
                    cmd.entity(pack_entity).insert(TemplEntiRef(sampled_pack_or_race_or_bit_ent));
                }
                pack_entity
            });
            for (selected_chunk_pos, spawn_target) in assigned_cpos_for_each_being_in_pack.drain(..).zip(locs.sampled_beings.drain(..)) {
                let is_bit = queries.bit_query.get(spawn_target).is_ok();
                let is_race = queries.race_query.get(spawn_target).is_ok();

                let gpos = selected_chunk_pos.random_gpos_within(&mut rng);
                let being_ent = cmd
                    .spawn((
                        BeingBundle::new(dim_ref, gpos),
                        Disabled,
                        NaturalSpawnOrigin(selected_chunk_pos),
                    ))
                    .id();
                if let Some(squad_ent) = squad_ent {
                    cmd.entity(being_ent).try_insert(SquadMemberOf(squad_ent));
                }
                res.pending_wildlife_by_chunk.insert(being_ent, dim_ref, selected_chunk_pos);
                if is_bit {
                    cmd.entity(being_ent).insert(BitRef(spawn_target));
                } else if is_race {
                    cmd.entity(being_ent).insert(RaceRef(spawn_target));
                }
                spawned_beings += 1;
            }
            if let Some(pack_entity) = squad_ent {
                debug!(target: WILDLIFE_SYSTEM, "Natural spawn created pack {:?} for target {:?} in macrochunk {} in {:?} with {} members", pack_entity, sampled_pack_or_race_or_bit_ent, macro_chunk_pos, dim_ref, locs.sampled_beings.len());
            }
        }

        if spawned_beings > 0 {
            debug!(target: WILDLIFE_SYSTEM, "Natural spawn seeded macrochunk {} in {:?} with biome {:?}, packs {}, beings {}", macro_chunk_pos, dim_ref, biome_ent, spawned_packs, spawned_beings);
        } else {
        }
    }
}
