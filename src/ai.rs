use bevy::prelude::*;
use rand::{thread_rng, Rng};
use crate::components::*;
use crate::resources::*;
use crate::utils::play_tactical_sound;
use crate::spawners::spawn_unit;

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
    
    // Calculate player performance metrics
    let cartel_units = unit_query.iter().filter(|u| u.faction == Faction::Cartel).count();
    let military_units = unit_query.iter().filter(|u| u.faction == Faction::Military).count();
    let ovidio_alive = unit_query.iter().any(|u| u.unit_type == UnitType::Ovidio && u.health > 0.0);
    
    // Update performance based on current situation
    let performance_factor = match (cartel_units, military_units, ovidio_alive) {
        (c, m, true) if c > m => 0.8, // Player doing well
        (c, m, true) if c == m => 0.5, // Balanced
        (c, m, true) if c < m => 0.2, // Player struggling
        (_, _, false) => 0.0, // Ovidio dead - critical situation
        (_, _, true) => 0.5, // Default case for Ovidio alive
    };
    
    ai_director.player_performance = ai_director.player_performance * 0.9 + performance_factor * 0.1;
    
    // Adjust difficulty based on game phase
    let phase_difficulty = match game_state.game_phase {
        GamePhase::MainMenu | GamePhase::SaveMenu | GamePhase::LoadMenu | GamePhase::MissionBriefing => 0.0,
        GamePhase::Preparation => 0.8,
        GamePhase::InitialRaid => 1.0,
        GamePhase::BlockConvoy => 1.2,
        GamePhase::ApplyPressure => 1.4,
        GamePhase::HoldTheLine => 1.6,
        GamePhase::GameOver => 0.0,
    };
    
    // Adaptive difficulty: make it easier if player is struggling
    let adaptive_modifier = if ai_director.adaptive_difficulty {
        if ai_director.player_performance < 0.3 {
            0.7 // Reduce difficulty
        } else if ai_director.player_performance > 0.8 {
            1.3 // Increase difficulty  
        } else {
            1.0
        }
    } else {
        1.0
    };
    
    ai_director.intensity_level = phase_difficulty * adaptive_modifier;
    
    // Dynamic reinforcement spawning based on intensity
    if ai_director.last_spawn_time > 45.0 && ai_director.intensity_level > 1.2 {
        // Spawn additional military units if intensity is high
        let spawn_count = (ai_director.intensity_level * 2.0) as u32;
        
        for i in 0..spawn_count.min(3) {
            let spawn_pos = Vec3::new(
                200.0 + i as f32 * 50.0,
                thread_rng().gen_range(-100.0..100.0),
                0.0,
            );
            
            let unit_type = if thread_rng().gen_bool(0.3) {
                UnitType::SpecialForces
            } else {
                UnitType::Soldier  
            };
            
            spawn_unit(&mut commands, unit_type, Faction::Military, spawn_pos, &game_assets);
        }
        
        play_tactical_sound("radio", &format!("AI Director: Intensity level {:.1} - {} reinforcements deployed", ai_director.intensity_level, spawn_count));
        ai_director.last_spawn_time = 0.0;
    }
}

// ==================== UNIT AI SYSTEM ====================

pub fn unit_ai_system(
    mut unit_query: Query<(&mut Unit, &Transform, &mut Movement), Without<Objective>>,
    objective_query: Query<&Transform, (With<Objective>, Without<Unit>)>,
    time: Res<Time>,
) {
    for (mut unit, transform, mut movement) in unit_query.iter_mut() {
        // Update attack cooldown
        unit.attack_cooldown.tick(time.delta());

        // Basic AI behavior based on faction
        match unit.faction {
            Faction::Military => {
                // Military units try to advance toward objectives or Ovidio
                if movement.target_position.is_none() {
                    // Find closest objective
                    let mut closest_objective_pos = None;
                    let mut closest_distance = f32::INFINITY;
                    
                    for obj_transform in objective_query.iter() {
                        let distance = transform.translation.distance(obj_transform.translation);
                        if distance < closest_distance {
                            closest_distance = distance;
                            closest_objective_pos = Some(obj_transform.translation);
                        }
                    }
                    
                    // Move toward objective if found, otherwise move toward center
                    movement.target_position = closest_objective_pos.or(Some(Vec3::ZERO));
                }
            },
            Faction::Cartel => {
                // Cartel units maintain defensive positions or patrol
                if movement.target_position.is_none() {
                    // Simple patrol behavior
                    let patrol_range = 100.0;
                    let patrol_pos = Vec3::new(
                        transform.translation.x + thread_rng().gen_range(-patrol_range..patrol_range),
                        transform.translation.y + thread_rng().gen_range(-patrol_range..patrol_range),
                        0.0,
                    );
                    movement.target_position = Some(patrol_pos);
                }
            },
            _ => {}
        }
    }
}

// ==================== END OF AI SYSTEMS ====================