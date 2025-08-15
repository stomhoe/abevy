#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
use common::common_components::{DisplayName, EntityPrefix, StrId};
use crate::{
    dimension_components::*,
    dimension_resources::*,
/*
    dimension_events::*,
    dimension_layout::*,
*/
};

#[allow(unused_parens)]
pub fn init_dimensions(
    mut cmd: Commands, map: Option<Res<DimensionEntityMap>>,
    mut seris_handles: ResMut<DimensionSerisHandles>,
    mut assets: ResMut<Assets<DimensionSeri>>,
) -> Result {
    if map.is_some(){ return Ok(()); }
    cmd.init_resource::<DimensionEntityMap>();

    for handle in std::mem::take(&mut seris_handles.handles) {
        let Some(seri) = assets.remove(&handle) else { continue };

        let str_id = match StrId::new(seri.id.clone()) {
            Ok(id) => id,
            Err(e) => {
                let err = BevyError::from(format!("Failed to create StrId for dimension {}: {}", seri.id, e));
                error!(target: "dimension_loading", "{}", err);
                continue;
            }
        };

        info!(target: "dimension_loading", "Spawning dimension '{}' with id '{}' ", seri.name, str_id);
        cmd.spawn((
            str_id,
            Transform::default(),
            DisplayName::new(seri.name),
            Dimension,
            Visibility::Visible,
        ));
    }
    Ok(())
}


pub fn add_dimensions_to_map(
    map: Option<ResMut<DimensionEntityMap>>,
    query: Query<(Entity, &EntityPrefix, &StrId), (Added<Dimension>, )>,
) -> Result {
    let mut result: Result = Ok(());
    if let Some(mut map) = map {
        for (ent, prefix, str_id) in query.iter() {
            if let Err(err) = map.0.insert(str_id, ent, ) {
                error!(target: "dimension_loading", "{} {} already in DimensionEntityMap : {}", prefix, str_id, err);
                result = Err(err);
            } else {
                info!(target: "dimension_loading", "Inserted tile '{}' into DimensionEntityMap with entity {:?}", str_id, ent);
            }
        }
    }
    result
}