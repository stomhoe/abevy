use bevy::prelude::*;
use bevy_replicon::prelude::*;
use common::common_components::*;
use common::common_resources::*;
use common::log_targets::ENTITY_MAP_SYSTEM;
use common::AnyDisabling;

#[allow(unused_parens)]
pub fn add_hash_id_from_str_id(
    mut cmd: Commands,
    query: Query<(
        Entity,
        AnyOf<(&StrId, &StrId20B)>,
    ), (
        Or<(Changed<StrId>, Changed<StrId20B>)>,
        With<AddHashIdFromStrId>,
        Without<HashId>,
        AnyDisabling,
    )>,
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

#[allow(unused_parens, unused)]
pub fn add_signature_from_hash_id(
    mut cmd: Commands,
    query: Query<(Entity, &HashId), (Without<Signature>,)>,
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

pub fn update_img_sizes_on_load(
    mut messages: MessageReader<AssetEvent<Image>>,
    assets: Res<Assets<Image>>,
    mut map: ResMut<ImageSizeMap>,
    mut pending_updates: ResMut<RegisteredImageSizeUpdateObservers>,
    mut ready_messages: Local<Vec<ImageSizeReady>>,
    mut ready_writer: MessageWriter<ImageSizeReady>,
) {
    for msg in messages.read() {
        match msg {
            AssetEvent::Added { id } => {
                if let Some(img) = assets.get(*id) {
                    let img_size = UVec2::new(
                        img.texture_descriptor.size.width,
                        img.texture_descriptor.size.height,
                    );
                    map.0.insert(id.clone(), img_size.as_u16vec2());
                    let entities = pending_updates.take_entities(*id);
                    ready_messages.reserve(entities.len());
                    for entity in entities {
                        ready_messages.push(ImageSizeReady { entity, image_id: *id });
                    }
                }
            }
            _ => {}
        }
    }
    ready_writer.write_batch(ready_messages.drain(..));
}

#[allow(unused_parens, )]
pub fn sync_replicate_if_server_starts(
    mut cmd: Commands,
    changed_clients: Query<(), (Changed<ConnectedClient>, )>,
    mut removed_connected: RemovedComponents<ConnectedClient>,
    clients_query: Query<(), (With<ConnectedClient>, )>,
    query: Query<(Entity,), (With<ReplicateIfServerStarts>,)>,
) {
    if changed_clients.is_empty() && removed_connected.is_empty() {
        return;
    }
    removed_connected.clear();

    let has_connected_players = !clients_query.is_empty();

    for (entity,) in query.iter() {
        if has_connected_players {
            cmd.entity(entity).try_insert(RemoveReplicatedAfterClone);
        } else {
            cmd.entity(entity).try_remove::<(Replicated, RemoveReplicatedAfterClone)>();
        }
    }
}

