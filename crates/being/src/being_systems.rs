use ::being_shared::*;
use bevy::{ecs::entity::EntityHashMap, prelude::*};
use camera::camera_components::CameraTarget;
use common::log_targets::{BEING_CONTROL, BEING_SYSTEM};
use faction::faction_components::*;
use game_common::game_common_components::{EntityZeroRef};
use game_common::game_common_samplers::GlobalTilePosWeightedSampler;
use modifier::{modifier_components::*, modifier_move_bundles::TempSpeedModifier,};
use movement::movement_messages::*;
use player::player_components::*;
use tilemap::{chunking::chunking_components::ActivatingChunks, chunking::chunking_resources::AaChunkRangeSettings, tile::tile_components::*};
use ::tilemap_shared::*;

use crate::{being_components::*};

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

    query: Query<(Entity, &ControlledBy, Has<CameraTarget>),(Or<(Changed<ControlledBy>, )>)>,
    mut removed_controlled_by: RemovedComponents<ControlledBy>,
    chunk_range: Res<AaChunkRangeSettings>,
) {
    for being_ent in removed_controlled_by.read() {
        commands.entity(being_ent).try_remove::<ComputedLocally>();
        commands.entity(being_ent).try_remove::<PlayerControlled>();
        commands.entity(being_ent).try_remove::<CameraTarget>();
    }
    let Ok((self_entity, is_host)) = self_player.single() else {
        error!(target: BEING_SYSTEM, "No self player found when trying to update control changes");
        return;
    };
    query.iter().for_each(|(being_ent, controlled_by, is_camera_target)| {
        if controlled_by.client_ent == self_entity {
            info!(target: BEING_CONTROL, "debug {:?} is now controlled locally by self", being_ent);
            commands.entity(being_ent).try_insert_if_new((ComputedLocally, ActivatingChunks::new(&chunk_range)));
            if controlled_by.human_input {//PROVISORIO
                debug!(target: BEING_CONTROL, "Entity {:?} is now a CameraTarget", being_ent);
                commands.entity(being_ent).try_insert((PlayerControlled, CameraTarget::default()));
            } else {
                debug!(target: BEING_CONTROL, "Entity {:?} is no longer a CameraTarget", being_ent);
                commands.entity(being_ent).try_remove::<CameraTarget>();
                commands.entity(being_ent).try_remove::<PlayerControlled>();
            }//ENDOF PROVISORIO

        } else {
            commands.entity(being_ent).try_remove::<ComputedLocally>();
            commands.entity(being_ent).try_remove::<CameraTarget>();
            if !is_host{
                commands.entity(being_ent).try_remove::<PlayerControlled>();
                if !is_camera_target{
                    commands.entity(being_ent).try_remove::<ActivatingChunks>();
                }
            }
            else{
                commands.entity(being_ent).try_insert(PlayerControlled);
            }
        }
    });

}

pub fn assign_uncontrolled_beings_to_host(
    mut commands: Commands,
    self_player: Query<Entity, (With<Mine>, With<Player>, With<HostPlayer>)>,
    beings: Query<Entity, (With<Being>, Without<ControlledBy>)>,
) {
    let Ok(self_entity) = self_player.single() else {
        return;
    };
    for being_ent in beings.iter() {
        commands.entity(being_ent).try_insert(ControlledBy {
            client_ent: self_entity,
            human_input: false,
        });
    }
}

#[allow(unused_parens)]
pub fn cross_portal(mut cmd: Commands,
    portal_query: Query<(Entity, &DimensionRef, &PortalTo, &GlobalTilePos, Option<&EntityZeroRef>, Option<&CardinalDirection>), (Without<Being>)>,
    interaction_zones_query: Query<(&InteractionZones), ()>,
    portal_arrival_sampler_query: Query<&GlobalTilePosWeightedSampler>,
    mut being_query: Query<(Entity, &mut DimensionRef, &Transform, &GlobalTransform, Option<&TouchingPortal>), (With<Being>, )>,
) {
    let mut rng = rand::rng();
    for (being_entity, mut being_dimension_ref, being_transform, being_globtransform, touching_portal)
    in being_query.iter_mut() {
        let being_gpos = GlobalTilePos::from(being_globtransform.translation().xy());
        portal_query.iter().for_each(|(portal_ent, &port_dim, port_to, gpos, ezero_ref, direction)| {
            if being_dimension_ref.clone() != port_dim
            { return; }

            let is_interacting = if let Some(ezero_ref) = ezero_ref {
                if let Ok(interaction_zones) = interaction_zones_query.get(ezero_ref.0) {
                    interaction_zones.is_inside_interaction_zone(
                        InteractionZones::ENTER,
                        gpos.to_pixelpos(),
                        being_globtransform.translation().xy(),
                        direction.copied().unwrap_or_default(),
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
                        TempSpeedModifier::new(being_entity, being_entity, 0.0, ApplyMode::Max, 1.0),
                    ));

                    cmd.entity(being_entity).try_insert((TouchingPortal(portal_ent), ));

                    let Ok((_, &oe_dim, _, oe_portal_gpos, oe_ezero_ref, _)) = portal_query.get(port_to.dest_portal) else {
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
                    let transf = (*oe_portal_gpos + sampled_offset).to_translation(being_transform.translation.z);

                    cmd.entity(being_entity)//for replicate_once propagation
                        .try_remove::<Transform>()
                        .try_insert(Transform::from_translation(transf));
                },
            }
        });
    }
}

#[allow(unused_parens)]
pub fn sync_beings_at_gpos(
    mut beings_at_gpos: ResMut<BeingsAtGpos>,
    mut removed_beings: RemovedComponents<Being>,
    mut tracked_pos: Local<EntityHashMap<(DimensionRef, GlobalTilePos)>>,
    query: Query<
        (Entity, &DimensionRef, &Transform),
        (With<Being>, Or<(Added<Being>, Changed<Transform>, Changed<DimensionRef>)>),
    >,
) {
    for ent in removed_beings.read() {
        let Some((old_dim, old_gpos)) = tracked_pos.remove(&ent) else {
            continue;
        };
        beings_at_gpos.remove_being(old_dim, old_gpos, ent);
    }

    for (being_ent, &dim_ref, transform) in query.iter() {
        let gpos = GlobalTilePos::from(transform.translation.xy());
        let prev = tracked_pos.get(&being_ent).copied();

        let Some((old_dim, old_gpos)) = prev else {
            tracked_pos.insert(being_ent, (dim_ref, gpos));
            beings_at_gpos.insert_being(dim_ref, gpos, being_ent);
            continue;
        };

        if old_dim == dim_ref && old_gpos == gpos {
            continue;
        }
        beings_at_gpos.remove_being(old_dim, old_gpos, being_ent);
        beings_at_gpos.insert_being(dim_ref, gpos, being_ent);
        tracked_pos.insert(being_ent, (dim_ref, gpos));
    }
}
