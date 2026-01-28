


use bevy::{ecs::entity_disabling::Disabled, prelude::*};
use bevy_replicon::prelude::{SendMode, ToClients};
use camera::camera_components::CameraTarget;
use common::common_components::AnyDisabling;
use dimension_shared::DimensionRef;
use faction::faction_components::*;
use movement::movement_events::TransformFromServer;
use player::player_components::*;
use tilemap::{chunking_components::ActivatingChunks, chunking_resources::AaChunkRangeSettings, tile::tile_components::{PortalTo, Tile}};

use crate::{being_components::*,};

#[allow(unused_parens)]
// A L CENTRO DE LA BASE VA A HABER Q PONERLE UNO DE ALGUNA FORMA
pub fn host_add_activates_chunks(mut cmd: Commands, 
    query: Query<(Entity),(With<Being>, Added<BelongsToAPlayerFaction>)>,
    mut removed: RemovedComponents<BelongsToAPlayerFaction>,
    chunk_range: Res<AaChunkRangeSettings>,
) {
    query.iter().for_each(|ent| { cmd.entity(ent).try_insert_if_new(ActivatingChunks::new(&chunk_range)); });
    for ent in removed.read() { cmd.entity(ent).try_remove::<ActivatingChunks>(); }
}

#[allow(unused_parens)]
pub fn on_control_change(
    mut commands: Commands, 
    self_player: Query<(Entity, Has<HostPlayer>), (With<Player>, With<OfSelf>)>,

    query: Query<(Entity, &ControlledBy, &IsHumanControlled, Has<CameraTarget>),(Or<(Changed<ControlledBy>, Changed<IsHumanControlled>)>)>,
    mut removed: RemovedComponents<ControlledBy>,
    chunk_range: Res<AaChunkRangeSettings>,
) {
    for ent in removed.read() {
        commands.entity(ent).try_remove::<ControlledLocally>();
    }
    let (self_entity, is_host) = self_player.single().unwrap();
    query.iter().for_each(|(ent, controlled_by, human_controlled, is_camera_target)| {
        if controlled_by.client == self_entity {
            info!(target: "being_control", "debug {:?} is now controlled locally by self", ent);
            commands.entity(ent).try_insert_if_new((ControlledLocally::default(), ActivatingChunks::new(&chunk_range)));
            if human_controlled.0 {//PROVISORIO
                debug!(target: "being_control", "Entity {:?} is now a CameraTarget", ent);
                commands.entity(ent).try_insert(CameraTarget::default());
            } else {
                debug!(target: "being_control", "Entity {:?} is no longer a CameraTarget", ent);
                commands.entity(ent).try_remove::<CameraTarget>();
            }//PROVISORIO
        } else {
            commands.entity(ent).try_remove::<ControlledLocally>();
            if !is_camera_target && !is_host {
                commands.entity(ent).try_remove::<ActivatingChunks>();
            }
        }
    });

}

#[allow(unused_parens)]
pub fn cross_portal(mut cmd: Commands, 
    mut ewriter: MessageWriter<ToClients<TransformFromServer>>,
    portal_query: Query<(Entity, &DimensionRef, &PortalTo, &GlobalTransform), (Without<Being>)>,
    mut being_query: Query<(Entity, &mut DimensionRef, &mut Transform, &GlobalTransform, Option<&TouchingPortal>), (With<Being>, )>,
) {
    let mut to_write = Vec::new();
    for (being_entity, mut being_dimension_ref, mut being_transform, being_globtransform, touching_portal) 
        in being_query.iter_mut() {
        portal_query.iter().for_each(|(portal_ent, &dimension_ref, portal_instance, portal_transform)| {
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

                let Ok((_, &oe_dim_ref, _oe_portal_instance, oe_portal_transform)) = portal_query.get(portal_instance.dest_portal) else {
                    error!("Portal entity {:?} not found in portal query", portal_instance.dest_portal);//TA DISABLED POR ALGUNA RAZÓN
                    return;
                };

                being_dimension_ref.0 = oe_dim_ref.0;
                being_transform.translation = oe_portal_transform.translation().xy().extend(being_transform.translation.z);

                let to_clients = ToClients { 
                    mode: SendMode::Broadcast, 
                    message: TransformFromServer::new(being_entity, being_transform.clone(), false),
                };
                to_write.push(to_clients);
                },
            }
            }
        });
    }
    ewriter.write_batch(to_write);
}
