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
pub fn set_entity_name(
    mut cmd: Commands,
    mut query: Query<(Entity, &EntityPrefix, Option<&StrId>, Option<&DisplayName>), (Or<(Changed<EntityPrefix>, Changed<StrId>, Changed<DisplayName>, Or<(With<Disabled>, Without<Disabled>)>)>, )>,
) {
    for (ent, prefix, str_id, disp_name) in query.iter_mut() {
        let new_name = format!("{} {} {:?}", prefix, str_id.cloned().unwrap_or_default(), disp_name.cloned().unwrap_or_default());
        cmd.entity(ent).insert(Name::new(new_name));
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