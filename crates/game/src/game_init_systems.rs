use being::being_components::*;
use being::being_inst_template::being_inst_template_resources::BitStrIdRef;
use common::{common_components::StrId, common_states::AppState};
use faction::{faction_components::*, faction_resources::*};
use modifier::{modifier_components::*, modifier_move_bundles::SpeedModifier, modifier_types::WalkSpeed };
use player::player_components::*;
use tilemap::{chunking::chunking_components::ActivatingChunks, chunking::chunking_resources::AaChunkRangeSettings};

use bevy::prelude::*;
use tilemap_shared::GlobalGenSettings;


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
                BitStrIdRef(StrId::trunc("bit_demo")),
                BelongsToFaction(host_faction),
            )).id();
            cmd.spawn(SpeedModifier::new(created_character, created_character, 1000.0, ApplyMode::Add));

        }else{
            //TODO ASIGNARLE SU CHARACTER SI TIENE EL MISMO OWNER
        }
    }
}

#[allow(unused_parens, )]
pub fn put_player_beings_on_map(
    mut cmd: Commands,
    players: Query<(Entity, &CreatedCharacters, Has<Mine>), (With<Player>)>,
    chunk_range: Res<AaChunkRangeSettings>,
) {
    for (player_ent, created_characters, self_player) in players.iter() {
        debug!(target: "game_init", "Spawning player being: {:?}", created_characters);

        for &created_character in created_characters.entities() {
            cmd.entity(created_character).try_insert_if_new((
                ActivatingChunks::new(&chunk_range),
            ));
        }

        if self_player {
            debug!(target: "game_init", "Spawning self player being:");

        }
    }

}
