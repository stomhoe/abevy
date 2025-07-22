#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
#[allow(unused_imports)] use bevy_asset_loader::prelude::*;
use crate::game::being::sprite::{
    sprite_components::*,
//    sprite_constants::*,
//    sprite_events::*,
};

#[derive(AssetCollection, Resource)]
pub struct AnimSerisHandles {
    #[asset(path = "sprite/animation", collection(typed))]
    pub handles: Vec<Handle<AnimationSeri>>,
}