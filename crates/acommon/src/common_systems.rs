
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
    query: Query<(Entity, &StrId),(With<AddHashIdFromStrId>, Without<HashId>, )>,
) {
    let to_add: Vec<_> = query
    .iter()
    .map(|(entity, str_id)| (entity, HashId::from(str_id.as_str())))
    .collect();
    cmd.try_insert_batch(to_add);
}


pub fn update_img_sizes_on_load(mut events: MessageReader<AssetEvent<Image>>, assets: Res<Assets<Image>>, 
    mut map: ResMut<ImageSizeMap>,) {
    for ev in events.read() {
        match ev {
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