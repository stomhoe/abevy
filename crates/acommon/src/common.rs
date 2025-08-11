
use crate::{components::*, resources::*, states::*, systems::*};

use {bevy::prelude::*,};

#[allow(unused_parens, path_statements, )]
pub fn plugin(app: &mut App) {
    app
        .add_systems(Update, (set_entity_name,))
        .add_systems(Startup, spawn_camera)

        .add_plugins((
        ))
        .init_state::<PreGameState>()
        .init_state::<LocalAssetsLoadingState>()
        .init_state::<ReplicatedAssetsLoadingState>()
        .init_state::<ConnectionAttempt>()


        .init_resource::<ImageSizeMap>()
        .init_resource::<GlobalEntityMap>()
        .init_resource::<PlayerData>()

        .register_type::<DisplayName>()
        .register_type::<StrId>()
        .register_type::<HashId>()


    ;
}