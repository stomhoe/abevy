
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
    mut query: Query<(Entity, &mut Name, AnyOf<(&EntityPrefix, &StrId, &StrId20B, &DisplayName)>), 
    (Or<(Changed<EntityPrefix>, Changed<StrId>, Changed<DisplayName>, Or<(With<Disabled>, Without<Disabled>)>)>, )>,
) {
    for (ent, mut name, any_of) in query.iter_mut() {
        let new_name = format!("{} {}{} {:?}", any_of.0.cloned().unwrap_or_default(), any_of.1.cloned().unwrap_or_default(), any_of.2.cloned().unwrap_or_default(), any_of.3.cloned().unwrap_or_default());

        name.set(new_name);
        
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