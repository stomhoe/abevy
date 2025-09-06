
use bevy::ecs::entity_disabling::Disabled;
#[allow(unused_imports)] use bevy::prelude::*;
use crate::{
    common_components::*, common_resources::ImageSizeMap,
//    common_resources::*,
//    common_constants::*,
//    common_layout::*,
//    common_events::*,
};





#[allow(unused_parens)]
pub fn set_entity_name(//DESACTIVAR EN RELEASE BUILDS
    mut cmd: Commands,
    mut query: Query<(Entity, Option<&mut Name>, &EntityPrefix, Option<&StrId>, Option<&StrId20B>, Option<&DisplayName>), (Or<(Changed<EntityPrefix>, Changed<StrId>, Changed<DisplayName>, Or<(With<Disabled>, Without<Disabled>)>)>, )>,
) {
    for (ent, name, prefix, str_id, half_str_id, disp_name) in query.iter_mut() {
        let new_name = format!("{} {}{} {:?}", prefix, str_id.cloned().unwrap_or_default(),half_str_id.cloned().unwrap_or_default(), disp_name.cloned().unwrap_or_default());

        if let Some(mut name) = name {
            name.set(new_name);
        } else {
            cmd.entity(ent).try_insert(Name::new(new_name));
        }
    }
}

pub fn update_img_sizes_on_load(mut events: EventReader<AssetEvent<Image>>, assets: Res<Assets<Image>>, 
    mut map: ResMut<ImageSizeMap>,) {
    for ev in events.read() {
        match ev {
            AssetEvent::Added { id } => {
                if let Some(img) = assets.get(*id) {
                    let img_size = UVec2::new(img.texture_descriptor.size.width, img.texture_descriptor.size.height);
                    map.0.insert(Handle::Weak(id.clone()), img_size.as_u16vec2());
                }
            },
            _ => {}
        }
    }
}