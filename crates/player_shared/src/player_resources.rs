#[allow(unused_imports)] use bevy::prelude::*;
use common::common_components::StrId;

#[derive(Resource, Debug, )]
pub struct PlayerData { pub username: StrId, }
impl Default for PlayerData {
    fn default() -> Self {
        let username = StrId::trunc(format!("Player-{}", nano_id::base64::<6>()));
        Self { username }
    }
}
