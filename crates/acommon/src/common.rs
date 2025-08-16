
use bevy_inspector_egui::inspector_egui_impls::InspectorEguiImpl;
use bevy_replicon::prelude::AppRuleExt;

use crate::{common_components::*, common_resources::*, common_states::*, common_systems::*, common_types::{FixedStr, HashIdToEntityMap}};

use {bevy::prelude::*,};

#[allow(unused_parens, path_statements, )]
pub fn plugin(app: &mut App) {
    app
        .add_systems(Update, (set_entity_name, update_img_sizes_on_load))

        .add_plugins((

        ))
        .insert_state::<AppState>(AppState::NoSession)
        .init_state::<PreGameState>()
        .init_state::<GamePhase>()
        .init_state::<GameSetupType>()
        .init_state::<AssetsLoadingState>()
        .init_state::<ConnectionAttempt>()
        .init_state::<LoadedAssetsSession>()
        .init_state::<TerrainGenHotLoading>()


        .init_resource::<ImageSizeMap>()
        .init_resource::<GlobalEntityMap>()
        
        .register_type::<EntityPrefix>().register_type_data::<EntityPrefix, InspectorEguiImpl>()
        
        .register_type::<DisplayName>()
        .register_type::<StrId>().register_type_data::<StrId, InspectorEguiImpl>()
        .register_type::<HashIdToEntityMap>()
        
        .replicate::<Name>()
        .replicate::<EntityPrefix>()
        .replicate::<StrId>()
        .replicate::<DisplayName>()

    ;
}