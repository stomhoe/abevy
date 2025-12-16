use being::being_components::CreatedCharacters;
use common::{common_components::StrId, common_states::{AppState, AssetsLoadingState}};
use faction::{faction_components::*, faction_resources::FactionEntityMap};
use modifier::{modifier_components::*, modifier_move_components::Speed};
use player::player_components::*;
use tilemap::{chunking_components::ActivatingChunks, chunking_resources::AaChunkRangeSettings};

use bevy::prelude::*;
use tilemap_shared::AaGlobalGenSettings;


#[allow(unused_parens, )]
pub fn server_or_singleplayer_setup(mut cmd: Commands, 
    mut map: ResMut<FactionEntityMap>,
    mut settings: ResMut<AaGlobalGenSettings>,
    mut app_state: ResMut<NextState<AppState>>,
) -> Result
{
    settings.seed = 123;
    
    let host_faction_id = StrId::new_truncated("host");
    let host_faction = cmd.spawn((Faction, host_faction_id.clone(), OfSelf)).id();
    
    let Ok(_) = map.0.insert(host_faction_id, host_faction)
    else {
        let err = BevyError::from("Failed to insert host faction into FactionEntityMap: duplicate id");
        return Err(err);
    };
    
    cmd.spawn((
        OfSelf, HostPlayer,
        StrId::new_truncated("HOOOOOST"),
        BelongsToFaction(host_faction),
    ));
    app_state.set(AppState::StatefulGameSession);
    Ok(())
}

#[allow(unused_parens, )]
pub fn spawn_player_beings(
    mut cmd: Commands,
    players: Query<(Entity, &CreatedCharacters, Option<&OfSelf>), (With<Player>)>,
    chunk_range: Res<AaChunkRangeSettings>,
) {
    for (player_ent, created_characters, self_player) in players.iter() {
        println!("Spawning player being: {:?}", created_characters);

        for &created_character in created_characters.entities() {
            cmd.entity(created_character).insert((
                //TargetSpawnPos::new(0.0, 0.0),
                ActivatingChunks::new(&chunk_range),
            ));
            cmd.spawn((ModifierTarget(created_character), ChildOf(created_character), Speed, EffectiveValue(500.0)));
        }

        if self_player.is_some() {
            debug!(target: "game", "Spawning self player being:");

        } 
    }

}

