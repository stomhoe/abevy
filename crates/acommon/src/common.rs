pub use crate::common_components::*;
pub use crate::common_id_components::*;
pub use crate::common_resources::*;
pub use crate::common_states::*;
pub use crate::common_tag_components::*;
pub use crate::common_types::*;
pub use crate::def_db::*;
#[allow(unused_imports, )]pub use crate::entity_map_macros::*;
pub use crate::file_logging::*;
pub use crate::log_targets::*;
#[allow(unused_imports, )]
pub use crate::marker_macros::*;

use bevy::ecs::entity_disabling::Disabled;
use bevy_replicon::prelude::AppRuleExt;

use crate::{common_systems::*, common_tag_systems::*, };


use {bevy::prelude::*,};

#[allow(unused_parens, path_statements, )]
pub fn plugin(app: &mut App) {
    app
        .add_systems(Update,
            (update_img_sizes_on_load,
                add_hash_id_from_str_id,
                add_hashed_tags, ))
        .add_plugins(())
        .insert_state::<AppState>(AppState::NoSession)
        .init_state::<PreGameState>()
        .init_state::<GamePhase>()

        .init_resource::<ImageSizeMap>()
        .init_resource::<GlobalEntityMap>()
        .init_resource::<crate::def_db::DefValidationConfig>()
        .init_resource::<crate::def_db::DefValidationRuntime>()

        .replicate::<Name>()
        .replicate::<Prefix>()
        //.register_type_data::<Prefix, InspectorEguiImpl>()
        .replicate::<StrId>()//.register_type_data::<StrId, InspectorEguiImpl>()
        .replicate::<ImagePathHolder>()
        .replicate::<DisplayName>()
        .replicate::<HashId>()
        .replicate::<StrId>()
        .replicate::<HashIdToEntityMap>()
        .replicate::<Tag>()//.register_type_data::<Tag, InspectorEguiImpl>()
        .replicate::<TagSet>()
        .replicate::<VisibilityGameState>()



    ;
}

pub type AnyDisabling = Or<(With<Disabled>, Without<Disabled>)>;
