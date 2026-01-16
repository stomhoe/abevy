use being::being_components::*;
use common::{common_components::StrId, common_states::AppState};
use faction::{faction_components::*, faction_resources::FactionEntityMap};
use modifier::{modifier_components::*, modifier_move_components::Speed};
use player::player_components::*;
use tilemap::{chunking_components::ActivatingChunks, chunking_resources::AaChunkRangeSettings};

use bevy::prelude::*;
use tilemap_shared::GlobalGenSettings;
use sprite_shared::SpriteConfigStrIds;


#[allow(unused_parens, )]
pub fn server_or_singleplayer_setup(mut cmd: Commands, 
    mut map: ResMut<FactionEntityMap>,
    mut settings: Query<&mut GlobalGenSettings>,
    mut app_state: ResMut<NextState<AppState>>,
) 
{
    info!("game");
    let Ok(mut settings) = settings.single_mut()
    else {
        error!(target: "game_init_systems", "Failed to get AaGlobalGenSettings");
        return;
    };
    

    settings.seed = 0;
    
    let host_faction_id = StrId::new_truncated("host");
    let host_faction = cmd.spawn((Faction, host_faction_id.clone(), OfSelf)).id();
    
    let Ok(_) = map.0.try_insert(host_faction_id, host_faction)
    else {
        error!(target: "game_init_systems", "Failed to insert host faction into FactionEntityMap: duplicate id");
        return;
    };
    
    cmd.spawn((
        OfSelf, HostPlayer,
        StrId::new_truncated("HOOOOOST"),
        BelongsToFaction(host_faction),
    ));
    app_state.set(AppState::StatefulGameSession);
}

#[allow(unused_parens)]
pub fn host_on_player_added(mut cmd: Commands, 
    query: Query<(Entity, &StrId),(Added<StrId>, With<Player>)>,
    player_query: Query<(&CreatedCharacters)>,

    host_faction: Single<(Entity), (With<Faction>, With<OfSelf>)>,
) -> Result {
    let host_faction = host_faction.into_inner();
    for (player_ent, username) in query.iter() {

        if player_query.get(player_ent).is_err() {

            //USAR EL DEFAULT ASE Q SE DESPAWNEE
            cmd.spawn((Being::default(), username.clone(), 
                DirControlledBy { client: player_ent }, 
                CharacterCreatedBy { player: player_ent },

                BelongsToFaction(host_faction.clone()),
                Transform::from_translation(Vec3::new(5900.0, 900.0, 0.0)),
                SpriteConfigStrIds::new(["humanhe0", "humanbo0"]),
                
            ));

        }else{
            //TODO ASIGNARLE SU CHARACTER SI TIENE EL MISMO OWNER
        }
    }
    Ok(())
}

#[allow(unused_parens, )]
pub fn put_player_beings_on_map(
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

