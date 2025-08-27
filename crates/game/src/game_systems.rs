use being::being_components::CreatedCharacters;
use common::{common_components::{ StrId}, common_states::AssetsLoadingState};
use faction::faction_components::*;
use modifier::{modifier_components::*, modifier_move_components::Speed};
use player::player_components::*;
use tilemap::chunking_components::ActivatingChunks;

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
        StrId::new("HOOOOOST", 0)?,
        BelongsToFaction(faction.into_inner()),
    ));
    Ok(())
}

#[allow(unused_parens, )]
pub fn spawn_player_beings(
    mut cmd: Commands,
    players: Query<(Entity, &CreatedCharacters, Option<&OfSelf>), (With<Player>)>,
) {
    for (player_ent, created_characters, self_player) in players.iter() {
        println!("Spawning player being: {:?}", created_characters);

        for &created_character in created_characters.entities() {
            cmd.entity(created_character).insert((
                //TargetSpawnPos::new(0.0, 0.0),
                ActivatingChunks::default(),
            ));
            cmd.spawn((ModifierTarget(created_character), ChildOf(created_character), Speed, EffectiveValue(40000.0)));
        }

        if self_player.is_some() {
            debug!(target: "game", "Spawning self player being:");

        } 
    }

}

