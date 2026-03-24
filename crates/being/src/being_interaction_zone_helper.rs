use bevy::prelude::*;
use common::common_components::{HashId, HashIdMap};
use tilemap_shared::{tilemap_seris::InteractionZoneSeri, InteractionZone, InteractionZones};

use crate::{
    being_inst_template::being_inst_template_resources::BitRef,
    race::race_resources::RaceRef,
};

//refactorizar esto

pub fn default_interaction_zone(zone_id: HashId) -> InteractionZone {
    if zone_id == InteractionZones::MELEE_ATTACK {
        return InteractionZone::melee_default_zone();
    }
    if zone_id == InteractionZones::COLLISION {
        return InteractionZone::collision_default_zone();
    }
    InteractionZone::from_seri(InteractionZoneSeri::default())
}

pub fn interaction_zone_from_seri_or_default(
    zone_id: HashId,
    zone_seri: InteractionZoneSeri,
) -> InteractionZone {
    if zone_seri.is_sentinel() {
        return default_interaction_zone(zone_id);
    }
    InteractionZone::from_seri(zone_seri)
}

pub fn build_being_interaction_zones(
    melee_zone_seri: InteractionZoneSeri,
    collision_zone_seri: InteractionZoneSeri,
) -> InteractionZones {
    let mut zones = HashIdMap::with_capacity(2);
    zones.overwrite(
        InteractionZones::MELEE_ATTACK,
        interaction_zone_from_seri_or_default(InteractionZones::MELEE_ATTACK, melee_zone_seri),
    );
    zones.overwrite(
        InteractionZones::COLLISION,
        interaction_zone_from_seri_or_default(InteractionZones::COLLISION, collision_zone_seri),
    );
    InteractionZones(zones)
}

pub fn build_being_interaction_zones_with_base(
    base_zones: Option<&InteractionZones>,
    melee_zone_seri: InteractionZoneSeri,
    collision_zone_seri: InteractionZoneSeri,
) -> InteractionZones {
    let mut zones = HashIdMap::with_capacity(2);
    zones.overwrite(
        InteractionZones::MELEE_ATTACK,
        resolve_zone_with_base(
            base_zones,
            InteractionZones::MELEE_ATTACK,
            melee_zone_seri,
        ),
    );
    zones.overwrite(
        InteractionZones::COLLISION,
        resolve_zone_with_base(
            base_zones,
            InteractionZones::COLLISION,
            collision_zone_seri,
        ),
    );
    InteractionZones(zones)
}

fn resolve_zone_with_base(
    base_zones: Option<&InteractionZones>,
    zone_id: HashId,
    zone_seri: InteractionZoneSeri,
) -> InteractionZone {
    if !zone_seri.is_sentinel() {
        return InteractionZone::from_seri(zone_seri);
    }
    if let Some(zone) = base_zones.and_then(|base| base.0.get(zone_id).ok()) {
        return zone.clone();
    }
    default_interaction_zone(zone_id)
}

pub fn resolve_being_interaction_zone(
    being_interaction_zones: Option<&InteractionZones>,
    bit_ref: Option<&BitRef>,
    race_ref: Option<&RaceRef>,
    zone_id: HashId,
    zone_sources: &Query<&InteractionZones>,
) -> InteractionZone {
    if let Some(zone) = being_interaction_zones.and_then(|zones| zones.0.get(zone_id).ok()) {
        return zone.clone();
    }
    if let Some(zone) = bit_ref
        .and_then(|bit_ref| zone_sources.get(bit_ref.0).ok())
        .and_then(|zones| zones.0.get(zone_id).ok())
    {
        return zone.clone();
    }
    if let Some(zone) = race_ref
        .and_then(|race_ref| zone_sources.get(race_ref.0).ok())
        .and_then(|zones| zones.0.get(zone_id).ok())
    {
        return zone.clone();
    }
    default_interaction_zone(zone_id)
}
