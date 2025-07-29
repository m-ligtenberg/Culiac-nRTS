use bevy::prelude::*;
use rand::{thread_rng, Rng};
use crate::components::*;
use crate::resources::*;
use crate::utils::play_tactical_sound;
use crate::spawners::spawn_unit;

// ==================== WAVE SPAWNER SYSTEM ====================

pub fn wave_spawner_system(
    time: Res<Time>,
    mut commands: Commands,
    mut wave_query: Query<&mut WaveSpawner>,
    mut game_state: ResMut<GameState>,
    game_assets: Res<GameAssets>,
) {
    for mut spawner in wave_query.iter_mut() {
        spawner.next_wave_timer.tick(time.delta());
        
        if spawner.next_wave_timer.finished() {
            spawner.wave_number += 1;
            game_state.current_wave = spawner.wave_number;
            
            // Calculate spawn positions around the perimeter
            let spawn_radius = 300.0;
            let entry_points = vec![
                Vec3::new(spawn_radius, 0.0, 0.0),     // Right
                Vec3::new(-spawn_radius, 0.0, 0.0),    // Left  
                Vec3::new(0.0, spawn_radius, 0.0),     // Top
                Vec3::new(0.0, -spawn_radius, 0.0),    // Bottom
            ];
            
            // Spawn military units for this wave
            for i in 0..spawner.units_in_wave {
                let entry_point = entry_points[i as usize % entry_points.len()];
                let offset = Vec3::new(
                    thread_rng().gen_range(-50.0..50.0),
                    thread_rng().gen_range(-50.0..50.0),
                    0.0,
                );
                
                let unit_type = match spawner.wave_number {
                    1..=2 => UnitType::Soldier,
                    3..=4 => if thread_rng().gen_bool(0.7) { UnitType::Soldier } else { UnitType::SpecialForces },
                    _ => if thread_rng().gen_bool(0.4) { UnitType::Vehicle } else { UnitType::SpecialForces },
                };
                
                spawn_unit(&mut commands, unit_type, Faction::Military, entry_point + offset, &game_assets);
            }
            
            // Increase difficulty for next wave
            spawner.units_in_wave = (spawner.units_in_wave as f32 * 1.2) as u32;
            
            play_tactical_sound("radio", &format!("Wave {} incoming! {} enemy units approaching from multiple directions", spawner.wave_number, spawner.units_in_wave));
        }
    }
}

// ==================== GAME PHASE SYSTEM ====================

pub fn game_phase_system(
    mut game_state: ResMut<GameState>,
    unit_query: Query<&Unit>,
    time: Res<Time>,
) {
    game_state.mission_timer += time.delta_seconds();
    
    let cartel_units = unit_query.iter().filter(|u| u.faction == Faction::Cartel && u.health > 0.0).count();
    let military_units = unit_query.iter().filter(|u| u.faction == Faction::Military && u.health > 0.0).count();
    let ovidio_alive = unit_query.iter().any(|u| u.unit_type == UnitType::Ovidio && u.health > 0.0);
    
    // Phase transitions based on time and conditions
    match game_state.game_phase {
        GamePhase::Preparation => {
            if game_state.mission_timer > 15.0 {
                game_state.game_phase = GamePhase::InitialRaid;
                play_tactical_sound("radio", "Phase 1: Initial military raid beginning. Defend Ovidio at all costs!");
            }
        },
        GamePhase::InitialRaid => {
            if game_state.mission_timer > 120.0 {
                game_state.game_phase = GamePhase::BlockConvoy;
                play_tactical_sound("radio", "Phase 2: Military convoy approaching. Block their advance!");
            }
        },
        GamePhase::BlockConvoy => {
            if game_state.mission_timer > 240.0 {
                game_state.game_phase = GamePhase::ApplyPressure;
                play_tactical_sound("radio", "Phase 3: Government pressure increasing. Show them the cost of this operation!");
            }
        },
        GamePhase::ApplyPressure => {
            if game_state.mission_timer > 360.0 {
                game_state.game_phase = GamePhase::HoldTheLine;
                play_tactical_sound("radio", "Phase 4: Final push. Hold the line until the government yields!");
            }
        },
        GamePhase::HoldTheLine => {
            // Victory condition: Survive long enough or eliminate enough military
            if game_state.mission_timer > 480.0 || (military_units == 0 && cartel_units > 0) {
                game_state.game_phase = GamePhase::GameOver;
                game_state.cartel_score += 1000; // Bonus for victory
                play_tactical_sound("radio", "VICTORY! Government forces withdraw. Ovidio remains free!");
            }
        },
        GamePhase::GameOver => {
            // Game is over, maintain state
        }
    }
    
    // Defeat condition: Ovidio captured
    if !ovidio_alive && game_state.game_phase != GamePhase::GameOver {
        game_state.game_phase = GamePhase::GameOver;
        game_state.military_score += 1000; // Bonus for capturing Ovidio
        play_tactical_sound("radio", "MISSION FAILED! Ovidio has been captured. The operation is over.");
    }
    
    // Update scores based on eliminated units
    let dead_cartel = unit_query.iter().filter(|u| u.faction == Faction::Cartel && u.health <= 0.0).count();
    let dead_military = unit_query.iter().filter(|u| u.faction == Faction::Military && u.health <= 0.0).count();
    
    game_state.cartel_score = dead_military as u32 * 10;
    game_state.military_score = dead_cartel as u32 * 10;
}

// ==================== MISSION SYSTEM ====================

pub fn mission_system(
    game_state: Res<GameState>,
    unit_query: Query<&Unit>,
    _time: Res<Time>,
) {
    // Mission objectives tracking
    let _ovidio_alive = unit_query.iter().any(|u| u.unit_type == UnitType::Ovidio && u.health > 0.0);
    let _cartel_strength = unit_query.iter().filter(|u| u.faction == Faction::Cartel && u.health > 0.0).count();
    let _military_strength = unit_query.iter().filter(|u| u.faction == Faction::Military && u.health > 0.0).count();
    
    // Mission-specific logic can be added here based on current phase
    match game_state.game_phase {
        GamePhase::Preparation => {
            // Setup phase - ensure all systems are ready
        },
        GamePhase::InitialRaid => {
            // Focus on immediate defense
        },
        GamePhase::BlockConvoy => {
            // Tactical roadblock deployment
        },
        GamePhase::ApplyPressure => {
            // Escalation phase
        },
        GamePhase::HoldTheLine => {
            // Final stand
        },
        GamePhase::GameOver => {
            // Mission complete
        }
    }
}

// ==================== INPUT HANDLING SYSTEM ====================

pub fn handle_input(
    input: Res<Input<KeyCode>>,
    mouse_button_input: Res<Input<MouseButton>>,
    mut commands: Commands,
    mut game_state: ResMut<GameState>,
    game_assets: Res<GameAssets>,
    mut app_exit_events: EventWriter<bevy::app::AppExit>,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform), With<IsometricCamera>>,
    mut selected_units: Query<&mut Movement, (With<Selected>, With<Unit>)>,
    selected_query: Query<Entity, (With<Selected>, With<Unit>)>,
) {
    // Right-click to move selected units
    if mouse_button_input.just_pressed(MouseButton::Right) {
        let window = windows.single();
        if let Ok((camera, camera_transform)) = camera_query.get_single() {
            if let Some(cursor_pos) = window.cursor_position() {
                if let Some(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_pos) {
                    let target_pos = Vec3::new(world_pos.x, world_pos.y, 0.0);
                    
                    // Move all selected units to the target position with formation
                    let selected_count = selected_units.iter().count();
                    for (i, mut movement) in selected_units.iter_mut().enumerate() {
                        let formation_offset = match selected_count {
                            1 => Vec3::ZERO,
                            2..=4 => {
                                let angle = (i as f32 / selected_count as f32) * std::f32::consts::PI * 2.0;
                                Vec3::new(angle.cos() * 30.0, angle.sin() * 30.0, 0.0)
                            },
                            _ => {
                                let row = i / 3;
                                let col = i % 3;
                                Vec3::new((col as f32 - 1.0) * 40.0, (row as f32) * 40.0, 0.0)
                            }
                        };
                        movement.target_position = Some(target_pos + formation_offset);
                    }
                    
                    if selected_count > 0 {
                        play_tactical_sound("radio", &format!("{} units moving to new position", selected_count));
                    }
                }
            }
        }
    }
    
    // Keyboard shortcuts
    if input.just_pressed(KeyCode::Space) {
        // Deploy roadblock at random position
        let roadblock_pos = Vec3::new(
            thread_rng().gen_range(-150.0..150.0),
            thread_rng().gen_range(-150.0..150.0),
            0.0,
        );
        spawn_unit(&mut commands, UnitType::Roadblock, Faction::Cartel, roadblock_pos, &game_assets);
        play_tactical_sound("construction", "Roadblock deployed! Blocking military advance");
        game_state.cartel_score += 5;
    }
    
    if input.just_pressed(KeyCode::R) {
        // Call cartel reinforcements
        let spawn_positions = vec![
            Vec3::new(-150.0, -40.0, 0.0),
            Vec3::new(-100.0, -40.0, 0.0),
            Vec3::new(-150.0, 40.0, 0.0),
        ];
        
        for (i, position) in spawn_positions.iter().enumerate() {
            let unit_type = if i == 0 { UnitType::Enforcer } else { UnitType::Sicario };
            spawn_unit(&mut commands, unit_type, Faction::Cartel, *position, &game_assets);
            
            // Spawn arrival particles
            for _ in 0..8 {
                let velocity = Vec3::new(
                    thread_rng().gen_range(-120.0..120.0),
                    thread_rng().gen_range(-120.0..120.0),
                    0.0,
                );
                
                commands.spawn((
                    SpriteBundle {
                        sprite: Sprite {
                            color: Color::GREEN,
                            custom_size: Some(Vec2::new(4.0, 4.0)),
                            ..default()
                        },
                        transform: Transform::from_translation(*position + Vec3::new(0.0, 0.0, 0.5)),
                        ..default()
                    },
                    ParticleEffect {
                        lifetime: Timer::from_seconds(2.0, TimerMode::Once),
                        velocity,
                    },
                ));
            }
        }
        
        play_tactical_sound("radio", "Reinforcements arriving! 3 additional units deployed");
        game_state.cartel_score += 15;
    }
    
    // Tactical abilities
    if input.just_pressed(KeyCode::Q) {
        // Defensive stance for selected units
        for _entity in selected_query.iter() {
            // Add defensive bonus (could be implemented as a component)
            play_tactical_sound("radio", "Units taking defensive positions");
        }
    }
    
    if input.just_pressed(KeyCode::E) {
        // Aggressive stance for selected units  
        for _entity in selected_query.iter() {
            // Add aggressive bonus (could be implemented as a component)
            play_tactical_sound("radio", "Units switching to aggressive tactics");
        }
    }
    
    // End simulation
    if input.just_pressed(KeyCode::Escape) {
        play_tactical_sound("radio", "Simulation terminated. Historical outcome: Government forces withdrew, Ovidio remained free.");
        info!("🏁 Game ended by user. Final score - Cartel: {}, Military: {}", game_state.cartel_score, game_state.military_score);
        app_exit_events.send(bevy::app::AppExit);
    }
}