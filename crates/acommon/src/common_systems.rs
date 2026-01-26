
use bevy::ecs::entity_disabling::Disabled;
use std::fmt::Write;
#[allow(unused_imports)] use bevy::prelude::*;
use crate::{
    common_components::*, common_resources::ImageSizeMap,
//    common_resources::*,
//    common_constants::*,
//    common_layout::*,
//    common_events::*,
};


/// DEACTIVATE THIS SYSTEM IN RELEASE BUILDS !!!!
#[allow(unused_parens)]
pub fn set_entity_name(
    mut query: Query<(&mut Name, AnyOf<(&Prefix, &StrId, &StrId20B, &DisplayName)>), 
    (
        Or<(Changed<Prefix>, Changed<StrId>, Changed<StrId20B>, Changed<DisplayName>,)>, 
    AnyDisabling)>,
) {
    for (mut name, (e_prefix, strid, strid20b, display_name)) in query.iter_mut() {
        let prefix = e_prefix.map(|p| p.as_str()).unwrap_or("");
        let sid = strid.map(|s| s.as_str()).unwrap_or("");
        let sid20 = strid20b.map(|s| s.as_str()).unwrap_or("");

        let mut new_name = String::with_capacity(
            prefix.len() + 1 + sid.len() + sid20.len() + display_name.map(|d| d.0.len() + 2).unwrap_or(0),
        );

        new_name.push_str(prefix);
        new_name.push(' ');
        new_name.push_str(sid);
        new_name.push_str(sid20);

        if let Some(dn) = display_name {
            new_name.push(' ');
            let _ = write!(new_name, "{:?}", dn.0.as_str());
        }

        name.set(new_name);
    }
}


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