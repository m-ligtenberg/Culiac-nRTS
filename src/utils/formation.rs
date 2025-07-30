use crate::components::{Faction, FormationType, UnitType};
use bevy::prelude::*;
use rand::{thread_rng, Rng};

// ==================== FORMATION UTILITIES ====================

/// Calculate position for a unit within a formation
pub fn calculate_formation_position(
    formation_type: FormationType,
    position_in_formation: usize,
    formation_center: Vec3,
    formation_facing: f32,
    unit_count: usize,
) -> Vec3 {
    let position_offset = match formation_type {
        FormationType::Line => {
            let spacing = 40.0;
            let offset_x =
                (position_in_formation as f32 - (unit_count as f32 - 1.0) * 0.5) * spacing;
            Vec3::new(offset_x, 0.0, 0.0)
        }
        FormationType::Circle => {
            let radius = 60.0;
            let angle =
                (position_in_formation as f32 / unit_count as f32) * std::f32::consts::PI * 2.0;
            Vec3::new(angle.cos() * radius, angle.sin() * radius, 0.0)
        }
        FormationType::Wedge => {
            let row = position_in_formation / 2;
            let col = position_in_formation % 2;
            Vec3::new((col as f32 - 0.5) * 30.0, -(row as f32) * 40.0, 0.0)
        }
        FormationType::Flanking => {
            let side = if position_in_formation % 2 == 0 {
                -1.0
            } else {
                1.0
            };
            let pos_in_side = position_in_formation / 2;
            Vec3::new(side * 80.0, (pos_in_side as f32) * 30.0, 0.0)
        }
        FormationType::Overwatch => {
            let spacing = 50.0;
            Vec3::new(0.0, position_in_formation as f32 * spacing, 0.0)
        }
        FormationType::Retreat => {
            let spacing = 35.0;
            Vec3::new((position_in_formation as f32 - 1.0) * spacing, 20.0, 0.0)
        }
    };

    // Rotate offset by formation facing direction
    let rotated_offset = Vec3::new(
        position_offset.x * formation_facing.cos() - position_offset.y * formation_facing.sin(),
        position_offset.x * formation_facing.sin() + position_offset.y * formation_facing.cos(),
        0.0,
    );

    formation_center + rotated_offset
}

/// Calculate optimal spacing for formation based on unit count
pub fn calculate_formation_spacing(unit_count: usize, formation_type: FormationType) -> f32 {
    let base_spacing = match formation_type {
        FormationType::Line => 40.0,
        FormationType::Circle => 60.0 / unit_count as f32,
        FormationType::Wedge => 35.0,
        FormationType::Flanking => 50.0,
        FormationType::Overwatch => 50.0,
        FormationType::Retreat => 35.0,
    };

    // Adjust spacing based on unit count to prevent overcrowding
    let crowding_factor = if unit_count > 5 { 1.2 } else { 1.0 };
    base_spacing * crowding_factor
}

/// Find optimal formation center for a group of units
pub fn find_optimal_formation_center(positions: &[Vec3]) -> Vec3 {
    if positions.is_empty() {
        return Vec3::ZERO;
    }

    let sum: Vec3 = positions.iter().sum();
    sum / positions.len() as f32
}

/// Calculate flanking position around a target
pub fn calculate_flanking_position(
    attacker_pos: Vec3,
    target_pos: Vec3,
    obstacles: &[Vec3],
    preferred_distance: f32,
) -> Vec3 {
    let to_target = (target_pos - attacker_pos).normalize();
    let right_flank = Vec3::new(-to_target.y, to_target.x, 0.0);
    let left_flank = Vec3::new(to_target.y, -to_target.x, 0.0);

    let right_pos = target_pos + right_flank * preferred_distance;
    let left_pos = target_pos + left_flank * preferred_distance;

    // Choose the flank position with fewer obstacles nearby
    let right_obstacle_count = obstacles
        .iter()
        .filter(|&&pos| right_pos.distance(pos) < 80.0)
        .count();
    let left_obstacle_count = obstacles
        .iter()
        .filter(|&&pos| left_pos.distance(pos) < 80.0)
        .count();

    if right_obstacle_count <= left_obstacle_count {
        right_pos
    } else {
        left_pos
    }
}

/// Find safe retreat position away from threats
pub fn calculate_retreat_position(
    unit_pos: Vec3,
    threat_positions: &[Vec3],
    retreat_distance: f32,
) -> Vec3 {
    if threat_positions.is_empty() {
        return unit_pos
            + Vec3::new(
                thread_rng().gen_range(-retreat_distance..retreat_distance),
                thread_rng().gen_range(-retreat_distance..retreat_distance),
                0.0,
            );
    }

    // Find direction away from closest threat
    let closest_threat = threat_positions
        .iter()
        .min_by_key(|&&pos| (unit_pos.distance(pos) * 1000.0) as i32)
        .unwrap();

    let escape_direction = (unit_pos - *closest_threat).normalize();
    let retreat_pos = unit_pos + escape_direction * retreat_distance;

    // Avoid moving too close to other threats
    avoid_threat_clusters(retreat_pos, threat_positions, 60.0)
}

/// Adjust position to avoid clusters of threats
pub fn avoid_threat_clusters(
    desired_pos: Vec3,
    threat_positions: &[Vec3],
    avoidance_radius: f32,
) -> Vec3 {
    let mut adjusted_pos = desired_pos;

    for &threat_pos in threat_positions {
        let distance = adjusted_pos.distance(threat_pos);
        if distance < avoidance_radius {
            let push_direction = (adjusted_pos - threat_pos).normalize();
            let push_strength = avoidance_radius - distance;
            adjusted_pos += push_direction * push_strength;
        }
    }

    adjusted_pos
}

/// Find good cover position near a location
pub fn find_cover_position(unit_pos: Vec3, search_radius: f32, threat_positions: &[Vec3]) -> Vec3 {
    let mut best_pos = unit_pos;
    let mut best_score = 0.0;

    // Test several positions around the unit
    for i in 0..8 {
        let angle = (i as f32 / 8.0) * std::f32::consts::PI * 2.0;
        let test_pos = unit_pos
            + Vec3::new(
                angle.cos() * search_radius,
                angle.sin() * search_radius,
                0.0,
            );

        // Score based on distance from threats (higher is better)
        let mut safety_score = 0.0;
        for &threat_pos in threat_positions {
            safety_score += test_pos.distance(threat_pos);
        }

        if safety_score > best_score {
            best_score = safety_score;
            best_pos = test_pos;
        }
    }

    best_pos
}

/// Get default formation type for a unit type and role
pub fn get_default_formation_type(
    unit_type: &UnitType,
    faction: &Faction,
    context: &str,
) -> FormationType {
    match context {
        "assault" => FormationType::Wedge,
        "defense" => FormationType::Line,
        "retreat" => FormationType::Retreat,
        "flank" => FormationType::Flanking,
        "overwatch" => FormationType::Overwatch,
        _ => match (unit_type, faction) {
            (UnitType::Tank | UnitType::Vehicle, _) => FormationType::Line,
            (UnitType::Helicopter, _) => FormationType::Overwatch,
            (UnitType::Engineer, _) => FormationType::Circle,
            (UnitType::Sniper, _) => FormationType::Overwatch,
            (UnitType::Medic, _) => FormationType::Circle,
            (UnitType::Ovidio, _) => FormationType::Circle,
            (_, Faction::Military) => FormationType::Wedge,
            (_, Faction::Cartel) => FormationType::Line,
            _ => FormationType::Line,
        },
    }
}

/// Calculate formation facing direction based on objective or threat
pub fn calculate_formation_facing(
    formation_center: Vec3,
    objective_pos: Option<Vec3>,
    threat_positions: &[Vec3],
) -> f32 {
    if let Some(objective) = objective_pos {
        let to_objective = (objective - formation_center).normalize();
        to_objective.y.atan2(to_objective.x)
    } else if !threat_positions.is_empty() {
        // Face towards the nearest threat
        let nearest_threat = threat_positions
            .iter()
            .min_by_key(|&&pos| (formation_center.distance(pos) * 1000.0) as i32)
            .unwrap();
        let to_threat = (*nearest_threat - formation_center).normalize();
        to_threat.y.atan2(to_threat.x)
    } else {
        0.0 // Default facing north
    }
}
