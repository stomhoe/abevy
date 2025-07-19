#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
use serde::{Deserialize, Serialize};

use crate::game::being::sprite::{animation_resources::*, sprite_components::*};

#[allow(unused_parens)]
pub fn init_sprites(
    mut cmd: Commands, 
    aserver: Res<AssetServer>,
    race_seris: ResMut<Assets<SpriteDataSeri>>,
    mut map: ResMut<IdSpriteDataEntityMap>,

) {
        
    let human_body0 = cmd.spawn((
        SpriteDataId::new("human_body0"),
        DefaultBodyBundle::new("being/body/human_male.png"),
    )).id();

    let human_head0 = cmd.spawn((
        SpriteDataId::new("human_head0"),
        DefaultHeadBundle::new("being/head/human/0.png"),
    )).id();
}


pub fn apply_offsets(
    mut query: Query<(
        &mut Transform,
        Option<&Offset>,
        Option<&OffsetLookingDown>,
        Option<&OffsetLookingUp>,
        Option<&OffsetLookingSideways>,
        Option<&OffsetLookingLeft>,
        Option<&OffsetLookingRight>,
    ), (With<Sprite>, With<ChildOf>)
    >,
) {
   
    for (mut transform, offset, offset_looking_down, offset_looking_up, offset_looking_sideways, offset_looking_left, offset_looking_right) in query.iter_mut() {
        let mut total_offset: Vec3 = Vec3::ZERO;
        if let Some(offset) = offset {
            total_offset += offset.0;
        }
        if let Some(offset_looking_down) = offset_looking_down {
            total_offset += offset_looking_down.0.extend(0.0);
        }
        if let Some(offset_looking_up) = offset_looking_up {
            total_offset += offset_looking_up.0.extend(0.0);
        }
        if let Some(offset_looking_sideways) = offset_looking_sideways {
            total_offset += offset_looking_sideways.0.extend(0.0);
        }
        if let Some(offset_looking_left) = offset_looking_left {
            total_offset += offset_looking_left.0.extend(0.0);
        }
        if let Some(offset_looking_right) = offset_looking_right {
            total_offset += offset_looking_right.0.extend(0.0);
        }
        transform.translation = total_offset;
    }
}
