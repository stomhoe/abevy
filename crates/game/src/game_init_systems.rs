use being::being_components::*;
use being::being_inst_template::being_inst_template_resources::BitStrIdRef;
use ::being_shared::*;
use common::{GAME_INIT, common_components::StrId, common_states::AppState};
use faction::{faction_components::*, faction_resources::*};
use modifier::{modifier_components::*, modifier_move_bundles::SpeedModifier,};
use player::player_components::*;
use tilemap::{
    chunking::{chunking_components::ActivatingChunks, chunking_resources::AaChunkRangeSettings},
    terrain::{
        terrprobe::{terrprobe_components::TerrProbeTempl, terrprobe_resources::TerrProbeTemplEntityMap},
    },
};

use bevy::prelude::*;
use tilemap::{
    run_oneshot_suitable_pos_search_logic,
    terrain::{
        terrprobe::terrprobe_messages::TerrProbeJob,
        terrgen_search::SearchParams,
    },
};
use tilemap_shared::{Dimension, DimensionEntityMap, DimensionRef, GlobalGenSettings, GlobalTilePos};

#[derive(Debug, Event, Copy, Clone)]
pub struct CommonSpawnOriginFound {
    pub dim_ref: DimensionRef,
    pub pos: GlobalTilePos,
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
        error!(target: "game_init_systems", "Failed to get AaGlobalGenSettings");
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

    host_faction: Query<Entity, (With<Faction>, With<Mine>)>,
) {
    if query.is_empty() {
        return;
    }

    let Ok(host_faction) = host_faction.single() else {
        error!("Failed to get host faction");
        return;
    };
    for (player_ent, username) in query.iter() {

        if player_query.get(player_ent).is_err() {

            //USAR EL DEFAULT ASE Q SE DESPAWNEE

            let created_character = cmd.spawn((Being::default(), username.clone(),
                ControlledBy { client: player_ent },
                CharacterCreatedBy { player: player_ent },
                Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
                BitStrIdRef(StrId::trunc("pig")),
                BelongsToFaction(host_faction),
            )).id();
            cmd.spawn(SpeedModifier::new(created_character, created_character, 1000.0, ApplyMode::Add));

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
    mut settings: Query<&GlobalGenSettings>,
) {
    let Ok(settings) = settings.single()
    else {
        error!(target: "game_init_systems", "Failed to get AaGlobalGenSettings");
        return;
    };
    let make_search_request = |_cmd: &mut Commands| -> Option<TerrProbeJob> {
        let Ok(ow_dimension) = dimension_entity_map.0.get_cloned(Dimension::overworld()) else {
            warn!(target: GAME_INIT, "Overworld dimension '{}' not in DimensionEntityMap yet", Dimension::overworld());
            return None;
        };

        let Ok(probe_template_ent) = terrprobe_entity_map.0.get_cloned(settings.spawn_tag.clone()) else {
            warn!(target: GAME_INIT, "TerrainProbe template {} not in TerrainProbeTemplateEntityMap yet", settings.spawn_tag.clone());
            return None;
        };
        let Ok(probe_template) = terrprobe_query.get(probe_template_ent) else {
            warn!(target: GAME_INIT, "TerrainProbe template entity {:?} missing TerrProbeTempl", probe_template_ent);
            return None;
        };
        Some(probe_template.to_probe(probe_template_ent, DimensionRef(ow_dimension), GlobalTilePos::default()))
    };
    let handle_success = |cmd: &mut Commands,
                              found_pos: GlobalTilePos,
                              requester: Entity,
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

    let handle_failure = |_cmd: &mut Commands, failed_filter_ent: Entity| {
        warn!(target: GAME_INIT, "Common spawn search failed for filter {:?}", failed_filter_ent);
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
    chunk_range: Res<AaChunkRangeSettings>,
    mut next_spawn_offset_x: Local<i32>,
) {
    let found = trigger.event();
    let spawn_dim = found.dim_ref;
    let origin = found.pos;

    for (created_characters, ) in players.iter() {
        debug!(target: GAME_INIT, "Spawning player being: {:?}", created_characters);

        for &created_character in created_characters.entities() {
            let spawn_pos = origin + GlobalTilePos::new(*next_spawn_offset_x, 0);
            *next_spawn_offset_x += 1;
            let world_pos: Vec2 = spawn_pos.into();

            cmd.entity(created_character).try_insert((
                Transform::from_translation(world_pos.extend(0.0)),
                DimensionRef(spawn_dim.0),
                ActivatingChunks::new(&chunk_range),
            ));
        }
    }

}
