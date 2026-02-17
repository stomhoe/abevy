pub mod operation_list_components;
pub mod operation_list_init_systems;
pub mod operation_list_resources;
pub mod operation_list_script;

use bevy::prelude::*;

#[allow(unused_imports)]
use operation_list_components::OperationList;
#[allow(unused_imports)]
use operation_list_resources::*;

#[allow(unused_parens, path_statements)]
pub fn plugin(app: &mut App) {
    app
        .add_plugins((
            plugin_operation_list,
        ))
        .init_resource::<OpListSerisHandles>()
        .init_asset::<OpListSeri>();
}
