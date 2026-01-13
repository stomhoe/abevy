
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
    mut query: Query<(&mut Name, AnyOf<(&EntityPrefix, &StrId, &StrId20B, &DisplayName)>), 
    (Or<(Changed<EntityPrefix>, Changed<StrId>, Changed<DisplayName>, DisabledOrNot)>, )>,
) {
    for (mut name, (e_prefix, strid, strid20b, display_name)) in query.iter_mut() {
        let display_name = if let Some(display_name) = display_name {
            format!(" {:?}", display_name.0.as_str())
        } else {
            "".to_string()
        };

        let new_name = format!("{} {}{}{}", e_prefix.cloned().unwrap_or_default(), strid.cloned().unwrap_or_default(), strid20b.cloned().unwrap_or_default(), display_name);

        name.set(new_name);
        
    }
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