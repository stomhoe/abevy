use bevy::prelude::*;
use bevy::ecs::entity::EntityHashSet;
use bevy::ecs::schedule::common_conditions::on_message;
use bevy_replicon::prelude::*;
use common::common_components::*;
use common::log_targets::ENTITY_MAP_SYSTEM;
use sprite_shared::BaseHolderRef;

fn clone_recursive_and_collect_removals(
    cmd: &mut Commands,
    root_to_clone: Entity,
    children_query: &Query<(&Children,), (Without<EguiHolder>,)>,
    base_holder_ref_query: &Query<&BaseHolderRef>,
    local_msgs: &mut Vec<RemoveReplicated>,
) {
    let mut pending = Vec::with_capacity(16);
    pending.push((root_to_clone, None, 0usize, None));

    while let Some((source_ent, parent_clone, depth, source_parent)) = pending.pop() {
        let cloned = cmd
            .entity(source_ent)
            .clone_and_spawn_with_opt_out(move |builder| {
                if depth == 0 {
                    builder.deny::<(
                        RemoveReplicatedAfterClone,
                        Replicated,
                        Children,
                    )>();
                } else {
                    builder.deny::<(
                        RemoveReplicatedAfterClone,
                        Replicated,
                        ChildOf,
                        Children,
                    )>();
                }
            })
            .id();

        if let Some(parent_clone) = parent_clone {
            cmd.entity(cloned).try_insert(ChildOf(parent_clone));

            if let Some(source_parent) = source_parent {
                if let Ok(base_holder_ref) = base_holder_ref_query.get(source_ent) {
                    if base_holder_ref.base == source_parent {
                        cmd.entity(cloned).insert(BaseHolderRef { base: parent_clone });
                    }
                }
            }
        }

        trace!(
            target: ENTITY_MAP_SYSTEM,
            "Cloned replicated entity {:?} locally as {:?} and requested server removal",
            source_ent,
            cloned,
        );
        local_msgs.push(RemoveReplicated(source_ent));

        let Ok((children,)) = children_query.get(source_ent) else {
            continue;
        };
        pending.reserve(children.len());
        for child in children.iter() {
            pending.push((child, Some(cloned), depth + 1, Some(source_ent)));
        }
    }
}

#[allow(unused_parens, )]
pub fn clone_and_tell_server(
    mut cmd: Commands,
    query: Query<
        (Entity, Option<&ChildOf>,),
        (
            With<Replicated>,
            Added<RemoveReplicatedAfterClone>,
        ),
    >,
    children_query: Query<(&Children,), (Without<EguiHolder>,)>,
    base_holder_ref_query: Query<&BaseHolderRef>,
    mut writer: MessageWriter<RemoveReplicated>,
    mut local_msgs: Local<Vec<RemoveReplicated>>,
    mut ents_to_clone: Local<EntityHashSet>,
) {
    ents_to_clone.clear();
    for (entity, _child_of) in query.iter() {
        ents_to_clone.insert(entity);
    }

    for (entity, child_of_opt) in query.iter() {
        let is_root = child_of_opt
            .map(|child_of| !ents_to_clone.contains(&child_of.parent()))
            .unwrap_or(true);
        if !is_root {
            continue;
        }
        clone_recursive_and_collect_removals(
            &mut cmd,
            entity,
            &children_query,
            &base_holder_ref_query,
            &mut local_msgs,
        );
    }
    writer.write_batch(local_msgs.drain(..));
}

#[allow(unused_parens, )]
pub fn remove_replicated_after_clone_from_client(
    mut cmd: Commands,
    mut remove_requests: MessageReader<FromClient<RemoveReplicated>>,
) {
    for from_client in remove_requests.read() {
        let RemoveReplicated(ent) = from_client.message.clone();
        cmd.entity(ent).try_remove::<(Replicated, RemoveReplicatedAfterClone)>();
    }
}

