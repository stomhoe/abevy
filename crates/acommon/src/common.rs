use bevy::ecs::entity_disabling::Disabled;
use bevy_inspector_egui::inspector_egui_impls::InspectorEguiImpl;
use bevy_replicon::prelude::AppRuleExt;

use crate::{common_components::*, common_resources::*, common_states::*, common_systems::*, common_tag_components::TagSet, common_tag_systems::*, common_types::*};

use crate::qol::AppRegisterAndReplicateExt;

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

        .replicate::<Name>()
        .regrepli::<Prefix>()
        .register_type_data::<Prefix, InspectorEguiImpl>()
        .regrepli::<StrId>().register_type_data::<StrId, InspectorEguiImpl>()
        .regrepli::<ImagePathHolder>()
        .regrepli::<DisplayName>()
        .regrepli::<HashId>()
        .regrepli::<StrId>()
        .regrepli::<HashIdToEntityMap>()
        .regrepli::<Tag>().register_type_data::<Tag, InspectorEguiImpl>()
        .regrepli::<TagSet>()
        .regrepli::<VisibilityGameState>()



    ;
}

pub type AnyDisabling = Or<(With<Disabled>, Without<Disabled>)>;
