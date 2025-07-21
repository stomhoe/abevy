use bevy::log::tracing::span::Id;
#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
use serde::{Deserialize, Serialize};
use crate::game::being::{race::{
   race_components::*, race_constants::*, race_resources::*, 
   //race_events::*,
}, sprite::sprite_resources::SpriteDataIdEntityMap};


pub fn init_races(
    mut cmd: Commands,
    mut seris_handles: ResMut<RaceSerisHandles>,
    mut assets: ResMut<Assets<RaceSeri>>,
    mut strid_ent_map: ResMut<RaceIdEntityMap>, 
    sprite_map: Res<SpriteDataIdEntityMap>,
) {
    
    let handles_vec = std::mem::take(&mut seris_handles.handles);
    
    for handle in handles_vec {
        strid_ent_map.new_race_from_seri(&mut cmd, handle, &mut assets, &sprite_map);
    }
}




