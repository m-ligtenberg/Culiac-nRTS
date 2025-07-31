// ==================== UTILITY MODULES ====================

use bevy::log::info;

pub mod abilities;
pub mod ai_optimizer;
pub mod combat;
pub mod formation;
pub mod particle_pool;
pub mod particles;
pub mod spatial;
pub mod ui_builders;
pub mod unit_queries;

// Re-export commonly used functions
pub use abilities::*;
pub use ai_optimizer::*;
pub use combat::*;
pub use formation::*;
pub use particle_pool::*;
pub use particles::*;
pub use spatial::*;
pub use ui_builders::*;
pub use unit_queries::*;

// ==================== SHARED UTILITY FUNCTIONS ====================

use bevy::prelude::*;

pub fn play_tactical_sound(sound_type: &str, message: &str) {
    // Console-based tactical audio system for atmospheric feedback
    // This is now a fallback system for when the enhanced audio system is not available
    match sound_type {
        "radio" => info!("📻 [RADIO] {message}"),
        "gunfire" => info!("🔫 [GUNFIRE] {message}"),
        "explosion" => info!("💥 [EXPLOSION] {message}"),
        "vehicle" => info!("🚗 [VEHICLE] {message}"),
        "ability" => info!("⚡ [ABILITY] {message}"),
        _ => info!("🔊 [AUDIO] {message}"),
    }
}

pub fn play_tactical_sound_at_position(sound_type: &str, message: &str, _position: Vec3) {
    // Enhanced version that includes position information
    // For now, fallback to console output
    match sound_type {
        "radio" => info!(
            "📻 [RADIO] {} (at {:.1}, {:.1})",
            message, _position.x, _position.y
        ),
        "gunfire" => info!(
            "🔫 [GUNFIRE] {} (at {:.1}, {:.1})",
            message, _position.x, _position.y
        ),
        "explosion" => info!(
            "💥 [EXPLOSION] {} (at {:.1}, {:.1})",
            message, _position.x, _position.y
        ),
        "vehicle" => info!(
            "🚗 [VEHICLE] {} (at {:.1}, {:.1})",
            message, _position.x, _position.y
        ),
        "ability" => info!(
            "⚡ [ABILITY] {} (at {:.1}, {:.1})",
            message, _position.x, _position.y
        ),
        _ => info!(
            "🔊 [AUDIO] {} (at {:.1}, {:.1})",
            message, _position.x, _position.y
        ),
    }
}

pub fn world_to_iso(world_pos: Vec3) -> Vec3 {
    // Convert world coordinates to isometric perspective
    let iso_x = (world_pos.x - world_pos.y) * 0.866; // cos(30°)
    let iso_y = (world_pos.x + world_pos.y) * 0.5; // sin(30°)
    Vec3::new(iso_x, iso_y, world_pos.z)
}

// ==================== MISSING UTILITY FUNCTIONS ====================

use crate::components::{Faction, Unit};

pub fn calculate_kill_ratio(
    unit_query: &Query<&Unit>,
    faction1: Faction,
    faction2: Faction,
) -> f32 {
    let faction1_kills = unit_query
        .iter()
        .filter(|u| u.faction == faction1)
        .map(|u| u.kills)
        .sum::<u32>();

    let faction2_kills = unit_query
        .iter()
        .filter(|u| u.faction == faction2)
        .map(|u| u.kills)
        .sum::<u32>();

    if faction2_kills == 0 {
        faction1_kills as f32
    } else {
        faction1_kills as f32 / faction2_kills as f32
    }
}

pub fn calculate_unit_ratio(
    unit_query: &Query<&Unit>,
    faction1: Faction,
    faction2: Faction,
) -> f32 {
    let faction1_alive = count_living_units_by_faction(unit_query, faction1);
    let faction2_alive = count_living_units_by_faction(unit_query, faction2);

    if faction2_alive == 0 {
        faction1_alive as f32
    } else {
        faction1_alive as f32 / faction2_alive as f32
    }
}

pub fn calculate_flanking_position(unit_pos: Vec3, target_pos: Vec3, distance: f32) -> Vec3 {
    // Calculate a flanking position perpendicular to the unit-target line
    let direction = (target_pos - unit_pos).normalize();
    let perpendicular = Vec3::new(-direction.z, direction.y, direction.x);
    target_pos + perpendicular * distance
}

pub fn find_combat_pairs_optimized(
    unit_query: &Query<(Entity, &Unit, &Transform)>,
    max_distance: f32,
) -> Vec<(Entity, Entity)> {
    let mut pairs = Vec::new();
    let units: Vec<_> = unit_query.iter().collect();

    for (i, (entity1, unit1, transform1)) in units.iter().enumerate() {
        for (_j, (entity2, unit2, transform2)) in units.iter().enumerate().skip(i + 1) {
            if unit1.faction != unit2.faction
                && unit1.health > 0.0
                && unit2.health > 0.0
                && transform1.translation.distance(transform2.translation) <= max_distance
            {
                pairs.push((*entity1, *entity2));
            }
        }
    }

    pairs
}
