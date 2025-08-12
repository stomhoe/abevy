use common::{common_components::{ StrId}, common_states::AssetsLoadingState};
use tilemap::chunking_components::ActivatingChunks;

use crate::{being_components::TargetSpawnPos, faction_components::*, player::*};
use bevy::prelude::*;


#[allow(unused_parens, )]
pub fn server_or_singleplayer_setup(mut cmd: Commands, 
    curr_assets_loading_state: Res<State<AssetsLoadingState>>,
    mut assets_loading_state: ResMut<NextState<AssetsLoadingState>>,
    faction: Single<(Entity), (With<Faction>, With<OfSelf>)>,
) -> Result
{
    assets_loading_state.set(AssetsLoadingState::ReplicatedInProcess);
    cmd.spawn((
        OfSelf, HostPlayer,
        StrId::new("HOOOOOST")?,
        BelongsToFaction(faction.into_inner()),
    ));
    Ok(())
}

#[allow(unused_parens, )]
pub fn spawn_player_beings(
    mut commands: Commands,
    players: Query<(Entity, &CreatedCharacters, &BelongsToFaction, Option<&OfSelf>), (With<Player>)>,
) {
    for (player_ent, created_characters, belongs_to_fac, self_player) in players.iter() {
        println!("Spawning player being: {:?}", created_characters);

        for created_character in created_characters.entities() {
            commands.entity(*created_character).insert((
                TargetSpawnPos::new(0.0, 0.0),
                ActivatingChunks::default(),
            ));
            //HACER Q OTRO SYSTEMA AGREGUE CAMERATARGET AL BEING CONTROLADO
        }

        if self_player.is_some() {
            debug!(target: "game", "Spawning self player being:");

        } 
    }

}

