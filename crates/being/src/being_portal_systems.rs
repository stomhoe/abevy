use ::being_shared::*;
use ::tilemap_shared::*;
use bevy::prelude::*;
use common::log_targets::BEING_SYSTEM;
use game_common::game_common_components::TemplEntiRef;
use modifier_shared::*;
use tilemap::tile::tile_components::*;
use crate::being_portal_resources::PortalCrossingIndex;

fn portal_is_interacting(
    portal_query: &Query<
        (
            Entity,
            &DimensionRef,
            &GlobalTilePos,
            Option<&PortalTo>,
            Option<&TemplEntiRef>,
            Option<&TileFlip>,
            Option<&CardinalDirection>,
        ),
        (Without<Being>, ),
    >,
    interaction_zones_query: &Query<(&InteractionZones, ), ()>,
    being_dim: DimensionRef,
    being_gpos: GlobalTilePos,
    being_transform: &Transform,
    portal_ent: Entity,
) -> Option<GlobalTilePos> {
    let Ok((_, &portal_dim, &portal_gpos, portal_to, portal_templ_ref, portal_flip, portal_facedir, )) =
        portal_query.get(portal_ent)
    else {
        return None;
    };
    let Some(_) = portal_to else {
        return None;
    };
    if portal_dim != being_dim {
        return None;
    }
    let is_interacting = if let Some(templ_ref) = portal_templ_ref {
        if let Ok((interaction_zones, )) = interaction_zones_query.get(templ_ref.0) {
            interaction_zones.is_point_inside_zone(
                InteractionZones::ENTER,
                portal_gpos.to_pixelpos(),
                portal_facedir.copied().unwrap_or_default(),
                portal_flip.copied().unwrap_or_default(),
                being_transform.translation.xy(),
            )
        } else {
            being_gpos == portal_gpos
        }
    } else {
        being_gpos == portal_gpos
    };
    is_interacting.then_some(portal_gpos)
}

#[allow(unused_parens, )]
pub fn cross_portal(
    mut cmd: Commands,
    portal_index: Res<PortalCrossingIndex>,
    portal_query: Query<
        (
            Entity,
            &DimensionRef,
            &GlobalTilePos,
            Option<&PortalTo>,
            Option<&TemplEntiRef>,
            Option<&TileFlip>,
            Option<&CardinalDirection>,
        ),
        (Without<Being>, ),
    >,
    interaction_zones_query: Query<(&InteractionZones, ), ()>,
    portal_arrival_sampler_query: Query<(&GlobalTilePosWeightedSampler, ), ()>,
    mut being_query: Query<
        (
            Entity,
            &mut DimensionRef,
            &GlobalTilePos,
            &mut Transform,
            &mut GridLockedMovement,
            &mut GridLockedMovementVisual,
            Option<&TouchingPortal>,
        ),
        (
            With<Being>,
            Changed<GlobalTilePos>,
        ),
    >,
) {
    let mut rng = rand::rng();
    for (being_entity, mut being_dim, &being_gpos, mut being_transform, mut being_glm, mut being_glm_visual, touching_portal, ) in being_query.iter_mut() {
        let mut interacting_portal_ent = None;
        if let Some(portals) = portal_index.portals_at_tile(being_dim.0, being_gpos) {
            interacting_portal_ent = portals.iter().copied().find(|portal_ent| {
                portal_is_interacting(
                    &portal_query,
                    &interaction_zones_query,
                    *being_dim,
                    being_gpos,
                    &being_transform,
                    *portal_ent,
                )
                .is_some()
            });
        }
        if interacting_portal_ent.is_none()
            && let Some(portals) = portal_index.portals_in_dimension(being_dim.0)
        {
            interacting_portal_ent = portals.iter().copied().find(|portal_ent| {
                portal_is_interacting(
                    &portal_query,
                    &interaction_zones_query,
                    *being_dim,
                    being_gpos,
                    &being_transform,
                    *portal_ent,
                )
                .is_some()
            });
        }

        let Some(interacting_portal_ent) = interacting_portal_ent else {
            let Some(&TouchingPortal(_, )) = touching_portal else {
                continue;
            };
            cmd.entity(being_entity).try_remove::<TouchingPortal>();
            continue;
        };

        let Some(&TouchingPortal(touching_portal_ent, )) = touching_portal else {
            cmd.spawn((TempSpeedModifier::new(being_entity, being_entity, 0.0, ApplyMode::Max, 0.6), ));
            cmd.entity(being_entity).try_insert(TouchingPortal(interacting_portal_ent, ));

            let Ok((_, _, _, Some(interacting_portal_to), _, _, _, )) = portal_query.get(interacting_portal_ent) else {
                error!(target: BEING_SYSTEM, "Portal entity {:?} not found in portal query", interacting_portal_ent);
                continue;
            };
            let Ok((_, &dest_dim, &dest_tile_gpos, _, dest_tile_templ_ref, _, _, )) = portal_query.get(interacting_portal_to.dest_tile) else {
                error!(target: BEING_SYSTEM, "Portal entity {:?} not found in portal query", interacting_portal_to.dest_tile);
                continue;
            };
            let arrival_sampler = dest_tile_templ_ref
                .and_then(|templ| portal_arrival_sampler_query.get(templ.0).ok())
                .map(|(arrivals, )| arrivals)
                .and_then(|arrivals| arrivals.sample_with_rng(&mut rng));
            let sampled_offset = arrival_sampler
                .or_else(|| interacting_portal_to.offset_pos_destinations.sample_with_rng(&mut rng))
                .unwrap_or_default();
            being_dim.0 = dest_dim.0;
            let arrival_gpos = dest_tile_gpos + sampled_offset;
            being_transform.translation = arrival_gpos.to_translation(being_transform.translation.z);
            being_glm.clear_step(&mut being_glm_visual, arrival_gpos);
            cmd.entity(being_entity).try_insert(arrival_gpos);
            debug!(target: BEING_SYSTEM, "Teleported {:?} from {:?} to {:?} via portal {:?}", being_entity, being_gpos, arrival_gpos, interacting_portal_ent);
            continue;
        };
        if touching_portal_ent == interacting_portal_ent {
            continue;
        }
        cmd.entity(being_entity).try_insert(TouchingPortal(interacting_portal_ent, ));
    }
}
