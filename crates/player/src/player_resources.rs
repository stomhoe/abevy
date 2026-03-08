#[allow(unused_imports)] use bevy::prelude::*;
use common::common_components::StrId;

#[derive(Resource, Debug, )]
pub struct PlayerData { pub username: StrId, }
impl Default for PlayerData {
    fn default() -> Self {
        let username = StrId::new_with_result(format!("Player-{}", nano_id::base64::<6>()), 0).expect("Failed to create StrId for playerdata");
        Self { username }
    }
}
