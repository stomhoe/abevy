use common::{components::DisplayName, states::ReplicatedAssetsLoadingState};
use sprite_shared::sprite_shared::SpriteConfigStringIds;

use crate::{being_components::{ControlledBy, PlayerDirectControllable, TargetSpawnPos}, faction_components::*, player::*};
use bevy::prelude::*;



pub fn server_or_singleplayer_setup(mut cmd: Commands, 
    mut assets_loading_state: ResMut<NextState<ReplicatedAssetsLoadingState>>,
    mut fac_map: ResMut<FactionEntityMap>)
{
    assets_loading_state.set(ReplicatedAssetsLoadingState::InProcess);

    let fac_ent = Faction::new(&mut cmd, &mut fac_map, "host", "Host Faction", ());
    cmd.spawn((
        OfSelf, HostPlayer,
        DisplayName::new("HOOOOOST"),
        BelongsToFaction(fac_ent),
    ));
}

#[allow(unused_parens, )]
pub fn spawn_player_beings(
    mut commands: Commands,
    players: Query<(Entity, &CreatedCharacter, &BelongsToFaction, Option<&OfSelf>), (With<Player>)>,
) {
    for (player_ent, created_character, belongs_to_fac, self_player) in players.iter() {
        println!("Spawning player being: {:?}", created_character);

        commands.entity(created_character.0).insert((
            ControlledBy { client: player_ent },
            PlayerDirectControllable,
            TargetSpawnPos::new(0.0, 0.0),
            Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),//PROVISORIO
            //HACER Q OTRO SYSTEMA AGREGUE CAMERATARGET AL BEING CONTROLADO
            SpriteConfigStringIds::new(["humanhe0", "humanbo0"]),
            belongs_to_fac.clone(),
        ));

        if self_player.is_some() {
            debug!(target: "game", "Spawning self player being:");

        } 
        commands.entity(player_ent).remove::<CreatedCharacter>();
    }

}