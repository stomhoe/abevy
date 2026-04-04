
#[allow(unused_imports)] use bevy::prelude::*;
use bevy_replicon::prelude::*;
use crate::log_targets::ENTITY_MAP_SYSTEM;
use crate::{
    common_components::*, common_resources::ImageSizeMap,
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
pub fn clone_and_tell_server(
    mut cmd: Commands,
    query: Query<
        Entity,
        (
            With<Replicated>,
            Added<RemoveReplicatedAfterClone>,
        ),
    >,
    mut writer: MessageWriter<RemoveReplicated>,
    mut local_msgs: Local<Vec<RemoveReplicated>>,
) {
    for entity in query.iter() {
        let cloned = cmd.entity(entity).clone_and_spawn_with_opt_out(|builder| {
            builder.deny::<Replicated>();
            builder.deny::<RemoveReplicatedAfterClone>();
        }).id();
        debug!(
            target: ENTITY_MAP_SYSTEM,
            "Cloned replicated entity {:?} locally as {:?} and requested server removal",
            entity,
            cloned
        );
        local_msgs.push(RemoveReplicated(entity));
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
        cmd.entity(being_ent).try_remove::<Replicated>();
    }
}
