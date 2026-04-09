use bevy::prelude::*;
use bevy_replicon::prelude::*;

pub use crate::terrain::terrprobe::terrprobe_seris::*;

use common::common_components::*;
use common::common_types::HashIdToEntityMap;
use common::log_targets::ENTITY_MAP_SYSTEM;

use crate::terrain::terrprobe::terrprobe_components::TerrProbeTempl;
use crate::terrain::terrprobe::terrprobe_tpt_parser::{
    load_terrain_probe_defs_from_filesystem, LoadedTerrainProbeDef,
};

#[derive(Component, Debug, Default, serde::Deserialize, serde::Serialize, Clone)]
#[require(
    AssetScoped,
    Prefix::trunc("EguiTerrProbeTemplHolder"),
    Replicated,
    Visibility::Hidden,
    Transform,
)]
pub struct EguiTptsHolder;

#[derive(Resource, Debug, Clone)]
pub struct TerrProbeTemplEntityMap(pub HashIdToEntityMap);

impl Default for TerrProbeTemplEntityMap {
    fn default() -> Self {
        Self(Default::default())
    }
}

pub fn load_terrain_probe_seri_defs() -> Vec<LoadedTerrainProbeDef> {
    load_terrain_probe_defs_from_filesystem()
}

#[allow(unused_parens, )]
pub fn map_terr_probe_templ_id_to_entity(
    mut cmd: Commands,
    map: Option<ResMut<TerrProbeTemplEntityMap>>,
    client_state: Res<State<ClientState>>,
    query: Query<(Entity, &StrId, Has<RemoveReplicatedAfterClone>, ), (Changed<StrId>, With<TerrProbeTempl>, ),>,
) {
    let am_i_client = *client_state.get() == ClientState::Connected;
    let Some(mut map) = map else {
        error!(
            target: ENTITY_MAP_SYSTEM,
            "TerrProbeTemplEntityMap resource not found when trying to add terrain probes to it."
        );
        return;
    };

    for (entity, id, remove_after_clone) in query.iter() {
        if am_i_client && remove_after_clone {
            continue;
        }
        if let Err(prev_ent) = map.0.insert(id, entity) {
            if prev_ent.0 == entity {
                continue;
            }
            error!(
                target: ENTITY_MAP_SYSTEM,
                "Terrain probe '{}' already in TerrProbeTemplEntityMap with entity {:?}, cannot insert entity {:?}",
                id,
                prev_ent,
                entity
            );
            cmd.entity(entity).try_despawn();
        } else {
            trace!(
                target: ENTITY_MAP_SYSTEM,
                "Inserted terrain probe '{}' into TerrProbeTemplEntityMap with entity {:?}",
                id,
                entity
            );
        }
    }
}

#[allow(unused_parens, )]
pub fn remove_terr_probe_templ_from_entity_map_on_despawn(
    trigger: On<Despawn, TerrProbeTempl>,
    query: Query<(&StrId, ), (With<TerrProbeTempl>, ),>,
    mut map: ResMut<TerrProbeTemplEntityMap>,
) {
    let Ok((id,)) = query.get(trigger.entity) else {
        return;
    };
    if let Ok(found_entity) = map.0.get_cloned(id) {
        if found_entity == trigger.entity {
            map.0.remove(id.as_str());
        }
    }
}

pub fn plugin_terr_probe_templ(app: &mut App) {
    use bevy_replicon::prelude::AppRuleExt;

    app
        .replicate::<TerrProbeTempl>()
        .replicate::<EguiTptsHolder>()
        .init_resource::<TerrProbeTemplEntityMap>()
        .add_systems(Update, map_terr_probe_templ_id_to_entity)
        .add_observer(remove_terr_probe_templ_from_entity_map_on_despawn)
        ;
}
