use ::being_shared::{Being, TouchingPortal};
use ::tilemap_shared::{CardinalDirection, DimensionRef, GlobalTilePos, InteractionZones};
use bevy::prelude::*;
use game_common::game_common_components::EntityZeroRef;
use game_common::game_common_samplers::GlobalTilePosWeightedSampler;
use modifier_shared::{modifier_components::ApplyMode, modifier_move_bundles::TempSpeedModifier};
use tilemap::tile::tile_components::{PortalTo, TileFlip};

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
        Without<Being>,
    >,
    interaction_zones_query: Query<(&InteractionZones, &tilemap_shared::SizeInTiles)>,
    portal_arrival_sampler_query: Query<&GlobalTilePosWeightedSampler>,
    mut being_query: Query<
        (
            Entity,
            &mut DimensionRef,
            &GlobalTilePos,
            &GlobalTransform,
            Option<&TouchingPortal>,
        ),
        (
            With<Being>,
            Changed<GlobalTilePos>,
        )>,
) {
    let mut rng = rand::rng();
    for (being_entity, mut being_dim, &being_gpos, being_gtransf, touching_portal) in being_query.iter_mut() {
        portal_query.iter().for_each(
            |(portal0_ent, &portal0_dim, &portal0_gpos, portal_to, portal0_ezero_ref, portal0_flip, portal0_facedir)| {
                if *being_dim != portal0_dim {
                    return;
                }
                let Some(portal_to) = portal_to else {
                    return;
                };

                let is_interacting = if let Some(ezero_ref) = portal0_ezero_ref {
                    if let Ok((interaction_zones, &size_in_tiles)) = interaction_zones_query.get(ezero_ref.0) {
                        interaction_zones.is_point_inside_zone(
                            InteractionZones::ENTER,
                            size_in_tiles,
                            portal0_gpos.to_pixelpos(),
                            being_gtransf.translation().xy(),
                            portal0_flip.copied().unwrap_or_default(),
                            portal0_facedir.copied().unwrap_or_default(),
                        )
                    } else {
                        being_gpos == portal0_gpos
                    }
                } else {
                    being_gpos == portal0_gpos
                };

                match (touching_portal, is_interacting) {
                    (None, false) => {}
                    (Some(&TouchingPortal(touching_portal)), false) => {
                        if portal0_ent == touching_portal {
                            cmd.entity(being_entity).try_remove::<TouchingPortal>();
                        }
                    }
                    (Some(&TouchingPortal(touching_portal)), true) => {
                        if portal0_ent != touching_portal {
                            cmd.entity(being_entity).try_insert(TouchingPortal(portal0_ent));
                        }
                    }
                    (None, true) => {
                        cmd.spawn((TempSpeedModifier::new(being_entity, being_entity, 0.0, ApplyMode::Max, 1.0),));
                        cmd.entity(being_entity).try_insert(TouchingPortal(portal0_ent));

                        let Ok((_, &dest_dim, &dest_tile_gpos, _, dest_tile_ezero_ref, _, _)) = portal_query.get(portal_to.dest_tile) else {
                            error!("Portal entity {:?} not found in portal query", portal_to.dest_tile);
                            return;
                        };
                        let arrival_sampler = dest_tile_ezero_ref
                            .and_then(|ezero| portal_arrival_sampler_query.get(ezero.0).ok())
                            .and_then(|arrivals| arrivals.sample_with_rng(&mut rng));
                        let sampled_offset = arrival_sampler
                            .or_else(|| portal_to.offset_pos_destinations.sample_with_rng(&mut rng))
                            .unwrap_or_default();

                        being_dim.0 = dest_dim.0;
                        let arrival_gpos = dest_tile_gpos + sampled_offset;
                        cmd.entity(being_entity).try_insert(arrival_gpos);
                    }
                }
            },
        );
    }
}
