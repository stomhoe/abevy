#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
#[allow(unused_imports)] use bevy_asset_loader::prelude::*;
use crate::common::{
    common_components::*,
//    common_resources::*,
//    common_constants::*,
//    common_layout::*,
//    common_events::*,
};


#[allow(unused_parens)]
pub fn set_entity_name(
    mut cmd: Commands,
    mut query: Query<(Entity, &EntityPrefix, Option<&StrId>, Option<&DisplayName>), (Or<(Changed<EntityPrefix>, Changed<StrId>, Changed<DisplayName>)>, )>,
) {
    for (ent, prefix, str_id, disp_name) in query.iter_mut() {
        let new_name = format!("{} {} {:?}", prefix, str_id.cloned().unwrap_or_default(), disp_name.cloned().unwrap_or_default());
        cmd.entity(ent).insert(Name::new(new_name));
    }
}
