use crate::components::{Faction, Unit, UnitType};
use bevy::prelude::*;

// ==================== UNIT QUERY UTILITIES ====================

/// Count living units by faction
pub fn count_living_units_by_faction(unit_query: &Query<&Unit>, faction: Faction) -> usize {
    unit_query
        .iter()
        .filter(|unit| unit.faction == faction && unit.health > 0.0)
        .count()
}

/// Count dead units by faction
pub fn count_dead_units_by_faction(unit_query: &Query<&Unit>, faction: Faction) -> usize {
    unit_query
        .iter()
        .filter(|unit| unit.faction == faction && unit.health <= 0.0)
        .count()
}

/// Find units within radius of a position
pub fn find_units_in_radius(
    unit_query: &Query<(Entity, &Unit, &Transform)>,
    center: Vec3,
    radius: f32,
    include_dead: bool,
) -> Vec<(Entity, Vec3, UnitType, f32)> {
    unit_query
        .iter()
        .filter(|(_, unit, _)| include_dead || unit.health > 0.0)
        .filter_map(|(entity, unit, transform)| {
            let distance = center.distance(transform.translation);
            if distance <= radius {
                Some((
                    entity,
                    transform.translation,
                    unit.unit_type.clone(),
                    unit.health,
                ))
            } else {
                None
            }
        })
        .collect()
}

/// Get enemy units relative to a faction
pub fn get_enemy_units(
    unit_query: &Query<(Entity, &Unit, &Transform)>,
    friendly_faction: Faction,
) -> Vec<(Entity, Vec3, UnitType, f32)> {
    unit_query
        .iter()
        .filter(|(_, unit, _)| unit.faction != friendly_faction && unit.health > 0.0)
        .map(|(entity, unit, transform)| {
            (
                entity,
                transform.translation,
                unit.unit_type.clone(),
                unit.health,
            )
        })
        .collect()
}

/// Get ally units relative to a faction
pub fn get_ally_units(
    unit_query: &Query<(Entity, &Unit, &Transform)>,
    friendly_faction: Faction,
) -> Vec<(Entity, Vec3, UnitType, f32)> {
    unit_query
        .iter()
        .filter(|(_, unit, _)| unit.faction == friendly_faction && unit.health > 0.0)
        .map(|(entity, unit, transform)| {
            (
                entity,
                transform.translation,
                unit.unit_type.clone(),
                unit.health,
            )
        })
        .collect()
}

/// Find nearest enemy to a position
pub fn find_nearest_enemy(
    unit_query: &Query<(Entity, &Unit, &Transform)>,
    position: Vec3,
    friendly_faction: Faction,
) -> Option<(Entity, Vec3, UnitType, f32)> {
    get_enemy_units(unit_query, friendly_faction)
        .into_iter()
        .min_by_key(|(_, pos, _, _)| (position.distance(*pos) * 1000.0) as i32)
}

/// Find nearest ally to a position
pub fn find_nearest_ally(
    unit_query: &Query<(Entity, &Unit, &Transform)>,
    position: Vec3,
    friendly_faction: Faction,
) -> Option<(Entity, Vec3, UnitType, f32)> {
    get_ally_units(unit_query, friendly_faction)
        .into_iter()
        .min_by_key(|(_, pos, _, _)| (position.distance(*pos) * 1000.0) as i32)
}

/// Count nearby allies of a specific unit
pub fn count_nearby_allies(
    unit_query: &Query<(Entity, &Unit, &Transform)>,
    position: Vec3,
    friendly_faction: Faction,
    radius: f32,
) -> usize {
    find_units_in_radius(unit_query, position, radius, false)
        .into_iter()
        .filter(|(_, _, _, _)| {
            // Filter for allies by checking faction in the query
            unit_query
                .iter()
                .any(|(entity, unit, _)| unit.faction == friendly_faction)
        })
        .count()
}

/// Count nearby enemies of a specific unit
pub fn count_nearby_enemies(
    unit_query: &Query<(Entity, &Unit, &Transform)>,
    position: Vec3,
    friendly_faction: Faction,
    radius: f32,
) -> usize {
    find_units_in_radius(unit_query, position, radius, false)
        .into_iter()
        .filter(|(_, _, _, _)| {
            // Filter for enemies by checking faction in the query
            unit_query
                .iter()
                .any(|(entity, unit, _)| unit.faction != friendly_faction)
        })
        .count()
}

/// Check if specific unit type exists and is alive
pub fn is_unit_type_alive(
    unit_query: &Query<&Unit>,
    unit_type: UnitType,
    faction: Option<Faction>,
) -> bool {
    unit_query.iter().any(|unit| {
        unit.unit_type == unit_type
            && unit.health > 0.0
            && faction.as_ref().map_or(true, |f| unit.faction == *f)
    })
}

/// Find units of specific type
pub fn find_units_of_type(
    unit_query: &Query<(Entity, &Unit, &Transform)>,
    unit_type: UnitType,
    faction: Option<Faction>,
) -> Vec<(Entity, Vec3, f32)> {
    unit_query
        .iter()
        .filter(|(_, unit, _)| {
            unit.unit_type == unit_type
                && unit.health > 0.0
                && faction.as_ref().map_or(true, |f| unit.faction == *f)
        })
        .map(|(entity, unit, transform)| (entity, transform.translation, unit.health))
        .collect()
}

/// Calculate unit ratio between factions
pub fn calculate_unit_ratio(
    unit_query: &Query<&Unit>,
    faction_a: Faction,
    faction_b: Faction,
) -> f32 {
    let count_a = count_living_units_by_faction(unit_query, faction_a);
    let count_b = count_living_units_by_faction(unit_query, faction_b);

    if count_a + count_b == 0 {
        0.5 // Equal when no units
    } else {
        count_a as f32 / (count_a + count_b) as f32
    }
}

/// Calculate kill ratio between factions
pub fn calculate_kill_ratio(
    unit_query: &Query<&Unit>,
    faction_a: Faction,
    faction_b: Faction,
) -> f32 {
    let kills_by_a = count_dead_units_by_faction(unit_query, faction_b);
    let kills_by_b = count_dead_units_by_faction(unit_query, faction_a);

    if kills_by_a + kills_by_b == 0 {
        0.5 // Equal when no kills
    } else {
        kills_by_a as f32 / (kills_by_a + kills_by_b) as f32
    }
}

/// Get units sorted by distance from position
pub fn get_units_by_distance(
    unit_query: &Query<(Entity, &Unit, &Transform)>,
    position: Vec3,
    faction_filter: Option<Faction>,
    max_count: Option<usize>,
) -> Vec<(Entity, Vec3, UnitType, f32, f32)> {
    // entity, pos, type, health, distance
    let mut units: Vec<_> = unit_query
        .iter()
        .filter(|(_, unit, _)| {
            unit.health > 0.0 && faction_filter.as_ref().map_or(true, |f| unit.faction == *f)
        })
        .map(|(entity, unit, transform)| {
            let distance = position.distance(transform.translation);
            (
                entity,
                transform.translation,
                unit.unit_type.clone(),
                unit.health,
                distance,
            )
        })
        .collect();

    // Sort by distance
    units.sort_by(|a, b| a.4.partial_cmp(&b.4).unwrap());

    // Limit count if specified
    if let Some(max) = max_count {
        units.truncate(max);
    }

    units
}
