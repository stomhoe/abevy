#[macro_export]
macro_rules! run_suitable_pos_search_logic {
    (
        target: $target:expr,
        searched_entity_label: $searched_entity_label:expr,
        cmd: $cmd:ident,
        searching_entities: $searching_entities:ident,
        search_params: $search_params:ident,
        make_search_request: $make_search_request:ident,
        handle_success_event: $handle_success_event:ident,
        handle_pending_failure: $handle_pending_failure:ident,
    ) => {{
        $search_params.pending_by_requester.clear();
        $search_params.requester_collect_all.clear();
        for (ent, &dim_ref, &my_pos, &ezero_ref, searching_for) in $searching_entities.iter() {
            if let Some($crate::terrain::terrprobe::terrprobe_components::SearchingForSuitablePos {
                requester,
                collect_all_successes,
            }) = searching_for {
                $search_params
                    .pending_by_requester
                    .entry(*requester)
                    .or_default()
                    .push((ent, my_pos, dim_ref, ezero_ref));
                $search_params.requester_collect_all.insert(*requester, *collect_all_successes);
                $search_params.requester_had_success.entry(*requester).or_insert(false);
            }
        }

        $searching_entities
            .iter().for_each(|(search_ent, &dim_ref, &global_pos, ezero_ref, ..)| {
                $cmd.entity(search_ent).try_remove::<$crate::terrain::terrprobe::terrprobe_components::AwaitingStartSearch>();

                let Some(mut probe) =
                    $make_search_request(&mut $cmd, search_ent, global_pos, *ezero_ref)
                else {
                    return;
                };
                if probe.requester == Entity::PLACEHOLDER {
                    probe.requester = search_ent;
                }
                let requester = probe.requester;

                info!(
                    target: $target,
                    "Starting suitable-pos search for {} entity {:?} at position {:?}",
                    $searched_entity_label,
                    search_ent,
                    global_pos
                );

                $cmd.entity(search_ent)
                    .try_insert($crate::terrain::terrprobe::terrprobe_components::SearchingForSuitablePos {
                        requester,
                        collect_all_successes: probe.collect_all_successes,
                    });
                $search_params
                    .requester_collect_all
                    .insert(requester, probe.collect_all_successes);
                $search_params
                    .requester_had_success
                    .insert(requester, false);
                $search_params
                    .min_result_distance_by_requester
                    .insert(requester, probe.min_result_distance as u64);
                $search_params.pos_searches_msgs_to_write.push(probe);
                $search_params
                    .pending_by_requester
                    .entry(requester)
                    .or_default()
                    .push((search_ent, global_pos, dim_ref, *ezero_ref));
            });

        let mut accepted_results: Vec<(DimensionRef, GlobalTilePos)> = Vec::new();
        for suitable_pos in $search_params.reader_search_successful.read() {
            let requester = suitable_pos.requester;
            let min_result_distance = $search_params
                .min_result_distance_by_requester
                .get(&requester)
                .copied()
                .unwrap_or(0);
            let min_result_distance_sq = min_result_distance.saturating_mul(min_result_distance);

            let mut completed_search_ent: Option<Entity> = None;
            let mut remove_requester = false;
            {
                let Some(owners) = $search_params.pending_by_requester.get_mut(&requester) else {
                    continue;
                };
                let Some((search_ent, my_pos, dim_ref, ezero_ref)) = owners.last().copied() else {
                    continue;
                };

                let too_close = min_result_distance_sq > 0 && accepted_results.iter().any(|(taken_dim_ref, taken_pos)| {
                    *taken_dim_ref == dim_ref
                        && suitable_pos.found_pos.distance_squared(taken_pos) <= min_result_distance_sq
                });
                if too_close {
                    trace!(
                        target: $target,
                        "Skipping suitable-pos result for requester {:?} at {:?} due to min_result_distance {}",
                        requester,
                        suitable_pos.found_pos,
                        min_result_distance
                    );
                    continue;
                }

                if $handle_success_event(
                    &mut $cmd,
                    search_ent,
                    my_pos,
                    dim_ref,
                    ezero_ref,
                    suitable_pos.found_pos,
                    suitable_pos.val,
                    suitable_pos.is_last,
                ) {
                    let collect_all_successes = $search_params
                        .requester_collect_all
                        .get(&requester)
                        .copied()
                        .unwrap_or(false);
                    if collect_all_successes {
                        if let Some(had_success) = $search_params.requester_had_success.get_mut(&requester) {
                            *had_success = true;
                        }
                    } else {
                        owners.pop();
                        remove_requester = owners.is_empty();
                        completed_search_ent = Some(search_ent);
                    }
                    accepted_results.push((dim_ref, suitable_pos.found_pos));
                }
            }
            if remove_requester {
                $search_params.pending_by_requester.remove(&requester);
                $search_params.min_result_distance_by_requester.remove(&requester);
                $search_params.requester_collect_all.remove(&requester);
                $search_params.requester_had_success.remove(&requester);
            }
            if let Some(search_ent) = completed_search_ent {
                $cmd.entity(search_ent).try_remove::<$crate::terrain::terrprobe::terrprobe_components::SearchingForSuitablePos>();
            }
        }

        for failed_search in $search_params.mreader_search_failed.read() {
            let Some(pending_searches) = $search_params.pending_by_requester.remove(&failed_search.0) else {
                continue;
            };
            let collect_all_successes = $search_params
                .requester_collect_all
                .remove(&failed_search.0)
                .unwrap_or(false);
            let had_success = $search_params
                .requester_had_success
                .remove(&failed_search.0)
                .unwrap_or(false);
            $search_params.min_result_distance_by_requester.remove(&failed_search.0);
            for (search_ent, global_pos, dim_ref, ezero_ref) in pending_searches {
                $cmd.entity(search_ent).try_remove::<$crate::terrain::terrprobe::terrprobe_components::SearchingForSuitablePos>();
                if !(collect_all_successes && had_success) {
                    error!(
                        target: $target,
                        "Failed to find suitable pos for a {} entity, {:?}",
                        $searched_entity_label,
                        failed_search.0
                    );
                    $handle_pending_failure(search_ent, global_pos, dim_ref, ezero_ref, failed_search.0);
                }
            }
        }

        $search_params.write_pos_searches();
    }};
}

#[macro_export]
macro_rules! run_oneshot_suitable_pos_search_logic {
    (
        target: $target:expr,
        searched_label: $searched_label:expr,
        cmd: $cmd:ident,
        search_params: $search_params:ident,
        active_probe_ent: $active_probe_ent:ident,
        search_finished: $search_finished:ident,
        make_search_request: $make_search_request:ident,
        handle_success: $handle_success:ident,
        handle_failure: $handle_failure:ident,
    ) => {{
        if *$search_finished {
            $search_params.write_pos_searches();
        } else {
            if $active_probe_ent.is_none() {
                if let Some(mut probe) = $make_search_request(&mut $cmd) {
                    if probe.requester == Entity::PLACEHOLDER {
                        probe.requester = $cmd.spawn_empty().id();
                    }
                    let requester = probe.requester;
                    info!(
                        target: $target,
                        "Starting one-shot suitable-pos search for {} with requester {:?}",
                        $searched_label,
                        requester
                    );
                    *$active_probe_ent = Some(requester);
                    $search_params.pos_searches_msgs_to_write.push(probe);
                }
            }

            if let Some(active_probe_ent) = *$active_probe_ent {
                for suitable_pos in $search_params.reader_search_successful.read() {
                    if suitable_pos.requester != active_probe_ent {
                        continue;
                    }
                    if $handle_success(
                        &mut $cmd,
                        suitable_pos.found_pos,
                        active_probe_ent,
                        suitable_pos.val,
                    ) {
                        *$search_finished = true;
                        *$active_probe_ent = None;
                        break;
                    }
                }

                if !*$search_finished {
                    for failed_search in $search_params.mreader_search_failed.read() {
                        if failed_search.0 != active_probe_ent {
                            continue;
                        }
                        error!(
                            target: $target,
                            "Failed to find suitable pos for one-shot {} search, terrain probe {:?}",
                            $searched_label,
                            active_probe_ent
                        );
                        $handle_failure(&mut $cmd, active_probe_ent);
                        *$search_finished = true;
                        *$active_probe_ent = None;
                        break;
                    }
                }
            }

            $search_params.write_pos_searches();
        }
    }};
}

#[macro_export]
macro_rules! run_sampled_value_matrix_search_logic {
    (
        target: $target:expr,
        searched_entity_label: $searched_entity_label:expr,
        cmd: $cmd:ident,
        searching_entities: $searching_entities:ident,
        search_params: $search_params:ident,
        make_search_request: $make_search_request:ident,
        handle_sampled_values_event: $handle_sampled_values_event:ident,
        handle_pending_failure: $handle_pending_failure:ident,
    ) => {{
        $search_params.pending_by_requester.clear();
        $search_params.requester_collect_all.clear();
        for (ent, &dim_ref, &my_pos, &ezero_ref, _, searching_for) in $searching_entities.iter() {
            if let Some($crate::terrain::terrprobe::terrprobe_components::SearchingForSuitablePos {
                requester,
                collect_all_successes,
            }) = searching_for {
                $search_params
                    .pending_by_requester
                    .entry(*requester)
                    .or_default()
                    .push((ent, my_pos, dim_ref, ezero_ref));
                $search_params.requester_collect_all.insert(*requester, *collect_all_successes);
                $search_params.requester_had_success.entry(*requester).or_insert(false);
            }
        }

        $searching_entities
            .iter().for_each(|(search_ent, &dim_ref, &global_pos, ezero_ref, is_awaiting_start, ..)| {
                if !is_awaiting_start {
                    return;
                }
                $cmd.entity(search_ent).try_remove::<$crate::terrain::terrprobe::terrprobe_components::AwaitingStartSearch>();

                let Some(mut probe) =
                    $make_search_request(&mut $cmd, search_ent, global_pos, *ezero_ref)
                else {
                    return;
                };
                if probe.requester == Entity::PLACEHOLDER {
                    probe.requester = search_ent;
                }
                let requester = probe.requester;

                info!(
                    target: $target,
                    "Starting sampled-value probe search for {} entity {:?} at position {:?}",
                    $searched_entity_label,
                    search_ent,
                    global_pos
                );

                $cmd.entity(search_ent)
                    .try_insert($crate::terrain::terrprobe::terrprobe_components::SearchingForSuitablePos {
                        requester,
                        collect_all_successes: probe.collect_all_successes,
                    });
                $search_params
                    .requester_collect_all
                    .insert(requester, probe.collect_all_successes);
                $search_params
                    .requester_had_success
                    .insert(requester, false);
                $search_params
                    .min_result_distance_by_requester
                    .insert(requester, probe.min_result_distance as u64);
                $search_params.pos_searches_msgs_to_write.push(probe);
                $search_params
                    .pending_by_requester
                    .entry(requester)
                    .or_default()
                    .push((search_ent, global_pos, dim_ref, *ezero_ref));
            });

        for sampled_values in $search_params.reader_sampled_value_matrix.read() {
            let requester = sampled_values.requester;
            let mut completed_search_ent: Option<Entity> = None;
            let mut remove_requester = false;

            {
                let Some(owners) = $search_params.pending_by_requester.get_mut(&requester) else {
                    continue;
                };
                let Some((search_ent, my_pos, dim_ref, ezero_ref)) = owners.last().copied() else {
                    continue;
                };
                if $handle_sampled_values_event(
                    &mut $cmd,
                    search_ent,
                    my_pos,
                    dim_ref,
                    ezero_ref,
                    &sampled_values.matrix.values,
                ) {
                    owners.pop();
                    remove_requester = owners.is_empty();
                    completed_search_ent = Some(search_ent);
                }
            }

            if remove_requester {
                $search_params.pending_by_requester.remove(&requester);
                $search_params.min_result_distance_by_requester.remove(&requester);
                $search_params.requester_collect_all.remove(&requester);
                $search_params.requester_had_success.remove(&requester);
            }
            if let Some(search_ent) = completed_search_ent {
                $cmd.entity(search_ent).try_remove::<$crate::terrain::terrprobe::terrprobe_components::SearchingForSuitablePos>();
            }
        }

        for failed_search in $search_params.mreader_search_failed.read() {
            let Some(pending_searches) = $search_params.pending_by_requester.remove(&failed_search.0) else {
                continue;
            };
            $search_params.requester_collect_all.remove(&failed_search.0);
            $search_params.requester_had_success.remove(&failed_search.0);
            $search_params.min_result_distance_by_requester.remove(&failed_search.0);
            for (search_ent, global_pos, dim_ref, ezero_ref) in pending_searches {
                $cmd.entity(search_ent).try_remove::<$crate::terrain::terrprobe::terrprobe_components::SearchingForSuitablePos>();
                error!(
                    target: $target,
                    "Failed sampled-value probe search for {} entity, {:?}",
                    $searched_entity_label,
                    failed_search.0
                );
                $handle_pending_failure(search_ent, global_pos, dim_ref, ezero_ref, failed_search.0);
            }
        }

        $search_params.write_pos_searches();
    }};
}
