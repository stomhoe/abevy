use ::being_shared::*;
use bevy::{prelude::*};
use bevy_replicon::prelude::*;
use camera::camera_components::CameraTarget;
use common::common_components::HashId;
use faction::faction_components::*;
use game_common::game_common_components::{EntityZeroRef};
use game_common::game_common_samplers::GlobalTilePosWeightedSampler;
use modifier::{modifier_components::*, modifier_move_bundles::TemporalSpeedModifier,};
use movement::movement_messages::TransformFromServer;
use player::player_components::*;
use tilemap::{chunking::chunking_components::ActivatingChunks, chunking::chunking_resources::AaChunkRangeSettings, tile::tile_components::*};
use ::tilemap_shared::*;

use crate::{being_components::*,};

#[allow(unused_parens)]
// A L CENTRO DE LA BASE VA A HABER Q PONERLE UNO DE ALGUNA FORMA
pub fn add_activates_chunks(mut cmd: Commands,
    query: Query<(Entity),(With<Being>, Added<BelongsToAPlayerFaction>)>,
    mut removed: RemovedComponents<BelongsToAPlayerFaction>,
    chunk_range: Res<AaChunkRangeSettings>,
) {
    let mut activates_chunks = Vec::new();
    query.iter().for_each(|ent| { activates_chunks.push((ent, ActivatingChunks::new(&chunk_range))); });
    for ent in removed.read() { cmd.entity(ent).try_remove::<ActivatingChunks>(); }
    cmd.try_insert_batch(activates_chunks);
}

#[allow(unused_parens)]
pub fn on_control_change(
    mut commands: Commands,
    self_player: Query<(Entity, Has<HostPlayer>), (With<Player>, With<Mine>)>,

    query: Query<(Entity, &ControlledBy, &IsHumanControlled, Has<CameraTarget>),(Or<(Changed<ControlledBy>, Changed<IsHumanControlled>)>)>,
    mut removed_controlled_by: RemovedComponents<ControlledBy>,
    chunk_range: Res<AaChunkRangeSettings>,
) {
    for being_ent in removed_controlled_by.read() {
        commands.entity(being_ent).try_remove::<ControlledLocally>();
    }
    let Ok((self_entity, is_host)) = self_player.single() else {
        error!("No self player found when trying to update control changes");
        return;
    };
    query.iter().for_each(|(being_ent, controlled_by, human_controlled, is_camera_target)| {
        if controlled_by.client == self_entity {
            info!(target: "being_control", "debug {:?} is now controlled locally by self", being_ent);
            commands.entity(being_ent).try_insert_if_new((ControlledLocally::default(), ActivatingChunks::new(&chunk_range)));
            if human_controlled.0 {//PROVISORIO
                debug!(target: "being_control", "Entity {:?} is now a CameraTarget", being_ent);
                commands.entity(being_ent).try_insert(CameraTarget::default());
            } else {
                debug!(target: "being_control", "Entity {:?} is no longer a CameraTarget", being_ent);
                commands.entity(being_ent).try_remove::<CameraTarget>();
            }//PROVISORIO
            if is_host {
                commands.entity(being_ent).try_remove::<ControlledByClient>();
            }
        } else {
            commands.entity(being_ent).try_remove::<ControlledLocally>();
            if !is_host{
                if !is_camera_target{
                    commands.entity(being_ent).try_remove::<ActivatingChunks>();
                }
            }
            else{
                commands.entity(being_ent).try_insert(ControlledByClient);
            }
        }
    });

}

#[allow(unused_parens)]
pub fn cross_portal(mut cmd: Commands,
    mut ewriter: MessageWriter<ToClients<TransformFromServer>>,
    portal_query: Query<(Entity, &DimensionRef, &PortalTo, &GlobalTilePos, Option<&EntityZeroRef>), (Without<Being>)>,
    interaction_zones_query: Query<(&InteractionZones), ()>,
    portal_arrival_sampler_query: Query<&GlobalTilePosWeightedSampler>,
    mut being_query: Query<(Entity, &mut DimensionRef, &mut Transform, &GlobalTransform, Option<&TouchingPortal>), (With<Being>, )>,
) {
    let mut rng = rand::rng();
    let mut to_write = Vec::new();
    for (being_entity, mut being_dimension_ref, mut being_transform, being_globtransform, touching_portal)
    in being_query.iter_mut() {
        let being_gpos = GlobalTilePos::from(being_globtransform.translation().xy());
        portal_query.iter().for_each(|(portal_ent, &port_dim, port_to, gpos, ezero_ref)| {
            if being_dimension_ref.clone() != port_dim
            { return; }

            let is_interacting = if let Some(ezero_ref) = ezero_ref {
                if let Ok(interaction_zones) = interaction_zones_query.get(ezero_ref.0) {
                    interaction_zones.is_inside_interaction_zone(
                        InteractionZones::ENTER,
                        gpos.to_pixelpos(),
                        being_globtransform.translation().xy(),
                    )
                } else {
                    being_gpos == *gpos
                }
            } else {
                being_gpos == *gpos
            };

            match (touching_portal, is_interacting) {
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

                    cmd.spawn((
                        TemporalSpeedModifier::new(being_entity, being_entity, 0.0, ApplyMode::Max, 1.0),
                    ));

                    cmd.entity(being_entity).try_insert((TouchingPortal(portal_ent), ));

                    let Ok((_, &oe_dim, _, oe_portal_gpos, oe_ezero_ref)) = portal_query.get(port_to.dest_portal) else {
                        error!("Portal entity {:?} not found in portal query", port_to.dest_portal);//TA DISABLED POR ALGUNA RAZÓN
                        return;
                    };
                    let arrival_sampler = oe_ezero_ref
                        .and_then(|ezero| portal_arrival_sampler_query.get(ezero.0).ok())
                        .and_then(|arrivals| arrivals.sample_with_rng(&mut rng));
                    let sampled_offset = arrival_sampler
                        .or_else(|| port_to.offset_pos_destinations.sample_with_rng(&mut rng))
                        .unwrap_or_default();

                    being_dimension_ref.0 = oe_dim.0;
                    being_transform.translation = (*oe_portal_gpos + sampled_offset).to_translation(being_transform.translation.z);

                    let to_clients = ToClients {
                        mode: SendMode::Broadcast,
                        message: TransformFromServer::new(being_entity, being_transform.clone(), false),
                    };
                    to_write.push(to_clients);
                },
            }

        });
    }
    ewriter.write_batch(to_write);
}
