
use crate::{common_components::*, common_resources::*, common_states::*, common_systems::*, common_types::HashIdToEntityMap};

use {bevy::prelude::*,};

#[allow(unused_parens, path_statements, )]
pub fn plugin(app: &mut App) {
    app
        .add_systems(Update, (set_entity_name, update_img_sizes_on_load))

        .add_plugins((

        ))
        .init_state::<AppState>()
        .init_state::<PreGameState>()
        .init_state::<GamePhase>()
        .init_state::<GameSetupType>()
        .init_state::<LocalAssetsLoadingState>()
        .init_state::<ReplicatedAssetsLoadingState>()
        .init_state::<ConnectionAttempt>()


        .init_resource::<ImageSizeMap>()
        .init_resource::<GlobalEntityMap>()
        .init_resource::<PlayerData>()

        .register_type::<DisplayName>()
        .register_type::<StrId>()
        .register_type::<HashId>()
        .register_type::<HashIdToEntityMap>()


    ;
}