#[allow(unused_imports)] use {bevy::prelude::*, superstate::superstate_plugin};

use crate::game::{AssetLoadingState, ActiveGameSystems};
use crate::game::tilemap::tile::{
    tile_systems::*,
     tile_resources::*,
};
mod tile_systems;
pub mod tile_components;
pub mod tile_resources;
pub mod tile_constants;
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct TileSystems;

//PLU ¡¡ NO OLVIDARSE DE METERLO EN .add_plugins((,Tile)) del módulo tilemap !!
pub struct TilePlugin;
#[allow(unused_parens, path_statements, )]
impl Plugin for TilePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Update, (update_img_sizes_on_load/*NO PONER EN SET */, ))
            .add_systems(Startup/*OnEnter(AssetLoadingState::InProcess)*/, (add_tileimgs_to_map, ))
            //.init_resource::<RESOURCE_NAME>()
            .add_plugins((
            // SomePlugin, 
            // superstate_plugin::<SuperState, (Substate1, Substate2)>
            ))
            .init_resource::<HandleConfigMap>()
            .init_state::<ImageSizeSetState>()

        ;
    }
}


#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum ImageSizeSetState {
    #[default]
    NotStarted,
    InProcess,
    Done,
}