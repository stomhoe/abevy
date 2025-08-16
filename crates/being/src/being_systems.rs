


use bevy::{prelude::*};
use camera::camera_components::CameraTarget;
use faction::faction_components::*;
use player::player_components::*;
use tilemap::chunking_components::ActivatingChunks;

use crate::{being_components::*,};




#[allow(unused_parens)]
// A L CENTRO DE LA BASE VA A HABER Q PONERLE UNO DE ALGUNA FORMA
pub fn host_add_activates_chunks(mut cmd: Commands, 
    query: Query<(Entity),(With<Being>, Added<BelongsToAPlayerFaction>)>,
    mut removed: RemovedComponents<BelongsToAPlayerFaction>,
) {
    
    for ent in query.iter() {
        cmd.entity(ent).try_insert(ActivatingChunks::default());
        debug!("Adding ActivatingChunks to entity {:?}", ent);    
    }
    for ent in removed.read() {
        cmd.entity(ent).try_remove::<ActivatingChunks>();
    }
}



#[allow(unused_parens)]
pub fn on_control_change(
    mut commands: Commands, 
    self_player: Query<(Entity, Has<HostPlayer>), (With<Player>, With<OfSelf>)>,

    query: Query<(Entity, &DirControlledBy, &HumanControlled, Has<CameraTarget>),(Or<(Changed<DirControlledBy>, Changed<HumanControlled>)>)>,
    mut removed: RemovedComponents<DirControlledBy>,
) {
    for ent in removed.read() {
        commands.entity(ent).remove::<ControlledLocally>();
    }
    let (self_entity, is_host) = self_player.single().unwrap();
    for (ent, controlled_by, human_controlled, is_camera_target) in query.iter() {
        if controlled_by.client == self_entity {
            info!("debug {:?} is now controlled locally by self", ent);
            commands.entity(ent).try_insert((ControlledLocally::default(), ActivatingChunks::default()));
            if human_controlled.0 {//PROVISORIO
                debug!("Entity {:?} is now a CameraTarget", ent);
                commands.entity(ent).try_insert(CameraTarget);
            } else {
                debug!("Entity {:?} is no longer a CameraTarget", ent);
                commands.entity(ent).try_remove::<CameraTarget>();
            }//PROVISORIO
        } else {
            commands.entity(ent).try_remove::<ControlledLocally>();
            if !is_camera_target && !is_host {
                commands.entity(ent).try_remove::<ActivatingChunks>();
            }
        }
    }

}