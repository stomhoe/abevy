use bevy::prelude::*;
use common::common_states::ConnectionAttempt;
use tilemap::terrain_gen::terrgen_resources::{OpListEntityMap, TerrGenEntityMap};


pub fn all_clean_resources(
    mut cmd: Commands,
    mut conn_attempt: ResMut<NextState<ConnectionAttempt>>,
){
    conn_attempt.set(ConnectionAttempt::default());

    cmd.remove_resource::<TerrGenEntityMap>();
    cmd.remove_resource::<OpListEntityMap>();
}
