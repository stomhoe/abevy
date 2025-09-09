


use bevy::{prelude::*};
use camera::camera_components::CameraTarget;
use dimension_shared::DimensionRef;
use faction::faction_components::*;
use player::player_components::*;
use tilemap::{chunking_components::ActivatingChunks, chunking_resources::AaChunkRangeSettings, tile::tile_components::{PortalInstance, Tile}};

use crate::{being_components::*,};


#[allow(unused_parens)]
// A L CENTRO DE LA BASE VA A HABER Q PONERLE UNO DE ALGUNA FORMA
pub fn host_add_activates_chunks(mut cmd: Commands, 
    query: Query<(Entity),(With<Being>, Added<BelongsToAPlayerFaction>)>,
    mut removed: RemovedComponents<BelongsToAPlayerFaction>,
    chunk_range: Res<AaChunkRangeSettings>,
) {
    for ent in query.iter() { cmd.entity(ent).try_insert_if_new(ActivatingChunks::new(&chunk_range)); }
    for ent in removed.read() { cmd.entity(ent).try_remove::<ActivatingChunks>(); }
}

#[allow(unused_parens)]
pub fn on_control_change(
    mut commands: Commands, 
    self_player: Query<(Entity, Has<HostPlayer>), (With<Player>, With<OfSelf>)>,

    query: Query<(Entity, &DirControlledBy, &IsHumanControlled, Has<CameraTarget>),(Or<(Changed<DirControlledBy>, Changed<IsHumanControlled>)>)>,
    mut removed: RemovedComponents<DirControlledBy>,
    chunk_range: Res<AaChunkRangeSettings>,
) {
    for ent in removed.read() {
        commands.entity(ent).remove::<ControlledLocally>();
    }
    let (self_entity, is_host) = self_player.single().unwrap();
    for (ent, controlled_by, human_controlled, is_camera_target) in query.iter() {
        if controlled_by.client == self_entity {
            info!("debug {:?} is now controlled locally by self", ent);
            commands.entity(ent).try_insert_if_new((ControlledLocally::default(), ActivatingChunks::new(&chunk_range)));
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

// ----------------------> NO OLVIDARSE DE AGREGARLO AL Plugin DEL MÓDULO <-----------------------------
//                                                       ^^^^
#[allow(unused_parens)]
pub fn cross_portal(mut cmd: Commands, 
    portal_query: Query<(Entity, &DimensionRef, &PortalInstance, &GlobalTransform), (Without<Being>)>,
    mut being_query: Query<(Entity, &mut DimensionRef, &mut Transform, &GlobalTransform, Option<&TouchingPortal>), (With<Being>, )>,
) {
    for (being_entity, mut being_dimension_ref, mut being_transform, being_globtransform, touching_portal) 
    in being_query.iter_mut() {
        for (portal_ent, &dimension_ref, portal_instance, portal_transform) in portal_query.iter() {
            if being_dimension_ref.clone() == dimension_ref {
                let distance = being_globtransform.translation().distance(portal_transform.translation());
                match (touching_portal, distance < 50.0) {
                    (None, false) => {},
                    (Some(&TouchingPortal(touching_portal)), false) => {
                        if portal_ent == touching_portal {
                            cmd.entity(being_entity).try_remove::<TouchingPortal>();
                        }
                    },
                    (Some(&TouchingPortal(touching_portal)), true) => {
                        if portal_ent != touching_portal {
                            cmd.entity(being_entity).try_insert(TouchingPortal(portal_ent));
                        }

                    },
                    (None, true) => {
                        cmd.entity(being_entity).try_insert(TouchingPortal(portal_ent));

                        let Ok((ent, &oe_dim_ref, _portal_instance, portal_transform)) = portal_query.get(portal_instance.dest_portal) else {
                            error!("Portal entity {:?} not found in portal query", portal_ent);
                            continue;
                        };

                        being_dimension_ref.0 = oe_dim_ref.0;
                        being_transform.translation = portal_transform.translation().xy().extend(being_transform.translation.z);
                    },
                }
            }
        }
    }
}
