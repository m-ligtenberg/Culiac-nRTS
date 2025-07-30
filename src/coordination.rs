use crate::components::*;
use crate::utils::{
    calculate_formation_position, find_optimal_formation_center, play_tactical_sound,
};
use bevy::prelude::*;
use rand::{thread_rng, Rng};

// ==================== SQUAD MANAGEMENT SYSTEM ====================

pub fn squad_management_system(
    mut commands: Commands,
    mut squad_query: Query<(Entity, &mut Squad)>,
    unit_query: Query<(Entity, &Unit, &Transform), Without<Squad>>,
    mut unit_squad_query: Query<
        (
            Entity,
            &Unit,
            &Transform,
            Option<&mut TacticalState>,
            Option<&mut Communication>,
        ),
        With<Squad>,
    >,
    time: Res<Time>,
) {
    // Create squads for unassigned units
    create_new_squads(&mut commands, &unit_query);

    // Update existing squads
    for (squad_entity, mut squad) in squad_query.iter_mut() {
        // Remove dead or missing members
        squad
            .members
            .retain(|&member_entity| unit_squad_query.get(member_entity).is_ok());

        // Dissolve empty squads
        if squad.members.is_empty() {
            commands.entity(squad_entity).despawn();
            continue;
        }

        // Update squad leadership
        update_squad_leadership(&mut squad, &unit_squad_query);

        // Coordinate squad objective
        coordinate_squad_objective(&mut squad, &unit_squad_query, time.elapsed_seconds());
    }
}

fn create_new_squads(
    commands: &mut Commands,
    unit_query: &Query<(Entity, &Unit, &Transform), Without<Squad>>,
) {
    let mut unassigned_cartel: Vec<(Entity, &Unit, &Transform)> = Vec::new();
    let mut unassigned_military: Vec<(Entity, &Unit, &Transform)> = Vec::new();

    // Collect unassigned units by faction
    for (entity, unit, transform) in unit_query.iter() {
        if unit.health <= 0.0 {
            continue;
        }

        match unit.faction {
            Faction::Cartel => unassigned_cartel.push((entity, unit, transform)),
            Faction::Military => unassigned_military.push((entity, unit, transform)),
            _ => {}
        }
    }

    // Create cartel squads
    create_faction_squads(commands, &unassigned_cartel, &Faction::Cartel);

    // Create military squads
    create_faction_squads(commands, &unassigned_military, &Faction::Military);
}

fn create_faction_squads(
    commands: &mut Commands,
    units: &[(Entity, &Unit, &Transform)],
    faction: &Faction,
) {
    if units.len() < 2 {
        return;
    } // Need at least 2 units for a squad

    let mut squad_id_counter = thread_rng().gen_range(1000..9999);

    // Group units into squads of 3-5 members
    for chunk in units.chunks(thread_rng().gen_range(3..=5)) {
        let squad_center = calculate_group_center(chunk);

        // Determine squad type based on unit composition
        let squad_type = determine_squad_type(chunk, faction.clone());

        // Create squad entity
        let squad_entity = commands
            .spawn(Squad {
                id: squad_id_counter,
                leader: Some(chunk[0].0), // First unit becomes leader
                members: chunk.iter().map(|(entity, _, _)| *entity).collect(),
                squad_type,
                current_objective: determine_initial_objective(squad_center, faction.clone()),
                rally_point: Some(squad_center),
                cohesion_radius: 80.0,
            })
            .id();

        // Add squad-related components to member units
        for (entity, unit, transform) in chunk {
            commands.entity(*entity).insert((
                TacticalState {
                    current_state: TacticalMode::HoldPosition,
                    state_timer: 0.0,
                    last_state_change: 0.0,
                    suppression_level: 0.0,
                    morale: 0.8,
                },
                Communication {
                    radio_range: 200.0,
                    last_report_time: 0.0,
                    known_enemies: Vec::new(),
                    received_orders: Vec::new(),
                },
                Formation {
                    formation_type: FormationType::Line,
                    position_in_formation: chunk.iter().position(|(e, _, _)| e == entity).unwrap(),
                    squad_id: squad_id_counter,
                    formation_center: squad_center,
                    formation_facing: 0.0,
                },
            ));
        }

        squad_id_counter += 1;

        play_tactical_sound(
            "radio",
            &format!(
                "{:?} squad {} formed with {} members",
                faction,
                squad_id_counter - 1,
                chunk.len()
            ),
        );
    }
}

fn calculate_group_center(units: &[(Entity, &Unit, &Transform)]) -> Vec3 {
    let positions: Vec<Vec3> = units
        .iter()
        .map(|(_, _, transform)| transform.translation)
        .collect();
    find_optimal_formation_center(&positions)
}

fn determine_squad_type(units: &[(Entity, &Unit, &Transform)], faction: Faction) -> SquadType {
    let unit_types: Vec<&UnitType> = units.iter().map(|(_, unit, _)| &unit.unit_type).collect();

    match faction {
        Faction::Cartel => {
            if unit_types.contains(&&UnitType::Ovidio) {
                SquadType::SecurityTeam
            } else if unit_types
                .iter()
                .any(|&t| matches!(t, UnitType::Enforcer | UnitType::HeavyGunner))
            {
                SquadType::AssaultTeam
            } else if unit_types
                .iter()
                .any(|&t| matches!(t, UnitType::Sniper | UnitType::Medic))
            {
                SquadType::SupportTeam
            } else {
                SquadType::ReconTeam
            }
        }
        Faction::Military => {
            if unit_types
                .iter()
                .any(|&t| matches!(t, UnitType::SpecialForces | UnitType::Tank))
            {
                SquadType::AssaultTeam
            } else if unit_types.iter().any(|&t| {
                matches!(
                    t,
                    UnitType::Vehicle | UnitType::Helicopter | UnitType::Engineer
                )
            }) {
                SquadType::SupportTeam
            } else {
                SquadType::ReconTeam
            }
        }
        _ => SquadType::ReconTeam,
    }
}

fn determine_initial_objective(position: Vec3, faction: Faction) -> SquadObjective {
    match faction {
        Faction::Cartel => SquadObjective::Defend(position),
        Faction::Military => SquadObjective::Advance(Vec3::ZERO), // Move toward center
        _ => SquadObjective::Defend(position),
    }
}

fn update_squad_leadership(
    squad: &mut Squad,
    unit_query: &Query<
        (
            Entity,
            &Unit,
            &Transform,
            Option<&mut TacticalState>,
            Option<&mut Communication>,
        ),
        With<Squad>,
    >,
) {
    // Check if current leader is still valid
    if let Some(leader_entity) = squad.leader {
        if let Ok((_, leader_unit, _, _, _)) = unit_query.get(leader_entity) {
            if leader_unit.health <= 0.0 {
                squad.leader = None;
            }
        } else {
            squad.leader = None;
        }
    }

    // Assign new leader if needed
    if squad.leader.is_none() {
        // Find the unit with highest health and experience
        let mut best_candidate = None;
        let mut best_score = 0.0;

        for &member_entity in &squad.members {
            if let Ok((entity, unit, _, _, _)) = unit_query.get(member_entity) {
                if unit.health > 0.0 {
                    let leadership_score =
                        unit.health + (unit.experience as f32 * 10.0) + (unit.kills as f32 * 5.0);
                    if leadership_score > best_score {
                        best_score = leadership_score;
                        best_candidate = Some(entity);
                    }
                }
            }
        }

        squad.leader = best_candidate;
    }
}

fn coordinate_squad_objective(
    squad: &mut Squad,
    unit_query: &Query<
        (
            Entity,
            &Unit,
            &Transform,
            Option<&mut TacticalState>,
            Option<&mut Communication>,
        ),
        With<Squad>,
    >,
    current_time: f32,
) {
    // Update objective based on squad type and current situation
    match squad.squad_type {
        SquadType::AssaultTeam => coordinate_assault_squad(squad, unit_query),
        SquadType::SupportTeam => coordinate_support_squad(squad, unit_query),
        SquadType::SecurityTeam => coordinate_security_squad(squad, unit_query),
        SquadType::ReconTeam => coordinate_recon_squad(squad, unit_query),
    }
}

fn coordinate_assault_squad(
    squad: &mut Squad,
    unit_query: &Query<
        (
            Entity,
            &Unit,
            &Transform,
            Option<&mut TacticalState>,
            Option<&mut Communication>,
        ),
        With<Squad>,
    >,
) {
    // Assault squads focus on aggressive advancement and flanking
    match &squad.current_objective {
        SquadObjective::Advance(target) => {
            let squad_center = calculate_squad_center(squad, unit_query);
            let distance_to_target = squad_center.distance(*target);

            if distance_to_target < 50.0 {
                // Close to target, switch to engaging or flanking
                let flank_position = calculate_flanking_position(squad_center, *target);
                squad.current_objective = SquadObjective::Flank(*target, flank_position);
            }
        }
        _ => {}
    }
}

fn coordinate_support_squad(
    squad: &mut Squad,
    unit_query: &Query<
        (
            Entity,
            &Unit,
            &Transform,
            Option<&mut TacticalState>,
            Option<&mut Communication>,
        ),
        With<Squad>,
    >,
) {
    // Support squads provide overwatch and suppressive fire
    let squad_center = calculate_squad_center(squad, unit_query);

    // Find good overwatch position
    let overwatch_pos = find_overwatch_position(squad_center);
    squad.current_objective = SquadObjective::Suppress(overwatch_pos);
}

fn coordinate_security_squad(
    squad: &mut Squad,
    unit_query: &Query<
        (
            Entity,
            &Unit,
            &Transform,
            Option<&mut TacticalState>,
            Option<&mut Communication>,
        ),
        With<Squad>,
    >,
) {
    // Security squads protect high-value targets and maintain perimeters
    let squad_center = calculate_squad_center(squad, unit_query);

    // Check if Ovidio is in this squad
    let has_ovidio = squad.members.iter().any(|&member| {
        if let Ok((_, unit, _, _, _)) = unit_query.get(member) {
            unit.unit_type == UnitType::Ovidio
        } else {
            false
        }
    });

    if has_ovidio {
        // Maintain defensive circle around Ovidio
        squad.current_objective = SquadObjective::Defend(squad_center);
    }
}

fn coordinate_recon_squad(
    squad: &mut Squad,
    unit_query: &Query<
        (
            Entity,
            &Unit,
            &Transform,
            Option<&mut TacticalState>,
            Option<&mut Communication>,
        ),
        With<Squad>,
    >,
) {
    // Recon squads advance carefully and gather intelligence
    let squad_center = calculate_squad_center(squad, unit_query);
    let advance_position = squad_center
        + Vec3::new(
            thread_rng().gen_range(-100.0..100.0),
            thread_rng().gen_range(-100.0..100.0),
            0.0,
        );

    squad.current_objective = SquadObjective::Advance(advance_position);
}

fn calculate_squad_center(
    squad: &Squad,
    unit_query: &Query<
        (
            Entity,
            &Unit,
            &Transform,
            Option<&mut TacticalState>,
            Option<&mut Communication>,
        ),
        With<Squad>,
    >,
) -> Vec3 {
    let mut sum = Vec3::ZERO;
    let mut count = 0;

    for &member_entity in &squad.members {
        if let Ok((_, _, transform, _, _)) = unit_query.get(member_entity) {
            sum += transform.translation;
            count += 1;
        }
    }

    if count > 0 {
        sum / count as f32
    } else {
        Vec3::ZERO
    }
}

fn calculate_flanking_position(squad_pos: Vec3, target_pos: Vec3) -> Vec3 {
    let to_target = (target_pos - squad_pos).normalize();
    let perpendicular = Vec3::new(-to_target.y, to_target.x, 0.0);
    let flank_distance = 120.0;

    // Choose left or right flank randomly
    let direction = if thread_rng().gen_bool(0.5) {
        1.0
    } else {
        -1.0
    };
    target_pos + perpendicular * flank_distance * direction
}

fn find_overwatch_position(current_pos: Vec3) -> Vec3 {
    // Find elevated position with good field of view
    current_pos
        + Vec3::new(
            thread_rng().gen_range(-80.0..80.0),
            thread_rng().gen_range(-80.0..80.0),
            0.0,
        )
}

// ==================== FORMATION MOVEMENT SYSTEM ====================

pub fn formation_movement_system(
    mut unit_query: Query<(&mut Movement, &Transform, &Formation, &Squad)>,
    squad_query: Query<&Squad>,
    time: Res<Time>,
) {
    for (mut movement, transform, formation, squad) in unit_query.iter_mut() {
        let formation_position = calculate_formation_position(
            formation.formation_type.clone(),
            formation.position_in_formation,
            formation.formation_center,
            formation.formation_facing,
            squad.members.len(),
        );

        // Maintain formation cohesion
        let distance_to_formation_pos = transform.translation.distance(formation_position);

        if distance_to_formation_pos > 30.0 {
            movement.target_position = Some(calculate_formation_position_legacy(
                formation,
                formation.formation_center,
                formation.formation_facing,
            ));
        }
    }
}

fn calculate_formation_position_legacy(formation: &Formation, center: Vec3, facing: f32) -> Vec3 {
    // Use the new utility function
    calculate_formation_position(
        formation.formation_type.clone(),
        formation.position_in_formation,
        center,
        facing,
        5, // Default unit count for legacy compatibility
    )
}

// ==================== TACTICAL COMMUNICATION SYSTEM ====================

pub fn communication_system(
    mut unit_query: Query<(Entity, &Transform, &mut Communication, &TacticalState)>,
    enemy_query: Query<(Entity, &Transform, &Unit)>,
    time: Res<Time>,
) {
    let current_time = time.elapsed_seconds();

    // Update enemy contacts and share intelligence
    for (entity, transform, mut comm, tactical_state) in unit_query.iter_mut() {
        // Update enemy contact confidence and remove old contacts
        comm.known_enemies.retain_mut(|contact| {
            contact.last_seen += time.delta_seconds();
            contact.confidence *= 0.98; // Decay confidence over time
            contact.confidence > 0.1 && contact.last_seen < 30.0
        });

        // Detect new enemies within line of sight
        for (enemy_entity, enemy_transform, enemy_unit) in enemy_query.iter() {
            let distance = transform.translation.distance(enemy_transform.translation);

            // Check if enemy is within detection range and not blocked
            if distance < 150.0
                && can_see_target(transform.translation, enemy_transform.translation)
            {
                let existing_contact = comm
                    .known_enemies
                    .iter_mut()
                    .find(|contact| contact.position.distance(enemy_transform.translation) < 20.0);

                if let Some(contact) = existing_contact {
                    // Update existing contact
                    contact.position = enemy_transform.translation;
                    contact.confidence = (contact.confidence + 0.1).min(1.0);
                    contact.last_seen = 0.0;
                } else {
                    // Add new contact
                    comm.known_enemies.push(EnemyContact {
                        position: enemy_transform.translation,
                        enemy_type: enemy_unit.unit_type.clone(),
                        confidence: 0.7,
                        last_seen: 0.0,
                    });
                }
            }
        }

        // Intelligence sharing would be handled separately to avoid borrow conflicts
    }
}

fn can_see_target(observer_pos: Vec3, target_pos: Vec3) -> bool {
    // Simplified line of sight check
    let distance = observer_pos.distance(target_pos);
    let height_diff = (target_pos.z - observer_pos.z).abs();

    // Basic visibility rules
    distance < 200.0 && height_diff < 10.0
}

// Intelligence sharing would be implemented as a separate system to avoid borrow conflicts

// ==================== ADVANCED TACTICAL AI SYSTEM ====================

pub fn advanced_tactical_ai_system(
    mut unit_query: Query<(
        Entity,
        &mut Unit,
        &Transform,
        &mut Movement,
        &mut TacticalState,
        &Communication,
        Option<&Formation>,
    )>,
    squad_query: Query<&Squad>,
    time: Res<Time>,
) {
    let current_time = time.elapsed_seconds();

    for (
        entity,
        mut unit,
        transform,
        mut movement,
        mut tactical_state,
        communication,
        formation_opt,
    ) in unit_query.iter_mut()
    {
        if unit.health <= 0.0 {
            continue;
        }

        // Update tactical state timer
        tactical_state.state_timer += time.delta_seconds();

        // Analyze tactical situation
        let situation = analyze_tactical_situation(
            transform.translation,
            &communication.known_enemies,
            unit.faction.clone(),
            tactical_state.suppression_level,
        );

        // Make tactical decision based on current state and situation
        let new_action = decide_tactical_action(
            &tactical_state.current_state,
            &situation,
            tactical_state.morale,
            formation_opt,
        );

        // Execute tactical action
        execute_tactical_action(
            &mut movement,
            &mut tactical_state,
            &new_action,
            transform.translation,
            current_time,
        );

        // Update suppression and morale
        update_psychological_state(&mut tactical_state, &situation, time.delta_seconds());
    }
}

#[derive(Debug)]
struct TacticalSituation {
    enemy_contacts: usize,
    closest_enemy_distance: f32,
    under_fire: bool,
    has_cover: bool,
    squad_support: bool,
    retreat_path_clear: bool,
}

fn analyze_tactical_situation(
    unit_pos: Vec3,
    known_enemies: &[EnemyContact],
    faction: Faction,
    suppression_level: f32,
) -> TacticalSituation {
    let nearby_enemies: Vec<&EnemyContact> = known_enemies
        .iter()
        .filter(|contact| contact.position.distance(unit_pos) < 200.0 && contact.confidence > 0.3)
        .collect();

    let closest_enemy_distance = nearby_enemies
        .iter()
        .map(|contact| contact.position.distance(unit_pos))
        .fold(f32::INFINITY, f32::min);

    TacticalSituation {
        enemy_contacts: nearby_enemies.len(),
        closest_enemy_distance,
        under_fire: suppression_level > 0.3,
        has_cover: check_cover_availability(unit_pos),
        squad_support: check_squad_support(unit_pos),
        retreat_path_clear: check_retreat_path(unit_pos, &nearby_enemies),
    }
}

fn check_cover_availability(pos: Vec3) -> bool {
    // Simplified cover check - in real implementation would check for obstacles
    thread_rng().gen_bool(0.4) // 40% chance of having cover
}

fn check_squad_support(pos: Vec3) -> bool {
    // Simplified squad support check
    thread_rng().gen_bool(0.6) // 60% chance of having squad support nearby
}

fn check_retreat_path(pos: Vec3, enemies: &[&EnemyContact]) -> bool {
    // Check if retreat path is not blocked by enemies
    !enemies.iter().any(|enemy| {
        let retreat_direction = (pos - Vec3::ZERO).normalize();
        let to_enemy = (enemy.position - pos).normalize();
        retreat_direction.dot(to_enemy) > 0.7 // Enemy is in retreat direction
    })
}

#[derive(Debug)]
enum TacticalAction {
    Advance(Vec3),
    Retreat(Vec3),
    TakeCover(Vec3),
    FlankLeft(Vec3),
    FlankRight(Vec3),
    SuppressiveFire(Vec3),
    HoldPosition,
    CallForSupport,
    Regroup(Vec3),
}

fn decide_tactical_action(
    current_state: &TacticalMode,
    situation: &TacticalSituation,
    morale: f32,
    formation: Option<&Formation>,
) -> TacticalAction {
    // Decision tree based on current state, situation, and morale
    match current_state {
        TacticalMode::Advancing => {
            if situation.enemy_contacts > 2 && situation.closest_enemy_distance < 80.0 {
                if situation.has_cover {
                    TacticalAction::TakeCover(Vec3::ZERO) // Take cover nearby
                } else if morale > 0.6 {
                    TacticalAction::FlankLeft(Vec3::ZERO) // Attempt flanking
                } else {
                    TacticalAction::Retreat(Vec3::ZERO)
                }
            } else if situation.enemy_contacts > 0 {
                TacticalAction::SuppressiveFire(Vec3::ZERO)
            } else {
                TacticalAction::Advance(Vec3::ZERO)
            }
        }

        TacticalMode::Engaging => {
            if situation.under_fire && morale < 0.4 {
                if situation.retreat_path_clear {
                    TacticalAction::Retreat(Vec3::ZERO)
                } else {
                    TacticalAction::TakeCover(Vec3::ZERO)
                }
            } else if situation.enemy_contacts > 1 && situation.squad_support {
                // Coordinate with squad for flanking
                if thread_rng().gen_bool(0.5) {
                    TacticalAction::FlankLeft(Vec3::ZERO)
                } else {
                    TacticalAction::FlankRight(Vec3::ZERO)
                }
            } else {
                TacticalAction::SuppressiveFire(Vec3::ZERO)
            }
        }

        TacticalMode::Retreating => {
            if situation.enemy_contacts == 0 {
                TacticalAction::Regroup(Vec3::ZERO)
            } else if situation.has_cover {
                TacticalAction::TakeCover(Vec3::ZERO)
            } else {
                TacticalAction::Retreat(Vec3::ZERO)
            }
        }

        TacticalMode::Suppressed => {
            if situation.under_fire {
                if situation.has_cover {
                    TacticalAction::TakeCover(Vec3::ZERO)
                } else {
                    TacticalAction::CallForSupport
                }
            } else {
                TacticalAction::Regroup(Vec3::ZERO)
            }
        }

        TacticalMode::Flanking => {
            if situation.closest_enemy_distance < 60.0 {
                TacticalAction::SuppressiveFire(Vec3::ZERO)
            } else {
                TacticalAction::FlankLeft(Vec3::ZERO) // Continue flanking
            }
        }

        TacticalMode::Overwatch => {
            if situation.enemy_contacts > 0 {
                TacticalAction::SuppressiveFire(Vec3::ZERO)
            } else {
                TacticalAction::HoldPosition
            }
        }

        TacticalMode::Regrouping => {
            if situation.squad_support {
                TacticalAction::HoldPosition
            } else {
                TacticalAction::Regroup(Vec3::ZERO)
            }
        }

        TacticalMode::HoldPosition => {
            if situation.enemy_contacts > 0 && situation.closest_enemy_distance < 100.0 {
                TacticalAction::SuppressiveFire(Vec3::ZERO)
            } else {
                TacticalAction::HoldPosition
            }
        }
    }
}

fn execute_tactical_action(
    movement: &mut Movement,
    tactical_state: &mut TacticalState,
    action: &TacticalAction,
    current_pos: Vec3,
    current_time: f32,
) {
    match action {
        TacticalAction::Advance(target) => {
            let advance_pos = current_pos
                + Vec3::new(
                    thread_rng().gen_range(-50.0..50.0),
                    thread_rng().gen_range(20.0..80.0),
                    0.0,
                );
            movement.target_position = Some(advance_pos);
            change_tactical_state(tactical_state, TacticalMode::Advancing, current_time);
        }

        TacticalAction::Retreat(target) => {
            let retreat_pos = current_pos
                + Vec3::new(
                    thread_rng().gen_range(-80.0..80.0),
                    thread_rng().gen_range(-120.0..-40.0),
                    0.0,
                );
            movement.target_position = Some(retreat_pos);
            change_tactical_state(tactical_state, TacticalMode::Retreating, current_time);
        }

        TacticalAction::TakeCover(_) => {
            let cover_pos = find_nearest_cover(current_pos);
            movement.target_position = Some(cover_pos);
            change_tactical_state(tactical_state, TacticalMode::HoldPosition, current_time);
        }

        TacticalAction::FlankLeft(_) => {
            let flank_pos = current_pos + Vec3::new(-60.0, 40.0, 0.0);
            movement.target_position = Some(flank_pos);
            change_tactical_state(tactical_state, TacticalMode::Flanking, current_time);
        }

        TacticalAction::FlankRight(_) => {
            let flank_pos = current_pos + Vec3::new(60.0, 40.0, 0.0);
            movement.target_position = Some(flank_pos);
            change_tactical_state(tactical_state, TacticalMode::Flanking, current_time);
        }

        TacticalAction::SuppressiveFire(_) => {
            // Hold position and engage
            movement.target_position = None;
            change_tactical_state(tactical_state, TacticalMode::Engaging, current_time);
        }

        TacticalAction::HoldPosition => {
            movement.target_position = None;
            change_tactical_state(tactical_state, TacticalMode::HoldPosition, current_time);
        }

        TacticalAction::CallForSupport => {
            // Request assistance (would trigger squad response)
            change_tactical_state(tactical_state, TacticalMode::Suppressed, current_time);
        }

        TacticalAction::Regroup(_) => {
            let regroup_pos = current_pos
                + Vec3::new(
                    thread_rng().gen_range(-40.0..40.0),
                    thread_rng().gen_range(-40.0..40.0),
                    0.0,
                );
            movement.target_position = Some(regroup_pos);
            change_tactical_state(tactical_state, TacticalMode::Regrouping, current_time);
        }
    }
}

fn change_tactical_state(
    tactical_state: &mut TacticalState,
    new_state: TacticalMode,
    current_time: f32,
) {
    if tactical_state.current_state != new_state {
        tactical_state.current_state = new_state;
        tactical_state.last_state_change = current_time;
        tactical_state.state_timer = 0.0;
    }
}

fn find_nearest_cover(pos: Vec3) -> Vec3 {
    // Simplified cover finding - move to nearby position
    pos + Vec3::new(
        thread_rng().gen_range(-30.0..30.0),
        thread_rng().gen_range(-30.0..30.0),
        0.0,
    )
}

fn update_psychological_state(
    tactical_state: &mut TacticalState,
    situation: &TacticalSituation,
    delta_time: f32,
) {
    // Update suppression level
    if situation.under_fire {
        tactical_state.suppression_level =
            (tactical_state.suppression_level + delta_time * 0.5).min(1.0);
    } else {
        tactical_state.suppression_level =
            (tactical_state.suppression_level - delta_time * 0.2).max(0.0);
    }

    // Update morale based on situation
    let morale_change = if situation.squad_support {
        0.1 * delta_time
    } else if situation.enemy_contacts > 2 {
        -0.15 * delta_time
    } else if situation.has_cover {
        0.05 * delta_time
    } else {
        -0.05 * delta_time
    };

    tactical_state.morale = (tactical_state.morale + morale_change).clamp(0.0, 1.0);
}
