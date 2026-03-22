use ::being_shared::*;
use ::tilemap_shared::*;
use bevy::prelude::*;
use common::log_targets::BEING_SYSTEM;
use game_common::game_common_components::EntityZeroRef;
use game_common::game_common_samplers::GlobalTilePosWeightedSampler;
use movement::movement_components::GridLockedMovement;
use modifier_shared::*;
use tilemap::tile::tile_components::*;

#[allow(unused_parens, )]
pub fn cross_portal(
    mut cmd: Commands,
    portal_query: Query<
        (
            Entity,
            &DimensionRef,
            &GlobalTilePos,
            Option<&PortalTo>,
            Option<&EntityZeroRef>,
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
            &GlobalTransform,
            &mut GridLockedMovement,
            Option<&TouchingPortal>,
        ),
        (
            With<Being>,
            Changed<GlobalTilePos>,
        ),
    >,
) {
    let mut rng = rand::rng();
    for (being_entity, mut being_dim, &being_gpos, mut being_transform, being_gtransf, mut being_glm, touching_portal, ) in being_query.iter_mut() {
        let mut interacting_portal: Option<(Entity, &PortalTo, )> = None;
        for (portal_ent, &portal_dim, &portal_gpos, portal_to, portal_ezero_ref, portal_flip, portal_facedir, ) in portal_query.iter() {
            if portal_dim != *being_dim {
                continue;
            }
            let Some(portal_to) = portal_to else {
                continue;
            };
            let is_interacting = if let Some(ezero_ref) = portal_ezero_ref {
                if let Ok((interaction_zones, )) = interaction_zones_query.get(ezero_ref.0) {
                    interaction_zones.is_point_inside_zone(
                        InteractionZones::ENTER,
                        portal_gpos.to_pixelpos(),
                        portal_facedir.copied().unwrap_or_default(),
                        portal_flip.copied().unwrap_or_default(),
                        being_gtransf.translation().xy(),
                    )
                } else {
                    being_gpos == portal_gpos
                }
            } else {
                being_gpos == portal_gpos
            };
            if !is_interacting {
                continue;
            }
            interacting_portal = Some((portal_ent, portal_to, ));
            break;
        }

        let Some((interacting_portal_ent, interacting_portal_to, )) = interacting_portal else {
            let Some(&TouchingPortal(_, )) = touching_portal else {
                continue;
            };
            cmd.entity(being_entity).try_remove::<TouchingPortal>();
            continue;
        };

        let Some(&TouchingPortal(touching_portal_ent, )) = touching_portal else {
            cmd.spawn((TempSpeedModifier::new(being_entity, being_entity, 0.0, ApplyMode::Max, 1.0), ));
            cmd.entity(being_entity).try_insert(TouchingPortal(interacting_portal_ent, ));

            let Ok((_, &dest_dim, &dest_tile_gpos, _, dest_tile_ezero_ref, _, _, )) = portal_query.get(interacting_portal_to.dest_tile) else {
                error!(target: BEING_SYSTEM, "Portal entity {:?} not found in portal query", interacting_portal_to.dest_tile);
                continue;
            };
            let arrival_sampler = dest_tile_ezero_ref
                .and_then(|ezero| portal_arrival_sampler_query.get(ezero.0).ok())
                .map(|(arrivals, )| arrivals)
                .and_then(|arrivals| arrivals.sample_with_rng(&mut rng));
            let sampled_offset = arrival_sampler
                .or_else(|| interacting_portal_to.offset_pos_destinations.sample_with_rng(&mut rng))
                .unwrap_or_default();
            being_dim.0 = dest_dim.0;
            let arrival_gpos = dest_tile_gpos + sampled_offset;
            being_transform.translation = arrival_gpos.to_translation(being_transform.translation.z);
            being_glm.clear_step(arrival_gpos);
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
