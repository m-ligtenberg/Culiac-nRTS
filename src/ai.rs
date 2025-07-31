use crate::components::*;
use crate::resources::*;
use crate::spawners::spawn_unit;
use crate::utils::{
    calculate_flanking_position, calculate_kill_ratio, calculate_unit_ratio,
    count_living_units_by_faction, play_tactical_sound,
};
use bevy::prelude::*;
use rand::{thread_rng, Rng};

// ==================== AI DIRECTOR SYSTEM ====================

pub fn ai_director_system(
    mut ai_director: ResMut<AiDirector>,
    game_state: ResMut<GameState>,
    mut commands: Commands,
    game_assets: Res<GameAssets>,
    unit_query: Query<&Unit>,
    time: Res<Time>,
) {
    ai_director.last_spawn_time += time.delta_seconds();

    // Enhanced player performance calculation using utility functions
    let cartel_units = count_living_units_by_faction(&unit_query, Faction::Cartel);
    let military_units = count_living_units_by_faction(&unit_query, Faction::Military);
    let ovidio_alive = unit_query
        .iter()
        .any(|u| u.unit_type == UnitType::Ovidio && u.health > 0.0);

    // Advanced performance calculation using utility functions
    let unit_ratio = calculate_unit_ratio(&unit_query, Faction::Cartel, Faction::Military);
    let kill_ratio = calculate_kill_ratio(&unit_query, Faction::Cartel, Faction::Military);

    let ovidio_factor = if ovidio_alive { 1.0 } else { 0.0 };
    let time_factor = (game_state.mission_timer / 300.0).min(1.0); // Normalize to 5 minutes

    // Composite performance score (0.0 = struggling, 1.0 = dominating)
    let current_performance =
        (unit_ratio * 0.3 + kill_ratio * 0.3 + ovidio_factor * 0.3 + time_factor * 0.1)
            .clamp(0.0, 1.0);

    // Smooth performance tracking with exponential moving average
    ai_director.player_performance =
        ai_director.player_performance * 0.85 + current_performance * 0.15;

    // Phase-based difficulty progression
    let phase_difficulty = match game_state.game_phase {
        GamePhase::MainMenu
        | GamePhase::SaveMenu
        | GamePhase::LoadMenu
        | GamePhase::MissionBriefing => 0.0,
        GamePhase::Preparation => 0.6,
        GamePhase::InitialRaid => 1.0,
        GamePhase::BlockConvoy => 1.3,
        GamePhase::ApplyPressure => 1.6,
        GamePhase::HoldTheLine => 2.0,
        GamePhase::Victory | GamePhase::Defeat | GamePhase::GameOver => 0.0,
    };

    // Enhanced adaptive difficulty system
    let adaptive_modifier = if ai_director.adaptive_difficulty {
        calculate_adaptive_modifier(ai_director.player_performance, game_state.mission_timer)
    } else {
        1.0
    };

    ai_director.intensity_level = (phase_difficulty * adaptive_modifier).max(0.1);

    // Dynamic spawning with multiple triggers
    let should_spawn =
        check_spawn_conditions(&ai_director, &game_state, cartel_units, military_units);

    if should_spawn {
        let spawn_result =
            execute_dynamic_spawning(&mut commands, &ai_director, &game_assets, &game_state);

        if spawn_result.spawned > 0 {
            play_tactical_sound(
                "radio",
                &format!(
                    "AI Director: Performance {:.0}%, Intensity {:.1} - {} {} units deployed",
                    ai_director.player_performance * 100.0,
                    ai_director.intensity_level,
                    spawn_result.spawned,
                    spawn_result.unit_type_name
                ),
            );
            ai_director.last_spawn_time = 0.0;
        }
    }

    // Adjust existing unit stats based on difficulty
    apply_difficulty_modifiers(&ai_director, &game_state);
}

// ==================== UNIT AI SYSTEM ====================

pub fn unit_ai_system(
    mut unit_query: Query<(&mut Unit, &Transform, &mut Movement), Without<Objective>>,
    _objective_query: Query<&Transform, (With<Objective>, Without<Unit>)>,
    time: Res<Time>,
    _game_state: Res<GameState>,
) {
    // Collect all unit positions for tactical analysis
    let mut cartel_positions = Vec::new();
    let mut military_positions = Vec::new();
    let mut ovidio_position = None;

    // First pass: collect positions for tactical analysis
    for (unit, transform, _) in unit_query.iter() {
        if unit.health <= 0.0 {
            continue;
        }

        match unit.faction {
            Faction::Cartel => {
                cartel_positions.push(transform.translation);
                if unit.unit_type == UnitType::Ovidio {
                    ovidio_position = Some(transform.translation);
                }
            }
            Faction::Military => {
                military_positions.push(transform.translation);
            }
            _ => {}
        }
    }

    for (mut unit, transform, mut movement) in unit_query.iter_mut() {
        if unit.health <= 0.0 {
            continue;
        }

        // Update attack cooldown
        unit.attack_cooldown.tick(time.delta());

        // Enhanced AI behavior based on faction and unit type
        match unit.faction {
            Faction::Military => {
                let behavior =
                    choose_military_behavior(&unit, transform, &cartel_positions, ovidio_position);
                execute_military_behavior(&mut movement, transform, behavior, &cartel_positions);
            }
            Faction::Cartel => {
                let behavior =
                    choose_cartel_behavior(&unit, transform, &military_positions, ovidio_position);
                execute_cartel_behavior(&mut movement, transform, behavior, &military_positions);
            }
            _ => {}
        }
    }
}

// ==================== AI BEHAVIOR SELECTION ====================

#[derive(Debug, Clone)]
enum TacticalBehavior {
    AssaultObjective(Vec3),  // Direct attack on target
    FlankingManeuver(Vec3),  // Attack from the side
    DefensivePosition(Vec3), // Hold defensive stance
    RetreatAndRegroup(Vec3), // Fall back to safety
    SupportAllies(Vec3),     // Move to support nearby units
    PatrolArea(Vec3),        // Maintain area control
    AdvanceCarefully(Vec3),  // Cautious advance
    SuppressiveFire(Vec3),   // Area denial tactics
}

fn choose_military_behavior(
    unit: &Unit,
    transform: &Transform,
    cartel_positions: &[Vec3],
    ovidio_position: Option<Vec3>,
) -> TacticalBehavior {
    let unit_pos = transform.translation;

    // Priority target selection
    let primary_target = if let Some(ovidio_pos) = ovidio_position {
        ovidio_pos
    } else if let Some(&closest_cartel) = cartel_positions
        .iter()
        .min_by_key(|&&pos| (unit_pos.distance(pos) * 1000.0) as i32)
    {
        closest_cartel
    } else {
        Vec3::ZERO
    };

    let distance_to_target = unit_pos.distance(primary_target);
    let nearby_enemies = cartel_positions
        .iter()
        .filter(|&&pos| unit_pos.distance(pos) < 150.0)
        .count();
    let nearby_allies = count_nearby_military_units(unit_pos, &[], 100.0); // Would need all_units_query

    // Tactical decision making based on situation
    match unit.unit_type {
        UnitType::SpecialForces => {
            if unit.health < unit.max_health * 0.3 {
                // Low health - retreat
                let retreat_pos = find_retreat_position(unit_pos, cartel_positions);
                TacticalBehavior::RetreatAndRegroup(retreat_pos)
            } else if nearby_enemies > 2 && nearby_allies < 2 {
                // Outnumbered - use flanking
                let flank_pos =
                    calculate_flanking_position_legacy(unit_pos, primary_target, cartel_positions);
                TacticalBehavior::FlankingManeuver(flank_pos)
            } else if distance_to_target < 80.0 {
                // Close range - assault
                TacticalBehavior::AssaultObjective(primary_target)
            } else {
                // Long range - advance carefully
                TacticalBehavior::AdvanceCarefully(primary_target)
            }
        }
        UnitType::Tank => {
            // Tanks provide heavy fire support from range
            if distance_to_target > 150.0 {
                TacticalBehavior::AdvanceCarefully(primary_target)
            } else {
                TacticalBehavior::SuppressiveFire(primary_target)
            }
        }
        UnitType::Helicopter => {
            // Helicopters maintain distance and provide air support
            if nearby_enemies > 1 {
                TacticalBehavior::SuppressiveFire(primary_target)
            } else {
                TacticalBehavior::PatrolArea(unit_pos)
            }
        }
        UnitType::Engineer => {
            // Engineers focus on support and defensive positions
            if nearby_allies >= 2 {
                TacticalBehavior::SupportAllies(primary_target)
            } else {
                TacticalBehavior::DefensivePosition(unit_pos)
            }
        }
        UnitType::Soldier => {
            if unit.health < unit.max_health * 0.4 {
                let retreat_pos = find_retreat_position(unit_pos, cartel_positions);
                TacticalBehavior::RetreatAndRegroup(retreat_pos)
            } else if nearby_allies >= 2 {
                // Strength in numbers - advance
                TacticalBehavior::AssaultObjective(primary_target)
            } else if distance_to_target > 120.0 {
                // Long range - advance with support
                TacticalBehavior::AdvanceCarefully(primary_target)
            } else {
                // Medium range - suppressive fire
                TacticalBehavior::SuppressiveFire(primary_target)
            }
        }
        UnitType::Vehicle => {
            // Vehicles provide fire support and transport
            if nearby_enemies > 3 {
                TacticalBehavior::SuppressiveFire(primary_target)
            } else {
                TacticalBehavior::AdvanceCarefully(primary_target)
            }
        }
        _ => TacticalBehavior::AssaultObjective(primary_target),
    }
}

fn choose_cartel_behavior(
    unit: &Unit,
    transform: &Transform,
    military_positions: &[Vec3],
    ovidio_position: Option<Vec3>,
) -> TacticalBehavior {
    let unit_pos = transform.translation;

    let nearest_threat = military_positions
        .iter()
        .min_by_key(|&&pos| (unit_pos.distance(pos) * 1000.0) as i32)
        .copied();

    let nearby_enemies = military_positions
        .iter()
        .filter(|&&pos| unit_pos.distance(pos) < 120.0)
        .count();

    match unit.unit_type {
        UnitType::Ovidio => {
            // Ovidio stays defensive and seeks cover
            if nearby_enemies > 0 {
                let safe_pos = find_safest_position(unit_pos, military_positions);
                TacticalBehavior::RetreatAndRegroup(safe_pos)
            } else {
                // Stay in defensive position
                TacticalBehavior::DefensivePosition(unit_pos)
            }
        }
        UnitType::Enforcer => {
            if let Some(ovidio_pos) = ovidio_position {
                let distance_to_ovidio = unit_pos.distance(ovidio_pos);
                if distance_to_ovidio > 100.0 {
                    // Move closer to protect Ovidio
                    TacticalBehavior::SupportAllies(ovidio_pos)
                } else if let Some(threat_pos) = nearest_threat {
                    if unit_pos.distance(threat_pos) < 80.0 {
                        // Engage nearby threats
                        TacticalBehavior::AssaultObjective(threat_pos)
                    } else {
                        // Maintain defensive perimeter
                        TacticalBehavior::DefensivePosition(unit_pos)
                    }
                } else {
                    TacticalBehavior::DefensivePosition(unit_pos)
                }
            } else if let Some(threat_pos) = nearest_threat {
                TacticalBehavior::AssaultObjective(threat_pos)
            } else {
                TacticalBehavior::PatrolArea(unit_pos)
            }
        }
        UnitType::Sicario => {
            if unit.health < unit.max_health * 0.3 {
                let safe_pos = find_safest_position(unit_pos, military_positions);
                TacticalBehavior::RetreatAndRegroup(safe_pos)
            } else if nearby_enemies > 2 {
                // Use hit-and-run tactics
                let retreat_pos = find_retreat_position(unit_pos, military_positions);
                TacticalBehavior::RetreatAndRegroup(retreat_pos)
            } else if let Some(threat_pos) = nearest_threat {
                if unit_pos.distance(threat_pos) < 100.0 {
                    TacticalBehavior::AssaultObjective(threat_pos)
                } else {
                    TacticalBehavior::AdvanceCarefully(threat_pos)
                }
            } else {
                TacticalBehavior::PatrolArea(unit_pos)
            }
        }
        UnitType::Sniper => {
            // Snipers maintain distance and find elevated positions
            if let Some(threat_pos) = nearest_threat {
                let sniper_distance = unit_pos.distance(threat_pos);
                if sniper_distance < 150.0 {
                    // Too close - retreat to optimal range
                    let retreat_pos = find_retreat_position(unit_pos, military_positions);
                    TacticalBehavior::RetreatAndRegroup(retreat_pos)
                } else {
                    // Good position - hold and fire
                    TacticalBehavior::DefensivePosition(unit_pos)
                }
            } else {
                TacticalBehavior::PatrolArea(unit_pos)
            }
        }
        UnitType::HeavyGunner => {
            // Heavy gunners provide suppressive fire
            if nearby_enemies > 0 {
                if let Some(threat_pos) = nearest_threat {
                    TacticalBehavior::SuppressiveFire(threat_pos)
                } else {
                    TacticalBehavior::DefensivePosition(unit_pos)
                }
            } else {
                TacticalBehavior::PatrolArea(unit_pos)
            }
        }
        UnitType::Medic => {
            // Medics stay back and support allies
            if let Some(ovidio_pos) = ovidio_position {
                let distance_to_ovidio = unit_pos.distance(ovidio_pos);
                if distance_to_ovidio > 80.0 {
                    // Move closer to support Ovidio
                    TacticalBehavior::SupportAllies(ovidio_pos)
                } else {
                    // Stay in support position
                    TacticalBehavior::DefensivePosition(unit_pos)
                }
            } else if nearby_enemies > 1 {
                // Retreat when threatened
                let safe_pos = find_safest_position(unit_pos, military_positions);
                TacticalBehavior::RetreatAndRegroup(safe_pos)
            } else {
                TacticalBehavior::PatrolArea(unit_pos)
            }
        }
        _ => {
            if let Some(threat_pos) = nearest_threat {
                TacticalBehavior::DefensivePosition(unit_pos)
            } else {
                TacticalBehavior::PatrolArea(unit_pos)
            }
        }
    }
}

// ==================== BEHAVIOR EXECUTION ====================

fn execute_military_behavior(
    movement: &mut Movement,
    transform: &Transform,
    behavior: TacticalBehavior,
    cartel_positions: &[Vec3],
) {
    let current_pos = transform.translation;

    let target_pos = match behavior {
        TacticalBehavior::AssaultObjective(target) => {
            // Direct approach with slight randomization
            let offset = Vec3::new(
                thread_rng().gen_range(-20.0..20.0),
                thread_rng().gen_range(-20.0..20.0),
                0.0,
            );
            target + offset
        }
        TacticalBehavior::FlankingManeuver(target) => {
            calculate_flanking_position(current_pos, target, cartel_positions, 120.0)
        }
        TacticalBehavior::AdvanceCarefully(target) => {
            // Move toward target but maintain distance from enemies
            let direction = (target - current_pos).normalize();
            let safe_advance = current_pos + direction * 40.0;
            avoid_enemy_clusters(safe_advance, cartel_positions, 60.0)
        }
        TacticalBehavior::RetreatAndRegroup(retreat_pos) => retreat_pos,
        TacticalBehavior::SuppressiveFire(target) => {
            // Find good firing position
            find_firing_position(current_pos, target, cartel_positions)
        }
        _ => current_pos, // Default to current position
    };

    movement.target_position = Some(target_pos);
}

fn execute_cartel_behavior(
    movement: &mut Movement,
    transform: &Transform,
    behavior: TacticalBehavior,
    military_positions: &[Vec3],
) {
    let current_pos = transform.translation;

    let target_pos = match behavior {
        TacticalBehavior::DefensivePosition(_) => {
            // Hold position with minor adjustments
            let adjustment = Vec3::new(
                thread_rng().gen_range(-15.0..15.0),
                thread_rng().gen_range(-15.0..15.0),
                0.0,
            );
            current_pos + adjustment
        }
        TacticalBehavior::SupportAllies(ally_pos) => {
            // Move toward ally but maintain tactical spacing
            let direction = (ally_pos - current_pos).normalize();
            let support_pos = ally_pos - direction * 50.0; // Stay 50 units away
            support_pos
        }
        TacticalBehavior::AssaultObjective(target) => {
            // Aggressive advance with cover seeking
            find_covered_approach(current_pos, target, military_positions)
        }
        TacticalBehavior::RetreatAndRegroup(retreat_pos) => retreat_pos,
        TacticalBehavior::PatrolArea(_) => {
            // Patrol around current area
            let patrol_radius = 80.0;
            let angle = thread_rng().gen_range(0.0..std::f32::consts::PI * 2.0);
            current_pos
                + Vec3::new(
                    angle.cos() * patrol_radius,
                    angle.sin() * patrol_radius,
                    0.0,
                )
        }
        TacticalBehavior::AdvanceCarefully(target) => {
            let direction = (target - current_pos).normalize();
            let careful_advance = current_pos + direction * 30.0;
            avoid_enemy_clusters(careful_advance, military_positions, 80.0)
        }
        _ => current_pos,
    };

    movement.target_position = Some(target_pos);
}

// ==================== TACTICAL UTILITY FUNCTIONS ====================

fn calculate_flanking_position_legacy(
    unit_pos: Vec3,
    target_pos: Vec3,
    enemy_positions: &[Vec3],
) -> Vec3 {
    calculate_flanking_position(unit_pos, target_pos, enemy_positions, 120.0)
}

fn find_retreat_position(unit_pos: Vec3, threat_positions: &[Vec3]) -> Vec3 {
    if threat_positions.is_empty() {
        return unit_pos
            + Vec3::new(
                thread_rng().gen_range(-100.0..100.0),
                thread_rng().gen_range(-100.0..100.0),
                0.0,
            );
    }

    // Find direction away from closest threat
    let closest_threat = threat_positions
        .iter()
        .min_by_key(|&&pos| (unit_pos.distance(pos) * 1000.0) as i32)
        .unwrap();

    let escape_direction = (unit_pos - *closest_threat).normalize();
    unit_pos + escape_direction * 150.0
}

fn find_safest_position(unit_pos: Vec3, threat_positions: &[Vec3]) -> Vec3 {
    let mut best_pos = unit_pos;
    let mut best_score = 0.0;

    // Test several positions around the unit
    for i in 0..8 {
        let angle = (i as f32 / 8.0) * std::f32::consts::PI * 2.0;
        let test_pos = unit_pos + Vec3::new(angle.cos() * 100.0, angle.sin() * 100.0, 0.0);

        // Score based on distance from threats
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

fn find_firing_position(unit_pos: Vec3, target_pos: Vec3, enemy_positions: &[Vec3]) -> Vec3 {
    let to_target = (target_pos - unit_pos).normalize();
    let optimal_distance = 100.0;

    // Find position at optimal firing distance
    let firing_pos = target_pos - to_target * optimal_distance;

    // Adjust to avoid other enemies
    avoid_enemy_clusters(firing_pos, enemy_positions, 50.0)
}

fn find_covered_approach(unit_pos: Vec3, target_pos: Vec3, threat_positions: &[Vec3]) -> Vec3 {
    let direct_path = target_pos - unit_pos;
    let distance = direct_path.length();

    if distance < 50.0 {
        return target_pos; // Already close
    }

    // Move partway while avoiding threats
    let safe_distance = distance * 0.3;
    let direction = direct_path.normalize();
    let intermediate_pos = unit_pos + direction * safe_distance;

    avoid_enemy_clusters(intermediate_pos, threat_positions, 70.0)
}

fn avoid_enemy_clusters(
    desired_pos: Vec3,
    enemy_positions: &[Vec3],
    avoidance_radius: f32,
) -> Vec3 {
    let mut adjusted_pos = desired_pos;

    for &enemy_pos in enemy_positions {
        let distance = adjusted_pos.distance(enemy_pos);
        if distance < avoidance_radius {
            let push_direction = (adjusted_pos - enemy_pos).normalize();
            let push_strength = avoidance_radius - distance;
            adjusted_pos += push_direction * push_strength;
        }
    }

    adjusted_pos
}

fn count_nearby_military_units(pos: Vec3, _all_units: &[Vec3], radius: f32) -> usize {
    // Placeholder - would count nearby military units in actual implementation
    thread_rng().gen_range(0..3) // Random for now
}

// ==================== DIFFICULTY CALCULATION FUNCTIONS ====================

// Tijdelijke placeholder implementaties
fn calculate_kill_ratio(_unit_query: &Query<&Unit>, _faction1: Faction, _faction2: Faction) -> f32 {
    0.5
}

fn calculate_unit_ratio(_unit_query: &Query<&Unit>, _faction1: Faction, _faction2: Faction) -> f32 {
    1.0
}

fn calculate_adaptive_modifier(player_performance: f32, mission_time: f32) -> f32 {
    // Base adaptive scaling
    let performance_modifier = if player_performance < 0.2 {
        0.5 // Significant reduction for struggling players
    } else if player_performance < 0.4 {
        0.7 // Moderate reduction
    } else if player_performance < 0.6 {
        0.9 // Slight reduction
    } else if player_performance > 0.8 {
        1.4 // Increase for dominating players
    } else if player_performance > 0.65 {
        1.2 // Moderate increase
    } else {
        1.0 // Balanced
    };

    // Time-based scaling - get harder as mission progresses
    let time_modifier = 1.0 + (mission_time / 600.0) * 0.3; // Up to 30% harder over 10 minutes

    performance_modifier * time_modifier
}

fn check_spawn_conditions(
    ai_director: &AiDirector,
    game_state: &GameState,
    cartel_units: usize,
    military_units: usize,
) -> bool {
    // Multiple spawn triggers
    let time_trigger = ai_director.last_spawn_time > (60.0 / ai_director.intensity_level.max(0.5));
    let intensity_trigger = ai_director.intensity_level > 1.5;
    let imbalance_trigger = cartel_units > military_units * 2; // Too many cartel units
    let phase_trigger = matches!(
        game_state.game_phase,
        GamePhase::ApplyPressure | GamePhase::HoldTheLine
    );

    time_trigger && (intensity_trigger || imbalance_trigger || phase_trigger)
}

struct SpawnResult {
    spawned: u32,
    unit_type_name: &'static str,
}

fn execute_dynamic_spawning(
    commands: &mut Commands,
    ai_director: &AiDirector,
    game_assets: &Res<GameAssets>,
    game_state: &GameState,
) -> SpawnResult {
    let base_spawn_count = (ai_director.intensity_level * 1.5) as u32;
    let spawn_count = base_spawn_count.clamp(1, 4);

    // Determine unit composition based on phase and intensity
    let (primary_unit, secondary_unit, unit_type_name) = match game_state.game_phase {
        GamePhase::InitialRaid => (UnitType::Soldier, UnitType::Soldier, "infantry"),
        GamePhase::BlockConvoy => (UnitType::Vehicle, UnitType::Soldier, "convoy"),
        GamePhase::ApplyPressure => (UnitType::SpecialForces, UnitType::Vehicle, "special ops"),
        GamePhase::HoldTheLine => (UnitType::SpecialForces, UnitType::SpecialForces, "elite"),
        _ => (UnitType::Soldier, UnitType::Soldier, "standard"),
    };

    // Smart spawn positioning - avoid clustering
    let spawn_positions = generate_tactical_spawn_positions(spawn_count);

    for (i, position) in spawn_positions.iter().enumerate() {
        let unit_type = if i == 0 || thread_rng().gen_bool(0.4) {
            primary_unit.clone()
        } else {
            secondary_unit.clone()
        };

        spawn_unit(
            commands,
            unit_type,
            Faction::Military,
            *position,
            game_assets,
        );
    }

    SpawnResult {
        spawned: spawn_count,
        unit_type_name,
    }
}

fn generate_tactical_spawn_positions(count: u32) -> Vec<Vec3> {
    let mut positions = Vec::new();
    let spawn_radius = 250.0;

    // Create multiple entry points for more realistic military tactics
    let entry_angles = [0.0, 90.0, 180.0, 270.0, 45.0, 135.0, 225.0, 315.0];

    for i in 0..count {
        let angle_index = (i as usize) % entry_angles.len();
        let base_angle = (entry_angles[angle_index] as f32).to_radians();

        // Add some randomization to avoid predictable spawning
        let angle_variation = thread_rng().gen_range(-0.3..0.3);
        let final_angle = base_angle + angle_variation;

        let distance_variation = thread_rng().gen_range(0.8..1.2);
        let final_distance = spawn_radius * distance_variation;

        positions.push(Vec3::new(
            final_angle.cos() * final_distance,
            final_angle.sin() * final_distance,
            0.0,
        ));
    }

    positions
}

fn apply_difficulty_modifiers(_ai_director: &AiDirector, _game_state: &GameState) {
    // Future: Apply real-time difficulty modifiers to existing units
    // Could modify unit stats, spawn rates, or AI behavior parameters
}

// ==================== DIFFICULTY SETTINGS SYSTEM ====================

pub fn difficulty_settings_system(mut ai_director: ResMut<AiDirector>, input: Res<Input<KeyCode>>) {
    // Toggle adaptive difficulty with 'D' key
    if input.just_pressed(KeyCode::D) {
        ai_director.adaptive_difficulty = !ai_director.adaptive_difficulty;

        let status = if ai_director.adaptive_difficulty {
            "ENABLED"
        } else {
            "DISABLED"
        };
        play_tactical_sound("radio", &format!("Dynamic Difficulty: {}", status));
    }

    // Manual intensity adjustment with F1-F4 keys
    if input.just_pressed(KeyCode::F1) {
        ai_director.intensity_level = 0.5;
        ai_director.adaptive_difficulty = false; // Disable adaptive when manually set
        play_tactical_sound("radio", "Difficulty set to: EASY");
    } else if input.just_pressed(KeyCode::F2) {
        ai_director.intensity_level = 1.0;
        ai_director.adaptive_difficulty = false;
        play_tactical_sound("radio", "Difficulty set to: NORMAL");
    } else if input.just_pressed(KeyCode::F3) {
        ai_director.intensity_level = 1.5;
        ai_director.adaptive_difficulty = false;
        play_tactical_sound("radio", "Difficulty set to: HARD");
    } else if input.just_pressed(KeyCode::F4) {
        ai_director.intensity_level = 2.0;
        ai_director.adaptive_difficulty = false;
        play_tactical_sound("radio", "Difficulty set to: EXTREME");
    }
}

// ==================== END OF AI SYSTEMS ====================
