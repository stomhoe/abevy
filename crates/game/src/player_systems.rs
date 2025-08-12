

use bevy::{prelude::*};
use camera::camera_components::CameraTarget;

use crate::{being_components::{ControlledBy, ControlledLocally, HumanControlled}, player::{OfSelf, Player}};









#[allow(unused_parens)]
pub fn on_control_change(
    mut commands: Commands, 
    self_player: Single<Entity,(With<Player>, With<OfSelf>)>,
    query: Query<(Entity, &ControlledBy, &HumanControlled),(Or<(Changed<ControlledBy>, Changed<HumanControlled>)>)>,
) {
    for (ent, controlled_by, human_controlled) in query.iter() {
        if controlled_by.client == *self_player {
            info!("debug {:?} is now controlled locally by self", ent);
            commands.entity(ent).insert((ControlledLocally, ));
            if human_controlled.0 {
                debug!("Entity {:?} is now a CameraTarget", ent);
                commands.entity(ent).insert(CameraTarget);
            } else {
                debug!("Entity {:?} is no longer a CameraTarget", ent);
                commands.entity(ent).remove::<CameraTarget>();
            }
        } else {
            debug!("Entity {:?} is no longer controlled locally by self", ent);
            commands.entity(ent).remove::<ControlledLocally>();
        }
    }
}


pub fn react_on_control_removal(mut commands: Commands, 
    mut removed: RemovedComponents<ControlledBy>) {
    for entity in removed.read() {

        commands.entity(entity).remove::<ControlledLocally>();
    }
}






