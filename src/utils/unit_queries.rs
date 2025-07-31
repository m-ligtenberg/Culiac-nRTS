use bevy::prelude::*;
use crate::components::{Faction, Unit};

pub fn count_living_units_by_faction(unit_query: &Query<&Unit>, faction: Faction) -> usize {
    unit_query.iter()
        .filter(|unit| unit.faction == faction && unit.health > 0.0)
        .count()
}

pub fn count_dead_units_by_faction(unit_query: &Query<&Unit>, faction: Faction) -> usize {
    unit_query.iter()
        .filter(|unit| unit.faction == faction && unit.health <= 0.0)
        .count()
}
