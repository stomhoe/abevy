
#[allow(unused_imports)] use bevy::prelude::*;
use bevy::ecs::entity::EntityHashSet;
use bevy_replicon::prelude::*;
use crate::log_targets::ENTITY_MAP_SYSTEM;
use crate::{
    common_components::*, common_resources::{ImageSizeMap, },
//    common_resources::*,
//    common_constants::*,
//    common_layout::*,
//    common_events::*,
};

#[allow(unused_parens)]
pub fn add_hash_id_from_str_id(mut cmd: Commands,
    query: Query<(Entity, AnyOf<(&StrId, &StrId20B)>),(Or<(Changed<StrId>, Changed<StrId20B>)>, With<AddHashIdFromStrId>, Without<HashId>, crate::AnyDisabling)>,
) {
    let to_add: Vec<_> = query
    .iter()
    .filter_map(|(entity, (str_id, str_id20b))| {
        if let Some(str_id) = str_id {
            Some((entity, HashId::from(str_id.as_str())))
        } else if let Some(str_id20b) = str_id20b {
            Some((entity, HashId::from(str_id20b.as_str())))
        } else {
            None
        }
    })
    .collect();
    cmd.try_insert_batch(to_add);
}

#[allow(unused_parens, )]
pub fn add_signature_from_hash_id(
    mut cmd: Commands,
    query: Query<(Entity, &HashId, ), (Without<Signature>, )>,
) {
    return;
    for (entity, hash_id) in query.iter() {
        trace!(
            target: ENTITY_MAP_SYSTEM,
            "Inserting Signature from HashId {:?} on entity {:?}",
            hash_id,
            entity,
        );
        cmd.entity(entity).try_insert(Signature::from(*hash_id));
    }
}


pub fn update_img_sizes_on_load(mut messages: MessageReader<AssetEvent<Image>>, assets: Res<Assets<Image>>,
    mut map: ResMut<ImageSizeMap>,) {
    for msg in messages.read() {
        match msg {
            AssetEvent::Added { id } => {
                if let Some(img) = assets.get(*id) {
                    let img_size = UVec2::new(img.texture_descriptor.size.width, img.texture_descriptor.size.height);
                    map.0.insert(id.clone(), img_size.as_u16vec2());
                }
            },
            _ => {}
        }
    }
}


#[allow(unused_parens, )]
pub fn sync_replicate_if_server_starts(
    mut cmd: Commands,
    changed_clients: Query<(), (Changed<ConnectedClient>, )>,
    mut removed_connected: RemovedComponents<ConnectedClient>,
    clients_query: Query<(), (With<ConnectedClient>, )>,
    query: Query<(Entity, ), (With<ReplicateIfServerStarts>, )>,
) {
    if changed_clients.is_empty() && removed_connected.is_empty() {
        return;
    }
    removed_connected.clear();

    let has_connected_players = !clients_query.is_empty();

    for (entity, ) in query.iter() {
        if has_connected_players {
            cmd.entity(entity).try_insert(RemoveReplicatedAfterClone);
        } else {
            cmd.entity(entity).try_remove::<(Replicated, RemoveReplicatedAfterClone)>();
        }
    }
}

fn clone_recursive_and_collect_removals(
    cmd: &mut Commands,
    root_to_clone: Entity,
    children_query: &Query<(&Children, ), (Without<EguiHolder>, )>,
    local_msgs: &mut Vec<RemoveReplicated>,
) {
    let mut pending = Vec::with_capacity(16);
    pending.push((root_to_clone, None, 0usize));

    while let Some((source_ent, parent_clone, depth)) = pending.pop() {
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
        }

        trace!(
            target: ENTITY_MAP_SYSTEM,
            "Cloned replicated entity {:?} locally as {:?} and requested server removal",
            source_ent,
            cloned
        );
        local_msgs.push(RemoveReplicated(source_ent));

        let Ok((children, )) = children_query.get(source_ent) else {
            continue;
        };
        pending.reserve(children.len());
        for child in children.iter() {
            pending.push((child, Some(cloned), depth + 1));
        }
    }
}

#[allow(unused_parens, )]
pub fn clone_and_tell_server(
    mut cmd: Commands,
    query: Query<
        (Entity, Option<&ChildOf>, ),
        (
            With<Replicated>,
            Added<RemoveReplicatedAfterClone>,
        ),
    >,
    children_query: Query<(&Children, ), (Without<EguiHolder>, )>,
    mut writer: MessageWriter<RemoveReplicated>,
    mut local_msgs: Local<Vec<RemoveReplicated>>,
) {
    let mut ents_to_clone = EntityHashSet::default();
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
        let RemoveReplicated(being_ent) = from_client.message.clone();
        debug!(
            target: ENTITY_MAP_SYSTEM,
            "Server removing Replicated from entity {:?} on client request",
            being_ent
        );
        cmd.entity(being_ent).try_remove::<(Replicated, RemoveReplicatedAfterClone)>();
    }
}
