
#[allow(unused_imports)] use bevy::prelude::*;
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
