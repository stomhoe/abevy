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
        for (ent, &dim_ref, &my_pos, &ezero_ref, _, searching_for) in $searching_entities.iter() {
            if let Some($crate::terrain::terrprobe::terrprobe_components::SearchingForSuitablePos { requester }) = searching_for {
                $search_params
                    .pending_by_requester
                    .entry(*requester)
                    .or_default()
                    .push((ent, my_pos, dim_ref, ezero_ref));
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
                    "Starting suitable-pos search for {} entity {:?} at position {:?}",
                    $searched_entity_label,
                    search_ent,
                    global_pos
                );

                $cmd.entity(search_ent)
                    .try_insert($crate::terrain::terrprobe::terrprobe_components::SearchingForSuitablePos { requester });
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
            let Some(owners) = $search_params.pending_by_requester.get_mut(&requester) else {
                continue;
            };
            let Some(&(search_ent, my_pos, dim_ref, ezero_ref)) = owners.last() else {
                continue;
            };

            let min_result_distance = $search_params
                .min_result_distance_by_requester
                .get(&requester)
                .copied()
                .unwrap_or(0);
            let min_result_distance_sq = min_result_distance.saturating_mul(min_result_distance);
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
            ) {
                owners.pop();
                if owners.is_empty() {
                    $search_params.pending_by_requester.remove(&requester);
                    $search_params.min_result_distance_by_requester.remove(&requester);
                }
                accepted_results.push((dim_ref, suitable_pos.found_pos));
                $cmd.entity(search_ent).try_remove::<$crate::terrain::terrprobe::terrprobe_components::SearchingForSuitablePos>();
            }
        }

        for failed_search in $search_params.mreader_search_failed.read() {
            let Some(pending_searches) = $search_params.pending_by_requester.remove(&failed_search.0) else {
                continue;
            };
            $search_params.min_result_distance_by_requester.remove(&failed_search.0);
            for (search_ent, global_pos, dim_ref, ezero_ref) in pending_searches {
                error!(
                    target: $target,
                    "Failed to find suitable pos for a {} entity, {:?}",
                    $searched_entity_label,
                    failed_search.0
                );
                $cmd.entity(search_ent).try_remove::<$crate::terrain::terrprobe::terrprobe_components::SearchingForSuitablePos>();
                $handle_pending_failure(search_ent, global_pos, dim_ref, ezero_ref, failed_search.0);
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
