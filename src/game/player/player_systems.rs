

use bevy::prelude::*;

use crate::game::player::player_components::*;


fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {

}

#[derive(States, Debug, Clone, PartialEq, Eq, Hash, )]
#[states(scoped_entities)]
enum CameraState {
    FollowingBeing(Entity),
    Second,
    Third,
}

#[allow(dead_code)]
pub fn movement(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mut player_query: Single<&mut Transform, With<SelfPlayer>>,
    mut cam_query: Single<(&mut Transform, &mut Projection), With<Camera>>,
) {
    let (mut transform, mut projection) = cam_query.into_inner();
    let mut direction = Vec3::ZERO;

    if keys.pressed(KeyCode::KeyW) {direction.y += 1.0;}
    if keys.pressed(KeyCode::KeyS) {direction.y -= 1.0;}
    if keys.pressed(KeyCode::KeyA) {direction.x -= 1.0;}
    if keys.pressed(KeyCode::KeyD) {direction.x += 1.0;}
    if direction != Vec3::ZERO {
        direction = direction.normalize() * 150.0 * time.delta_secs(); // Speed of the player
        //player_transform.translation += direction;
    }

    let z = transform.translation.z;
    transform.translation += time.delta_secs() * direction * 500.;
    // Important! We need to restore the Z values when moving the camera around.
    // Bevy has a specific camera setup and this can mess with how our layers are shown.
    transform.translation.z = z;
}
