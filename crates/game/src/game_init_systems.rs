use being::being_components::*;
use being::being_bundles::{BeingBundle, };
use being::being_inst_template::being_inst_template_resources::BitStrIdRef;
use ::being_shared::*;
use common::{GAME_INIT, common_components::StrId, common_states::AppState};
use faction::{faction_components::*, faction_resources::*};
use modifier_shared::{modifier_components::*, modifier_move_bundles::SpeedModifier,};
use movement::movement_components::GridLockedMovement;
use player::player_components::*;
use tilemap::{
    chunking::{chunking_components::ActivatingChunks, chunking_resources::AaChunkRangeSettings},
    terrain::{
        terrprobe::{terrprobe_components::TerrProbeTempl, terrprobe_resources::TerrProbeTemplEntityMap},
    },
};

use bevy::prelude::*;
use serde::Deserialize;
use tilemap::{
    run_oneshot_suitable_pos_search_logic,
    terrain::{
        terrprobe::terrprobe_messages::TerrProbeJob,
        terrprobe::terrprobe_systems::SearchParams,
    },
};
use tilemap_shared::{Dimension, DimensionEntityMap, DimensionRef, GlobalGenSettings, GlobalTilePos};

#[derive(Debug, Event, Copy, Clone)]
pub struct CommonSpawnOriginFound {
    pub dim_ref: DimensionRef,
    pub pos: GlobalTilePos,
}

#[derive(Resource, Clone, Debug)]
pub struct GameInitSettings {
    pub players_spawn_probe_id: StrId,
    pub players_initial_bit_ref_strid: StrId,
}
impl Default for GameInitSettings {
    fn default() -> Self {
        Self {
            players_spawn_probe_id: StrId::trunc("coland"),
            players_initial_bit_ref_strid: StrId::trunc("player_warrior"),
        }
    }
}

#[derive(Deserialize, Asset, TypePath, Clone, Debug)]
pub struct GameInitSettingsSeri {
    pub id: String,
    #[serde(default = "default_players_spawn_probe_id")]
    pub players_spawn_probe_id: String,
    #[serde(default = "default_players_initial_bit_ref_strid")]
    pub players_initial_bit_ref_strid: String,
}
impl GameInitSettingsSeri {
    pub fn to_settings(&self) -> GameInitSettings {
        GameInitSettings {
            players_spawn_probe_id: StrId::trunc(self.players_spawn_probe_id.trim()),
            players_initial_bit_ref_strid: StrId::trunc(self.players_initial_bit_ref_strid.trim()),
        }
    }
}
fn default_players_spawn_probe_id() -> String { "coland".to_string() }
fn default_players_initial_bit_ref_strid() -> String { "player_warrior".to_string() }

pub fn load_game_init_settings(mut settings: ResMut<GameInitSettings>) {
    let db = match common::def_db::DefDatabase::<GameInitSettingsSeri>::load_from_assets_dir_with_type(
        stringify!(GameInitSettingsSeri),
        &["game_init.settings.ron"],
        |_| "game_init_settings",
    ) {
        Ok(db) => db,
        Err(err) => {
            error!(target: GAME_INIT, "Failed loading GameInitSettingsSeri defs: {err:#}");
            return;
        }
    };
    let Some(first) = db.into_records().into_iter().next() else {
        return;
    };
    *settings = first.value.to_settings();
}


#[allow(unused_parens, )]
pub fn server_or_singleplayer_setup(mut cmd: Commands,
    mut map: ResMut<FactionEntityMap>,
    mut settings: Query<&mut GlobalGenSettings>,
    mut app_state: ResMut<NextState<AppState>>,
)
{
    let Ok(mut settings) = settings.single_mut()
    else {
        error!(target: GAME_INIT, "Failed to get AaGlobalGenSettings");
        return;
    };



    let host_faction_id = StrId::trunc("host");
    let host_faction = cmd.spawn((Faction, host_faction_id.clone(), Mine)).id();

    map.0.overwrite(host_faction_id, host_faction);


    cmd.spawn((
        Mine, HostPlayer,
        StrId::trunc("HOOOOOST"),
        BelongsToFaction(host_faction),
    ));
    app_state.set(AppState::StatefulGameSession);
}

#[allow(unused_parens)]
pub fn host_on_player_added(mut cmd: Commands,
    query: Query<(Entity, &StrId),(Added<StrId>, With<Player>)>,
    player_query: Query<(&CreatedCharacters)>,
    settings: Res<GameInitSettings>,

    host_faction: Query<Entity, (With<Faction>, With<Mine>)>,
) {
    if query.is_empty() {
        return;
    }

    let Ok(host_faction) = host_faction.single() else {
        error!(target: GAME_INIT, "Failed to get host faction");
        return;
    };
    for (player_ent, username) in query.iter() {

        if player_query.get(player_ent).is_err() {

            //USAR EL DEFAULT ASE Q SE DESPAWNEE

            let created_character = cmd.spawn((Being, username.clone(),
                CharacterCreatedBy { player: player_ent },
                BelongsToFaction(host_faction),
                ComputedBy {
                    client_ent: player_ent,
                    human_dc_input: true,
                },
            )).id();
            //cmd.spawn(SpeedModifier::new(created_character, created_character, 1000.0, ApplyMode::Add));

        }else{
            //TODO ASIGNARLE SU CHARACTER SI TIENE EL MISMO OWNER
        }
    }
}

#[allow(unused_parens, )]
pub fn find_common_player_spawn_origin(
    mut cmd: Commands,
    dimension_entity_map: Res<DimensionEntityMap>,
    terrprobe_entity_map: Res<TerrProbeTemplEntityMap>,
    terrprobe_query: Query<&TerrProbeTempl>,
    mut search_params: SearchParams,
    mut active_probe_ent: Local<Option<Entity>>,
    mut search_finished: Local<bool>,
    settings: Res<GameInitSettings>,
) {
    let make_search_request = |_cmd: &mut Commands| -> Option<TerrProbeJob> {
        let Ok(ow_dimension) = dimension_entity_map.0.get_cloned(Dimension::overworld()) else {
            error_once!(target: GAME_INIT, "Overworld dimension '{}' not in DimensionEntityMap", Dimension::overworld());
            return None;
        };

        let Ok(probe_template_ent) = terrprobe_entity_map.0.get_cloned(settings.players_spawn_probe_id.clone()) else {
            error_once!(target: GAME_INIT, "TerrainProbe template {} not in TerrainProbeTemplateEntityMap", settings.players_spawn_probe_id.clone());
            return None;
        };
        let Ok(probe_template) = terrprobe_query.get(probe_template_ent) else {
            error_once!(target: GAME_INIT, "TerrainProbe template entity {:?} missing TerrProbeTempl", probe_template_ent);
            return None;
        };
        Some(probe_template.to_probe(probe_template_ent, DimensionRef(ow_dimension), GlobalTilePos::default()))
    };
    let handle_success = |cmd: &mut Commands,
                              found_pos: GlobalTilePos,
                              _requester: Entity,
                              _sampled_val: f32|
     -> bool {
        let Ok(ow_dimension) = dimension_entity_map.0.get_cloned(Dimension::overworld()) else {
            return false;
        };
        info!(target: GAME_INIT, "Found shared spawn origin at {:?} in dimension {:?}", found_pos, ow_dimension);
        cmd.trigger(CommonSpawnOriginFound {
            dim_ref: DimensionRef(ow_dimension),
            pos: found_pos,
        });
        true
    };

    let handle_failure = |_cmd: &mut Commands, failed_probe_ent: Entity| {
        error_once!(target: GAME_INIT, "Common spawn search failed for probe {:?}", failed_probe_ent);
    };

    run_oneshot_suitable_pos_search_logic!(
        target: GAME_INIT,
        searched_label: "common character spawn origin",
        cmd: cmd,
        search_params: search_params,
        active_probe_ent: active_probe_ent,
        search_finished: search_finished,
        make_search_request: make_search_request,
        handle_success: handle_success,
        handle_failure: handle_failure,
    );
}

#[allow(unused_parens, )]
pub fn put_player_beings_on_map(
    trigger: On<CommonSpawnOriginFound>,
    mut cmd: Commands,
    players: Query<(&CreatedCharacters, ), (With<Player>)>,
    created_by_query: Query<&CharacterCreatedBy>,
    chunk_range: Res<AaChunkRangeSettings>,
    settings: Res<GameInitSettings>,
    mut next_spawn_offset_x: Local<i32>,
) {
    let found = trigger.event();
    let spawn_dim = found.dim_ref;
    let origin = found.pos;

    let compute_spawn_pos = |origin: GlobalTilePos, next_x: &mut i32| -> GlobalTilePos {
        let spawn_pos = origin + GlobalTilePos::new(*next_x, 0);
        *next_x += 1;
        spawn_pos
    };

    for (created_characters, ) in players.iter() {
        debug!(target: GAME_INIT, "Spawning player being: {:?}", created_characters);

        for &being_ent in created_characters.entities() {
            let gpos = compute_spawn_pos(origin, &mut *next_spawn_offset_x);
            let Ok(created_by) = created_by_query.get(being_ent) else { continue; };

            cmd.entity(being_ent)
                .try_remove::<Transform>()
                .try_remove::<GlobalTilePos>()
                .try_remove::<DimensionRef>()
                .try_insert((
                Transform::from_translation(gpos.to_translation(0.0)),
                gpos,
                GridLockedMovement {
                    visual_origin_tile: gpos.0,
                    ..default()
                },
                DimensionRef(spawn_dim.0),
                ActivatingChunks::new(&chunk_range),

                BitStrIdRef::new(settings.players_initial_bit_ref_strid.as_str()),
                Being,

            ));
        }
    }
    let gpos = compute_spawn_pos(origin, &mut *next_spawn_offset_x);
    return;
    let bear_ent = cmd.spawn((
        BeingBundle::new(DimensionRef(spawn_dim.0), gpos),
        BitStrIdRef::new("pobear"),
        Predator::default(),
    )).id();
}
