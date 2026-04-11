
#[allow(unused_imports, )]use being::being_bundles::{BeingBundle, };
use ::being_shared::*;
use common::{GAME_INIT, common_components::StrId, common_states::AppState};
use common::common_components::HashId;
use faction::{faction_resources::*};
use ::being_shared::JoinedGroups;
use ::being_shared::movement_shared_components::{GridLockedMovement, GridLockedMovementVisual};
use faction_shared::Faction;
use game_common::game_common_components::Templ;
use player_shared::player_components::*;
use tilemap::{
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

#[derive(Resource, Debug, Default, Copy, Clone)]
pub struct CommonSpawnOriginCache(pub Option<CommonSpawnOriginFound>);

#[derive(Resource, Clone, Debug)]
pub struct GameInitSettings {
    pub players_spawn_probe_id: StrId,
    pub players_initial_bit: StrId,
}
impl Default for GameInitSettings {
    fn default() -> Self {
        Self {
            players_spawn_probe_id: StrId::new_with_result("land", 0).expect("default players_spawn_probe_id must be a valid StrId"),
            players_initial_bit: StrId::new_with_result("player_warrior", 0).expect("default players_initial_bit must be a valid StrId"),
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
        let players_spawn_probe_id = match StrId::new_with_result(self.players_spawn_probe_id.trim(), 0) {
            Ok(str_id) => str_id,
            Err(err) => {
                error!(
                    target: GAME_INIT,
                    "Invalid players_spawn_probe_id '{}' in GameInitSettingsSeri '{}': {}. Falling back to 'land'",
                    self.players_spawn_probe_id,
                    self.id,
                    err,
                );
                StrId::new_with_result("land", 0).expect("fallback players_spawn_probe_id must be a valid StrId")
            }
        };
        let players_initial_bit = match StrId::new_with_result(self.players_initial_bit_ref_strid.trim(), 0) {
            Ok(str_id) => str_id,
            Err(err) => {
                error!(
                    target: GAME_INIT,
                    "Invalid players_initial_bit_ref_strid '{}' in GameInitSettingsSeri '{}': {}. Falling back to 'player_warrior'",
                    self.players_initial_bit_ref_strid,
                    self.id,
                    err,
                );
                StrId::new_with_result("player_warrior", 0).expect("fallback players_initial_bit must be a valid StrId")
            }
        };
        GameInitSettings {
            players_spawn_probe_id,
            players_initial_bit,
        }
    }
}
fn default_players_spawn_probe_id() -> String { "land".to_string() }
fn default_players_initial_bit_ref_strid() -> String { "player_warrior".to_string() }

fn place_player_being_at(
    cmd: &mut Commands,
    being_ent: Entity,
    spawn_dim: DimensionRef,
    gpos: GlobalTilePos,
    settings: &GameInitSettings,
) {
    cmd.entity(being_ent).try_insert((
        Transform::from_translation(gpos.to_translation(0.0)),
        gpos,
        GridLockedMovement::default(),
        GridLockedMovementVisual {
            visual_origin_tile: gpos.0,
            ..default()
        },
        DimensionRef(spawn_dim.0),
        BitStrIdRef::new(settings.players_initial_bit.as_str()),
        Being,
    ));
}

pub fn load_game_init_settings(
    mut settings: ResMut<GameInitSettings>,
    terrprobe_entity_map: Res<TerrProbeTemplEntityMap>,
) {
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
    let mut loaded_settings = first.value.to_settings();
    let requested_probe_id = loaded_settings.players_spawn_probe_id.clone();
    if terrprobe_entity_map.0.get_cloned(requested_probe_id.clone()).is_err() {
        error!(
            target: GAME_INIT,
            "Configured players_spawn_probe_id '{}' in GameInitSettingsSeri '{}' does not match any loaded terrain probe. Falling back to 'land'",
            requested_probe_id,
            first.value.id,
        );
        let fallback_probe_id = StrId::new_with_result("land", 0)
            .expect("fallback players_spawn_probe_id must be a valid StrId");
        if terrprobe_entity_map.0.get_cloned(fallback_probe_id.clone()).is_err() {
            error!(
                target: GAME_INIT,
                "Fallback terrain probe 'land' is also missing after loading GameInitSettingsSeri '{}'; keeping default GameInitSettings",
                first.value.id,
            );
            return;
        }
        loaded_settings.players_spawn_probe_id = fallback_probe_id;
    }
    *settings = loaded_settings;
}


#[allow(unused_parens, )]
pub fn server_or_singleplayer_setup(mut cmd: Commands,
    mut map: ResMut<FactionEntityMap>,
    mut settings: Query<&mut GlobalGenSettings>,
    mut app_state: ResMut<NextState<AppState>>,
)
{
    let Ok(mut _____settings) = settings.single_mut()
    else {
        error!(target: GAME_INIT, "Failed to get AaGlobalGenSettings");
        return;
    };



    let host_faction_id = match StrId::new_with_result("host", 0) {
        Ok(host_faction_id) => host_faction_id,
        Err(err) => {
            error!(target: GAME_INIT, "Invalid hardcoded host faction id 'host': {}", err);
            return;
        }
    };
    let host_faction_hash = HashId::from(host_faction_id.as_str());
    let host_faction = cmd.spawn((Faction, host_faction_id.clone(), host_faction_hash, Mine)).id();

    map.0.overwrite(host_faction_id, host_faction);


    let host_player_id = match StrId::new_with_result("HOOOOOST", 0) {
        Ok(host_player_id) => host_player_id,
        Err(err) => {
            error!(target: GAME_INIT, "Invalid hardcoded host player id 'HOOOOOST': {}", err);
            return;
        }
    };
    cmd.spawn((
        Mine, HostPlayer,
        host_player_id,
        FactionRef(host_faction_hash),
    ));
    app_state.set(AppState::StatefulGameSession);
}

#[allow(unused_parens)]
pub fn host_on_player_added(mut cmd: Commands,
    query: Query<(Entity, &StrId),(Added<StrId>, With<Player>)>,
    player_query: Query<(&CreatedCharacters)>,
    ________settings: Res<GameInitSettings>,
    host_player_faction_ref: Query<&FactionRef, (With<HostPlayer>, With<Mine>, )>,
    host_faction_query: Query<Entity, (With<Faction>, With<Mine>, Without<Templ>, )>,
    faction_map: Res<FactionEntityMap>,
) {
    if query.is_empty() {
        return;
    }

    let host_faction = host_player_faction_ref
        .single()
        .ok()
        .and_then(|host_faction_ref| faction_map.0.get_cloned(host_faction_ref.0).ok())
        .or_else(|| host_faction_query.iter().next());
    let Some(host_faction) = host_faction else {
        error!(target: GAME_INIT, "Failed to get host faction: no HostPlayer FactionRef could be resolved and no fallback Faction+Mine entity was found");
        return;
    };
    for (player_ent, username) in query.iter() {

        if player_query.get(player_ent).is_err() {

            //USAR EL DEFAULT ASE Q SE DESPAWNEE

            let _created_character = cmd.spawn((Being, username.clone(),
                CharacterCreatedBy { player: player_ent },
                JoinedGroups::single(host_faction),
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
    dimension_hash_query: Query<&HashId, With<Dimension>>,
    terrprobe_entity_map: Res<TerrProbeTemplEntityMap>,
    terrprobe_query: Query<&TerrProbeTempl>,
    mut search_params: SearchParams,
    mut active_probe_ent: Local<Option<Entity>>,
    mut search_finished: Local<bool>,
    settings: Res<GameInitSettings>,
) {
    let make_search_request = |_cmd: &mut Commands| -> Option<TerrProbeJob> {
    let Ok(ow_dimension) = dimension_entity_map.0.get_cloned(Dimension::overworld()) else {
            return None;
        };
        let Ok(&ow_dimension_hash) = dimension_hash_query.get(ow_dimension) else {
            return None;
        };

        let Ok(probe_template_ent) = terrprobe_entity_map.0.get_cloned(settings.players_spawn_probe_id.clone()) else {
            error_once!(
                target: GAME_INIT,
                "Failed to find TerrProbe template for players_spawn_probe_id '{}'; no common player spawn origin can be found",
                settings.players_spawn_probe_id.as_str(),
            );
            return None;
        };
        let Ok(probe_template) = terrprobe_query.get(probe_template_ent) else {
            return None;
        };
        Some(probe_template.to_probe(probe_template_ent, DimensionRef(ow_dimension_hash), GlobalTilePos::default()))
    };
    let handle_success = |cmd: &mut Commands,
                              found_pos: GlobalTilePos,
                              _requester: Entity,
                              _sampled_val: f32|
     -> bool {
        let Ok(ow_dimension) = dimension_entity_map.0.get_cloned(Dimension::overworld()) else {
            return false;
        };
        let Ok(&ow_dimension_hash) = dimension_hash_query.get(ow_dimension) else {
            return false;
        };
        cmd.trigger(CommonSpawnOriginFound {
            dim_ref: DimensionRef(ow_dimension_hash),
            pos: found_pos,
        });
        true
    };

    let handle_failure = |_cmd: &mut Commands, _failed_probe_ent: Entity| {
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
    beings_to_place: Query<(Entity, &CharacterCreatedBy, ), (With<Being>, Without<DimensionRef>)>,
    settings: Res<GameInitSettings>,
    mut next_spawn_offset_x: Local<i32>,
    mut spawn_cache: ResMut<CommonSpawnOriginCache>,
) {
    let found = trigger.event();
    spawn_cache.0 = Some(*found);
    let spawn_dim = found.dim_ref;
    let origin = found.pos;

    let compute_spawn_pos = |origin: GlobalTilePos, next_x: &mut i32| -> GlobalTilePos {
        let spawn_pos = origin + GlobalTilePos::new(*next_x, 0);
        *next_x += 1;
        spawn_pos
    };

    for (being_ent, _, ) in beings_to_place.iter() {
        let gpos = compute_spawn_pos(origin, &mut *next_spawn_offset_x);
        place_player_being_at(&mut cmd, being_ent, spawn_dim, gpos, &settings);
    }
}

#[allow(unused_parens, )]
pub fn place_unpositioned_player_beings_with_cached_origin(
    mut cmd: Commands,
    beings_to_place: Query<(Entity, &CharacterCreatedBy, ), (With<Being>, Without<DimensionRef>)>,
    settings: Res<GameInitSettings>,
    spawn_cache: Res<CommonSpawnOriginCache>,
    mut next_spawn_offset_x: Local<i32>,
) {
    let Some(found) = spawn_cache.0
    else {
        return;
    };
    let compute_spawn_pos = |origin: GlobalTilePos, next_x: &mut i32| -> GlobalTilePos {
        let spawn_pos = origin + GlobalTilePos::new(*next_x, 0);
        *next_x += 1;
        spawn_pos
    };
    for (being_ent, _, ) in beings_to_place.iter() {
        let gpos = compute_spawn_pos(found.pos, &mut *next_spawn_offset_x);
        place_player_being_at(&mut cmd, being_ent, found.dim_ref, gpos, &settings);
    }
}
