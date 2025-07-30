use bevy::prelude::*;
use bevy::ecs::system::ParamSet;
use crate::components::*;
use crate::resources::*;
use crate::utils::play_tactical_sound;
use crate::campaign::Campaign;
use crate::save_system::{save_game, load_game, has_save_file};
use crate::campaign::{get_objective_summary, MissionConfig};

// ==================== UI UPDATE SYSTEMS ====================

pub fn ui_update_system(
    game_state: Res<GameState>,
    ai_director: Res<AiDirector>,
    unit_query: Query<&Unit>,
    mut status_query: Query<&mut Text, (With<StatusText>, Without<WaveText>, Without<ScoreText>, Without<DifficultyDisplay>)>,
    mut wave_query: Query<&mut Text, (With<WaveText>, Without<StatusText>, Without<ScoreText>, Without<DifficultyDisplay>)>,
    mut score_query: Query<&mut Text, (With<ScoreText>, Without<StatusText>, Without<WaveText>, Without<DifficultyDisplay>)>,
    mut difficulty_query: Query<&mut Text, (With<DifficultyDisplay>, Without<StatusText>, Without<WaveText>, Without<ScoreText>)>,
) {
    // Count units by faction
    let cartel_count = unit_query.iter().filter(|u| u.faction == Faction::Cartel && u.health > 0.0).count();
    let military_count = unit_query.iter().filter(|u| u.faction == Faction::Military && u.health > 0.0).count();
    let ovidio_alive = unit_query.iter().any(|u| u.unit_type == UnitType::Ovidio && u.health > 0.0);

    // Update status text
    if let Ok(mut text) = status_query.get_single_mut() {
        let status = if !ovidio_alive {
            "❌ MISSION FAILED: Ovidio captured!"
        } else if game_state.game_phase == GamePhase::GameOver {
            "✅ MISSION SUCCESS: Government retreats!"
        } else {
            match game_state.game_phase {
                GamePhase::MainMenu => "🎮 Main Menu",
                GamePhase::SaveMenu => "💾 Save Game",
                GamePhase::LoadMenu => "📂 Load Game",
                GamePhase::MissionBriefing => "📋 Mission Briefing",
                GamePhase::Preparation => "🔄 Phase: Preparation",
                GamePhase::InitialRaid => "⚔️ Phase: Initial Raid",
                GamePhase::BlockConvoy => "🚧 Phase: Block Convoy",
                GamePhase::ApplyPressure => "🔥 Phase: Apply Pressure",
                GamePhase::HoldTheLine => "🛡️ Phase: Hold The Line",
                GamePhase::Victory => "🏆 VICTORY!",
                GamePhase::Defeat => "💀 DEFEAT!",
                GamePhase::GameOver => "🏁 Mission Complete",
            }
        };
        text.sections[0].value = format!("{}\nCartel: {} | Military: {}", status, cartel_count, military_count);
    }

    // Update wave text
    if let Ok(mut text) = wave_query.get_single_mut() {
        text.sections[0].value = format!("Wave: {} - Timer: {:.1}s",
            game_state.current_wave,
            game_state.mission_timer
        );
    }

    // Update score text
    if let Ok(mut text) = score_query.get_single_mut() {
        text.sections[0].value = format!("Score: Cartel {} - Military {}",
            game_state.cartel_score,
            game_state.military_score
        );
    }

    // Update difficulty display
    if let Ok(mut text) = difficulty_query.get_single_mut() {
        let adaptive_status = if ai_director.adaptive_difficulty { "AUTO" } else { "MANUAL" };
        text.sections[0].value = format!(
            "Difficulty: {:.1} ({}) | Performance: {:.0}%\nD=Toggle | F1-F4=Set Level",
            ai_director.intensity_level,
            adaptive_status,
            ai_director.player_performance * 100.0
        );
    }
}

pub fn health_bar_system(
    mut commands: Commands,
    unit_query: Query<(Entity, &Unit, &Transform), Changed<Unit>>,
    mut health_bar_query: Query<(Entity, &mut Transform, &mut Sprite, &HealthBar), (With<HealthBar>, Without<Unit>)>,
) {
    // Update health bars when units change
    for (unit_entity, unit, unit_transform) in unit_query.iter() {
        for (bar_entity, mut bar_transform, mut bar_sprite, health_bar) in health_bar_query.iter_mut() {
            if health_bar.owner == unit_entity {
                // Update position
                bar_transform.translation = unit_transform.translation + health_bar.offset;

                // Update health bar color and width based on health percentage
                let health_percent = unit.health / unit.max_health;
                let bar_color = if health_percent > 0.6 {
                    Color::rgb(0.2, 0.8, 0.2) // Green
                } else if health_percent > 0.3 {
                    Color::rgb(0.8, 0.8, 0.2) // Yellow
                } else {
                    Color::rgb(0.8, 0.2, 0.2) // Red
                };

                bar_sprite.color = bar_color;

                // Adjust bar width based on health (only for foreground bars)
                if health_bar.offset.z > 0.15 { // Foreground bar
                    if let Some(ref mut size) = bar_sprite.custom_size {
                        size.x = 50.0 * health_percent;
                    }
                }

                // Remove health bar if unit is dead
                if unit.health <= 0.0 {
                    commands.entity(bar_entity).despawn();
                }
            }
        }
    }

    // Clean up health bars for dead units
    let living_units: std::collections::HashSet<Entity> = unit_query.iter()
        .filter(|(_, unit, _)| unit.health > 0.0)
        .map(|(entity, _, _)| entity)
        .collect();

    for (bar_entity, _, _, health_bar) in health_bar_query.iter() {
        if !living_units.contains(&health_bar.owner) {
            commands.entity(bar_entity).despawn();
        }
    }
}

pub fn damage_indicator_system(
    mut commands: Commands,
    mut damage_query: Query<(Entity, &mut Transform, &mut DamageIndicator, Option<&ParticleEffect>)>,
    time: Res<Time>,
) {
    for (entity, mut transform, mut indicator, particle_effect) in damage_query.iter_mut() {
        indicator.lifetime.tick(time.delta());

        // Use particle effect velocity if available, otherwise default upward movement
        if let Some(particle) = particle_effect {
            transform.translation += particle.velocity * time.delta_seconds();
        } else {
            transform.translation.y += 30.0 * time.delta_seconds();
        }

        // Fade out over time for smooth disappearance (future enhancement)
        let _alpha = 1.0 - (indicator.lifetime.elapsed_secs() / indicator.lifetime.duration().as_secs_f32());

        // Remove when expired
        if indicator.lifetime.finished() {
            commands.entity(entity).despawn();
        }
    }
}

pub fn particle_system(
    mut commands: Commands,
    mut particle_query: Query<(Entity, &mut Transform, &mut ParticleEffect)>,
    time: Res<Time>,
) {
    for (entity, mut transform, mut particle) in particle_query.iter_mut() {
        particle.lifetime.tick(time.delta());

        // Move particle
        transform.translation += particle.velocity * time.delta_seconds();

        // Remove when expired
        if particle.lifetime.finished() {
            commands.entity(entity).despawn();
        }
    }
}

// ==================== CAMERA CONTROL SYSTEM ====================

pub fn camera_control_system(
    mut camera_query: Query<(&mut Transform, &mut IsometricCamera), With<Camera>>,
    input: Res<Input<KeyCode>>,
    _mouse_wheel: Res<Input<MouseButton>>,
    mut scroll_events: EventReader<bevy::input::mouse::MouseWheel>,
    time: Res<Time>,
) {
    // Robust camera control with error handling
    let Ok((mut transform, camera)) = camera_query.get_single_mut() else {
        warn!("Camera system: No camera found or multiple cameras detected");
        return;
    };

    let mut movement = Vec3::ZERO;

    // WASD camera movement
    if input.pressed(KeyCode::W) { movement.y += 1.0; }
    if input.pressed(KeyCode::S) { movement.y -= 1.0; }
    if input.pressed(KeyCode::A) { movement.x -= 1.0; }
    if input.pressed(KeyCode::D) { movement.x += 1.0; }

    // Apply movement
    if movement != Vec3::ZERO {
        transform.translation += movement.normalize() * camera.pan_speed * time.delta_seconds();
    }

    // Mouse wheel zoom
    for scroll in scroll_events.read() {
        let zoom_delta = -scroll.y * camera.zoom_speed;
        let new_scale = (transform.scale.x + zoom_delta).clamp(camera.min_zoom, camera.max_zoom);
        transform.scale = Vec3::splat(new_scale);
    }
}

// ==================== UNIT SELECTION SYSTEM ====================

pub fn unit_selection_system(
    mut commands: Commands,
    mouse_button_input: Res<Input<MouseButton>>,
    keyboard_input: Res<Input<KeyCode>>,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform), With<IsometricCamera>>,
    mut unit_queries: ParamSet<(
        Query<(Entity, &Transform, &Unit, Option<&Selected>)>,
        Query<&mut Unit>,
    )>,
    mut movement_query: Query<&mut Movement>,
    selected_query: Query<Entity, With<Selected>>,
) {
    let window = windows.single();

    // Handle left-click selection
    if mouse_button_input.just_pressed(MouseButton::Left) {
        let Ok((camera, camera_transform)) = camera_query.get_single() else {
            warn!("Unit selection: Camera not available for viewport conversion");
            return;
        };

        if let Some(cursor_pos) = window.cursor_position() {
            if let Some(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_pos) {
                let click_pos = Vec3::new(world_pos.x, world_pos.y, 0.0);

                // Clear previous selection if not holding shift
                if !keyboard_input.pressed(KeyCode::ShiftLeft) && !keyboard_input.pressed(KeyCode::ShiftRight) {
                    for entity in selected_query.iter() {
                        commands.entity(entity).remove::<Selected>();
                    }
                }

                // Find closest unit to click position
                let mut closest_unit = None;
                let mut closest_distance = f32::INFINITY;

                for (entity, transform, unit, selected) in unit_queries.p0().iter() {
                    // Only select cartel units (player units)
                    if unit.faction != Faction::Cartel || unit.health <= 0.0 {
                        continue;
                    }

                    let distance = transform.translation.distance(click_pos);
                    if distance < 50.0 && distance < closest_distance {
                        closest_distance = distance;
                        closest_unit = Some((entity, selected.is_some()));
                    }
                }

                // Select the closest unit
                if let Some((entity, already_selected)) = closest_unit {
                    if !already_selected {
                        commands.entity(entity).insert(Selected {
                            selection_color: Color::CYAN,
                        });
                    }
                }
            }
        }
    }

    // Handle right-click commands (movement or attack)
    if mouse_button_input.just_pressed(MouseButton::Right) {
        if let Ok((camera, camera_transform)) = camera_query.get_single() {
            if let Some(cursor_pos) = window.cursor_position() {
            if let Some(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_pos) {
                    let target_pos = Vec3::new(world_pos.x, world_pos.y, 0.0);

                    // Collect selected units
                    let selected_units: Vec<Entity> = selected_query.iter().collect();

                    if !selected_units.is_empty() {
                        // Check if right-clicking on an enemy unit for attack command
                        let target_enemy = find_enemy_at_position(target_pos, &unit_queries.p0());

                        if let Some(enemy_entity) = target_enemy {
                            // Attack command: assign enemy as target
                            assign_attack_targets(&selected_units, enemy_entity, &mut unit_queries.p1());
                            play_tactical_sound("radio", &format!("{} units ordered to attack target", selected_units.len()));
                        } else {
                            // Movement command: formation movement
                            let formation_type = if keyboard_input.pressed(KeyCode::ControlLeft) {
                                FormationType::Wedge
                            } else if keyboard_input.pressed(KeyCode::AltLeft) {
                                FormationType::Circle
                            } else {
                                FormationType::Line
                            };

                            assign_formation_positions(&selected_units, target_pos, formation_type.clone(), &mut movement_query);
                            play_tactical_sound("movement", &format!("{} units moving in {:?} formation", selected_units.len(), formation_type));
                        }
                    }
                }
            }
        }
    }
}

pub fn selection_indicator_system(
    mut commands: Commands,
    selected_query: Query<(Entity, &Transform, &Selected), (With<Unit>, Changed<Selected>)>,
    indicator_query: Query<(Entity, &mut Transform), (With<SelectionIndicator>, Without<Unit>)>,
) {
    // Remove old indicators
    for (entity, _) in indicator_query.iter() {
        commands.entity(entity).despawn();
    }

    // Create enhanced selection indicators for selected units
    for (_, transform, selected) in selected_query.iter() {
        // Outer selection ring (animated)
        commands.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: Color::rgba(selected.selection_color.r(), selected.selection_color.g(), selected.selection_color.b(), 0.6),
                    custom_size: Some(Vec2::new(48.0, 48.0)),
                    ..default()
                },
                transform: Transform::from_translation(transform.translation + Vec3::new(0.0, 0.0, -0.1)),
                ..default()
            },
            SelectionIndicator,
        ));

        // Inner selection ring (solid)
        commands.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: Color::rgba(selected.selection_color.r(), selected.selection_color.g(), selected.selection_color.b(), 0.8),
                    custom_size: Some(Vec2::new(40.0, 2.0)), // Thin ring
                    ..default()
                },
                transform: Transform::from_translation(transform.translation + Vec3::new(0.0, -25.0, 0.1)),
                ..default()
            },
            SelectionIndicator,
        ));

        // Selection corners for better visibility
        for (x, y) in [(-15.0, 15.0), (15.0, 15.0), (-15.0, -15.0), (15.0, -15.0)] {
            commands.spawn((
                SpriteBundle {
                    sprite: Sprite {
                        color: selected.selection_color,
                        custom_size: Some(Vec2::new(8.0, 8.0)),
                        ..default()
                    },
                    transform: Transform::from_translation(transform.translation + Vec3::new(x, y, 0.2)),
                    ..default()
                },
                SelectionIndicator,
            ));
        }
    }
}

// ==================== TARGET INDICATOR SYSTEM ====================

pub fn target_indicator_system(
    mut commands: Commands,
    unit_query: Query<(&Unit, &Transform)>,
    target_indicator_query: Query<(Entity, &mut Transform), (With<TargetIndicator>, Without<Unit>)>,
) {
    // Remove old target indicators
    for (entity, _) in target_indicator_query.iter() {
        commands.entity(entity).despawn();
    }

    // Create target indicators for units with assigned targets
    for (unit, _) in unit_query.iter() {
        if let Some(target_entity) = unit.target {
            // Find the target's position
            if let Ok((_, target_transform)) = unit_query.get(target_entity) {
                // Create a red crosshair indicator on the target
                commands.spawn((
                    SpriteBundle {
                        sprite: Sprite {
                            color: Color::rgb(1.0, 0.2, 0.2),
                            custom_size: Some(Vec2::new(40.0, 4.0)),
                            ..default()
                        },
                        transform: Transform::from_translation(target_transform.translation + Vec3::new(0.0, 0.0, 0.3)),
                        ..default()
                    },
                    TargetIndicator,
                ));

                // Vertical crosshair
                commands.spawn((
                    SpriteBundle {
                        sprite: Sprite {
                            color: Color::rgb(1.0, 0.2, 0.2),
                            custom_size: Some(Vec2::new(4.0, 40.0)),
                            ..default()
                        },
                        transform: Transform::from_translation(target_transform.translation + Vec3::new(0.0, 0.0, 0.3)),
                        ..default()
                    },
                    TargetIndicator,
                ));
            }
        }
    }
}

// ==================== MISSION BRIEFING SYSTEM ====================

pub fn mission_briefing_system(
    mut commands: Commands,
    mut game_state: ResMut<GameState>,
    campaign: Res<Campaign>,
    input: Res<Input<KeyCode>>,
    briefing_query: Query<Entity, With<MissionBriefing>>,
) {
    // Only show briefing when in MissionBriefing phase
    if game_state.game_phase == GamePhase::MissionBriefing {
        // Remove any existing briefing UI
        for entity in briefing_query.iter() {
            commands.entity(entity).despawn_recursive();
        }

        // Get current mission config
        let mission_config = crate::campaign::MissionConfig::get_mission_config(&campaign.progress.current_mission);

        // Create mission briefing UI
        create_mission_briefing_ui(&mut commands, &mission_config);

        // Check for input to start mission
        if input.just_pressed(KeyCode::Space) || input.just_pressed(KeyCode::Return) {
            // Clear briefing UI
            for entity in briefing_query.iter() {
                commands.entity(entity).despawn_recursive();
            }

            // Start the actual mission
            game_state.game_phase = GamePhase::Preparation;
            play_tactical_sound("radio", &format!("Mission: {} - Begin operation!", mission_config.name));
        }
    } else {
        // Clean up any lingering briefing UI when not in briefing phase
        for entity in briefing_query.iter() {
            commands.entity(entity).despawn_recursive();
        }
    }
}

fn create_mission_briefing_ui(commands: &mut Commands, mission_config: &crate::campaign::MissionConfig) {
    // Main briefing container
    commands.spawn((
        NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            background_color: BackgroundColor(Color::rgba(0.0, 0.0, 0.0, 0.9)),
            ..default()
        },
        MissionBriefing,
    )).with_children(|parent| {
        // Mission title
        parent.spawn((
            TextBundle::from_section(
                format!("🎯 MISSION: {}", mission_config.name.to_uppercase()),
                TextStyle {
                    font_size: 48.0,
                    color: Color::rgb(1.0, 0.8, 0.0),
                    ..default()
                },
            ),
            MissionTitle,
        ));

        // Spacer
        parent.spawn(NodeBundle {
            style: Style {
                height: Val::Px(30.0),
                ..default()
            },
            ..default()
        });

        // Mission description
        parent.spawn((
            TextBundle::from_section(
                mission_config.description,
                TextStyle {
                    font_size: 24.0,
                    color: Color::WHITE,
                    ..default()
                },
            ).with_style(Style {
                max_width: Val::Px(800.0),
                ..default()
            }),
            MissionDescription,
        ));

        // Spacer
        parent.spawn(NodeBundle {
            style: Style {
                height: Val::Px(40.0),
                ..default()
            },
            ..default()
        });

        // Objectives section
        parent.spawn((
            TextBundle::from_section(
                "📋 OBJECTIVES:",
                TextStyle {
                    font_size: 28.0,
                    color: Color::rgb(0.3, 0.8, 1.0),
                    ..default()
                },
            ),
            MissionObjectives,
        ));

        // List objectives
        for (i, objective) in mission_config.objectives.iter().enumerate() {
            let objective_text = match objective {
                crate::campaign::MissionObjective::SurviveTime(time) => {
                    format!("{}. Survive for {:.0} seconds", i + 1, time)
                },
                crate::campaign::MissionObjective::DefendTarget(target) => {
                    format!("{}. Protect {}", i + 1, target)
                },
                crate::campaign::MissionObjective::EliminateEnemies(count) => {
                    format!("{}. Eliminate {} enemy units", i + 1, count)
                },
                crate::campaign::MissionObjective::ControlArea(area) => {
                    format!("{}. Control {}", i + 1, area)
                },
            };

            parent.spawn(TextBundle::from_section(
                objective_text,
                TextStyle {
                    font_size: 20.0,
                    color: Color::rgb(0.8, 0.8, 0.8),
                    ..default()
                },
            ).with_style(Style {
                margin: UiRect::top(Val::Px(8.0)),
                ..default()
            }));
        }

        // Time limit info
        if let Some(time_limit) = mission_config.time_limit {
            parent.spawn(NodeBundle {
                style: Style {
                    height: Val::Px(20.0),
                    ..default()
                },
                ..default()
            });

            parent.spawn(TextBundle::from_section(
                format!("⏰ Time Limit: {:.0} seconds", time_limit),
                TextStyle {
                    font_size: 18.0,
                    color: Color::rgb(1.0, 0.5, 0.5),
                    ..default()
                },
            ));
        }

        // Instructions
        parent.spawn(NodeBundle {
            style: Style {
                height: Val::Px(60.0),
                ..default()
            },
            ..default()
        });

        parent.spawn(TextBundle::from_section(
            "Press SPACE or ENTER to begin mission",
            TextStyle {
                font_size: 22.0,
                color: Color::rgb(0.0, 1.0, 0.0),
                ..default()
            },
        ));
    });
}

// ==================== MAIN MENU SYSTEM ====================

pub fn main_menu_system(
    mut commands: Commands,
    mut game_state: ResMut<GameState>,
    input: Res<Input<KeyCode>>,
    menu_query: Query<Entity, With<SaveLoadMenu>>,
) {
    match game_state.game_phase {
        GamePhase::MainMenu => {
            // Remove any existing menu UI
            for entity in menu_query.iter() {
                commands.entity(entity).despawn_recursive();
            }

            // Create main menu UI
            create_main_menu_ui(&mut commands);

            // Handle input
            if input.just_pressed(KeyCode::Key1) {
                game_state.game_phase = GamePhase::MissionBriefing;
                play_tactical_sound("radio", "New campaign starting!");
            } else if input.just_pressed(KeyCode::Key2) && has_save_file() {
                game_state.game_phase = GamePhase::LoadMenu;
                play_tactical_sound("radio", "Accessing saved campaigns...");
            } else if input.just_pressed(KeyCode::Key3) {
                game_state.game_phase = GamePhase::SaveMenu;
                play_tactical_sound("radio", "Opening save menu...");
            }
        },
        GamePhase::SaveMenu => {
            // Handle save menu
            if menu_query.is_empty() {
                create_save_menu_ui(&mut commands);
            }

            if input.just_pressed(KeyCode::Escape) {
                game_state.game_phase = GamePhase::MainMenu;
            } else if input.just_pressed(KeyCode::Key1) {
                // Save to slot 1
                if let Err(e) = save_game(&game_state) {
                    error!("Failed to save game: {}", e);
                    play_tactical_sound("radio", "Save failed!");
                } else {
                    play_tactical_sound("radio", "Game saved successfully!");
                    game_state.game_phase = GamePhase::MainMenu;
                }
            }
        },
        GamePhase::LoadMenu => {
            // Handle load menu
            if menu_query.is_empty() {
                create_load_menu_ui(&mut commands);
            }

            if input.just_pressed(KeyCode::Escape) {
                game_state.game_phase = GamePhase::MainMenu;
            } else if input.just_pressed(KeyCode::Key1) && has_save_file() {
                // Load from slot 1
                match load_game() {
                    Ok(save_data) => {
                        *game_state = save_data.game_state;
                        play_tactical_sound("radio", "Game loaded successfully! Resuming operation...");
                    },
                    Err(e) => {
                        error!("Failed to load game: {}", e);
                        play_tactical_sound("radio", "Load failed!");
                        game_state.game_phase = GamePhase::MainMenu;
                    }
                }
            }
        },
        _ => {
            // Clean up any lingering menu UI when not in menu phases
            for entity in menu_query.iter() {
                commands.entity(entity).despawn_recursive();
            }
        }
    }
}

fn create_main_menu_ui(commands: &mut Commands) {
    commands.spawn((
        NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            background_color: BackgroundColor(Color::rgba(0.0, 0.0, 0.0, 0.95)),
            ..default()
        },
        SaveLoadMenu,
    )).with_children(|parent| {
        // Game title
        parent.spawn(TextBundle::from_section(
            "🏛️ BATTLE OF CULIACÁN 🏛️\nEl Culiacanazo RTS",
            TextStyle {
                font_size: 56.0,
                color: Color::rgb(1.0, 0.8, 0.0),
                ..default()
            },
        ).with_style(Style {
            margin: UiRect::bottom(Val::Px(50.0)),
            ..default()
        }));

        // Menu options
        parent.spawn(TextBundle::from_section(
            "1. New Campaign",
            TextStyle {
                font_size: 32.0,
                color: Color::WHITE,
                ..default()
            },
        ).with_style(Style {
            margin: UiRect::all(Val::Px(10.0)),
            ..default()
        }));

        let load_color = if has_save_file() { Color::WHITE } else { Color::rgb(0.5, 0.5, 0.5) };
        parent.spawn(TextBundle::from_section(
            if has_save_file() { "2. Load Campaign" } else { "2. Load Campaign (No Save Found)" },
            TextStyle {
                font_size: 32.0,
                color: load_color,
                ..default()
            },
        ).with_style(Style {
            margin: UiRect::all(Val::Px(10.0)),
            ..default()
        }));

        parent.spawn(TextBundle::from_section(
            "3. Save Current Game",
            TextStyle {
                font_size: 32.0,
                color: Color::WHITE,
                ..default()
            },
        ).with_style(Style {
            margin: UiRect::all(Val::Px(10.0)),
            ..default()
        }));

        // Instructions
        parent.spawn(TextBundle::from_section(
            "Press 1-3 to select option",
            TextStyle {
                font_size: 20.0,
                color: Color::rgb(0.7, 0.7, 0.7),
                ..default()
            },
        ).with_style(Style {
            margin: UiRect::top(Val::Px(40.0)),
            ..default()
        }));
    });
}

fn create_save_menu_ui(commands: &mut Commands) {
    commands.spawn((
        NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            background_color: BackgroundColor(Color::rgba(0.0, 0.0, 0.0, 0.9)),
            ..default()
        },
        SaveLoadMenu,
    )).with_children(|parent| {
        parent.spawn(TextBundle::from_section(
            "💾 SAVE GAME",
            TextStyle {
                font_size: 48.0,
                color: Color::rgb(0.3, 0.8, 1.0),
                ..default()
            },
        ).with_style(Style {
            margin: UiRect::bottom(Val::Px(40.0)),
            ..default()
        }));

        parent.spawn(TextBundle::from_section(
            "1. Save Slot 1",
            TextStyle {
                font_size: 28.0,
                color: Color::WHITE,
                ..default()
            },
        ).with_style(Style {
            margin: UiRect::all(Val::Px(15.0)),
            ..default()
        }));

        parent.spawn(TextBundle::from_section(
            "Press 1 to save, ESC to cancel",
            TextStyle {
                font_size: 18.0,
                color: Color::rgb(0.7, 0.7, 0.7),
                ..default()
            },
        ).with_style(Style {
            margin: UiRect::top(Val::Px(30.0)),
            ..default()
        }));
    });
}

fn create_load_menu_ui(commands: &mut Commands) {
    commands.spawn((
        NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            background_color: BackgroundColor(Color::rgba(0.0, 0.0, 0.0, 0.9)),
            ..default()
        },
        SaveLoadMenu,
    )).with_children(|parent| {
        parent.spawn(TextBundle::from_section(
            "📂 LOAD GAME",
            TextStyle {
                font_size: 48.0,
                color: Color::rgb(0.3, 0.8, 1.0),
                ..default()
            },
        ).with_style(Style {
            margin: UiRect::bottom(Val::Px(40.0)),
            ..default()
        }));

        let load_text = if has_save_file() {
            "1. Load Slot 1 (Available)"
        } else {
            "1. Load Slot 1 (Empty)"
        };

        let load_color = if has_save_file() { Color::WHITE } else { Color::rgb(0.5, 0.5, 0.5) };

        parent.spawn(TextBundle::from_section(
            load_text,
            TextStyle {
                font_size: 28.0,
                color: load_color,
                ..default()
            },
        ).with_style(Style {
            margin: UiRect::all(Val::Px(15.0)),
            ..default()
        }));

        parent.spawn(TextBundle::from_section(
            "Press 1 to load, ESC to cancel",
            TextStyle {
                font_size: 18.0,
                color: Color::rgb(0.7, 0.7, 0.7),
                ..default()
            },
        ).with_style(Style {
            margin: UiRect::top(Val::Px(30.0)),
            ..default()
        }));
    });
}

// ==================== VICTORY/DEFEAT SYSTEM ====================

pub fn victory_defeat_system(
    mut commands: Commands,
    mut game_state: ResMut<GameState>,
    campaign: Res<Campaign>,
    input: Res<Input<KeyCode>>,
    result_query: Query<Entity, Or<(With<VictoryScreen>, With<DefeatScreen>)>>,
) {
    match game_state.game_phase {
        GamePhase::Victory => {
            // Remove any existing result UI
            for entity in result_query.iter() {
                commands.entity(entity).despawn_recursive();
            }

            // Create victory screen
            create_victory_screen(&mut commands, &game_state, &campaign);

            // Handle input to continue
            if input.just_pressed(KeyCode::Space) || input.just_pressed(KeyCode::Return) {
                advance_campaign_or_end(&mut game_state, &campaign);
            } else if input.just_pressed(KeyCode::Escape) {
                game_state.game_phase = GamePhase::MainMenu;
                play_tactical_sound("radio", "Returning to main menu...");
            }
        },
        GamePhase::Defeat => {
            // Remove any existing result UI
            for entity in result_query.iter() {
                commands.entity(entity).despawn_recursive();
            }

            // Create defeat screen
            create_defeat_screen(&mut commands, &game_state, &campaign);

            // Handle input to continue
            if input.just_pressed(KeyCode::Space) || input.just_pressed(KeyCode::Return) {
                // On defeat, return to main menu or retry
                game_state.game_phase = GamePhase::MainMenu;
                play_tactical_sound("radio", "Operation terminated. Regrouping...");
            } else if input.just_pressed(KeyCode::Escape) {
                game_state.game_phase = GamePhase::MainMenu;
                play_tactical_sound("radio", "Returning to main menu...");
            }
        },
        _ => {
            // Clean up any lingering result UI when not in victory/defeat phases
            for entity in result_query.iter() {
                commands.entity(entity).despawn_recursive();
            }
        }
    }
}

fn create_victory_screen(commands: &mut Commands, game_state: &GameState, campaign: &Campaign) {
    commands.spawn((
        NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            background_color: BackgroundColor(Color::rgba(0.0, 0.3, 0.0, 0.95)),
            ..default()
        },
        VictoryScreen,
    )).with_children(|parent| {
        // Victory title
        parent.spawn((
            TextBundle::from_section(
                "🏆 ¡VICTORIA! 🏆",
                TextStyle {
                    font_size: 64.0,
                    color: Color::rgb(1.0, 0.8, 0.0),
                    ..default()
                },
            ),
            MissionResultText,
        ));

        // Mission name
        let mission_config = MissionConfig::get_mission_config(&campaign.progress.current_mission);
        parent.spawn(TextBundle::from_section(
            format!("Mission: {} Complete", mission_config.name),
            TextStyle {
                font_size: 32.0,
                color: Color::WHITE,
                ..default()
            },
        ).with_style(Style {
            margin: UiRect::top(Val::Px(20.0)),
            ..default()
        }));

        // Historical context
        parent.spawn(TextBundle::from_section(
            "Historical Outcome: The Sinaloa Cartel successfully\npressured the Mexican government to release Ovidio Guzmán.\nThis event became known as 'El Culiacanazo' or 'Black Thursday'.",
            TextStyle {
                font_size: 20.0,
                color: Color::rgb(0.9, 0.9, 0.9),
                ..default()
            },
        ).with_style(Style {
            margin: UiRect::vertical(Val::Px(30.0)),
            max_width: Val::Px(800.0),
            ..default()
        }));

        // Objectives summary
        parent.spawn(TextBundle::from_section(
            "📊 MISSION OBJECTIVES:",
            TextStyle {
                font_size: 24.0,
                color: Color::rgb(0.3, 0.8, 1.0),
                ..default()
            },
        ).with_style(Style {
            margin: UiRect::top(Val::Px(20.0)),
            ..default()
        }));

        parent.spawn(TextBundle::from_section(
            get_objective_summary(campaign),
            TextStyle {
                font_size: 18.0,
                color: Color::WHITE,
                ..default()
            },
        ).with_style(Style {
            margin: UiRect::all(Val::Px(10.0)),
            ..default()
        }));

        // Score summary
        parent.spawn(TextBundle::from_section(
            format!("Final Score: {} | Time: {:.1}s",
                game_state.cartel_score,
                game_state.mission_timer
            ),
            TextStyle {
                font_size: 22.0,
                color: Color::rgb(0.0, 1.0, 0.0),
                ..default()
            },
        ).with_style(Style {
            margin: UiRect::top(Val::Px(30.0)),
            ..default()
        }));

        // Continue instructions
        parent.spawn(TextBundle::from_section(
            "Press SPACE to continue | ESC for main menu",
            TextStyle {
                font_size: 18.0,
                color: Color::rgb(0.7, 0.7, 0.7),
                ..default()
            },
        ).with_style(Style {
            margin: UiRect::top(Val::Px(40.0)),
            ..default()
        }));
    });
}

fn create_defeat_screen(commands: &mut Commands, game_state: &GameState, campaign: &Campaign) {
    commands.spawn((
        NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            background_color: BackgroundColor(Color::rgba(0.3, 0.0, 0.0, 0.95)),
            ..default()
        },
        DefeatScreen,
    )).with_children(|parent| {
        // Defeat title
        parent.spawn((
            TextBundle::from_section(
                "💀 MISIÓN FALLIDA 💀",
                TextStyle {
                    font_size: 64.0,
                    color: Color::rgb(1.0, 0.3, 0.3),
                    ..default()
                },
            ),
            MissionResultText,
        ));

        // Mission name
        let mission_config = MissionConfig::get_mission_config(&campaign.progress.current_mission);
        parent.spawn(TextBundle::from_section(
            format!("Mission: {} Failed", mission_config.name),
            TextStyle {
                font_size: 32.0,
                color: Color::WHITE,
                ..default()
            },
        ).with_style(Style {
            margin: UiRect::top(Val::Px(20.0)),
            ..default()
        }));

        // Failure context
        parent.spawn(TextBundle::from_section(
            "The government forces succeeded in their objective.\nHowever, this simulation helps understand the complex\ndynamics that led to the actual historical outcome.",
            TextStyle {
                font_size: 20.0,
                color: Color::rgb(0.9, 0.9, 0.9),
                ..default()
            },
        ).with_style(Style {
            margin: UiRect::vertical(Val::Px(30.0)),
            max_width: Val::Px(800.0),
            ..default()
        }));

        // Objectives summary
        parent.spawn(TextBundle::from_section(
            "📊 MISSION OBJECTIVES:",
            TextStyle {
                font_size: 24.0,
                color: Color::rgb(0.3, 0.8, 1.0),
                ..default()
            },
        ).with_style(Style {
            margin: UiRect::top(Val::Px(20.0)),
            ..default()
        }));

        parent.spawn(TextBundle::from_section(
            get_objective_summary(campaign),
            TextStyle {
                font_size: 18.0,
                color: Color::WHITE,
                ..default()
            },
        ).with_style(Style {
            margin: UiRect::all(Val::Px(10.0)),
            ..default()
        }));

        // Score summary
        parent.spawn(TextBundle::from_section(
            format!("Final Score: {} | Survived: {:.1}s",
                game_state.cartel_score,
                game_state.mission_timer
            ),
            TextStyle {
                font_size: 22.0,
                color: Color::rgb(1.0, 0.5, 0.5),
                ..default()
            },
        ).with_style(Style {
            margin: UiRect::top(Val::Px(30.0)),
            ..default()
        }));

        // Continue instructions
        parent.spawn(TextBundle::from_section(
            "Press SPACE to try again | ESC for main menu",
            TextStyle {
                font_size: 18.0,
                color: Color::rgb(0.7, 0.7, 0.7),
                ..default()
            },
        ).with_style(Style {
            margin: UiRect::top(Val::Px(40.0)),
            ..default()
        }));
    });
}

fn advance_campaign_or_end(game_state: &mut GameState, _campaign: &Campaign) {
    // For now, return to main menu after victory
    // In the future, this could advance to the next mission
    game_state.game_phase = GamePhase::MainMenu;
    play_tactical_sound("radio", "Mission complete. Ready for next operation...");

    // Reset mission timer for potential replay
    game_state.mission_timer = 0.0;
}

// ==================== MINIMAP SYSTEM ====================

pub fn minimap_system(
    mut commands: Commands,
    unit_query: Query<(&Transform, &Unit), Without<MiniMapIcon>>,
    minimap_icon_query: Query<(Entity, &mut Style, &MiniMapIcon), (With<MiniMapIcon>, Without<Transform>)>,
    minimap_query: Query<Entity, With<MiniMap>>,
) {
    if let Ok(minimap_entity) = minimap_query.get_single() {
        // Clear old icons
        for (entity, _, _) in minimap_icon_query.iter() {
            commands.entity(entity).despawn();
        }

        // Create new icons for all living units
        for (transform, unit) in unit_query.iter() {
            if unit.health <= 0.0 {
                continue;
            }

            // Scale world position to minimap coordinates (200x150 minimap)
            let minimap_x = (transform.translation.x / 1000.0) * 100.0 + 100.0; // Center at 100
            let minimap_y = (transform.translation.y / 750.0) * 75.0 + 75.0;   // Center at 75

            let icon_color = match unit.faction {
                Faction::Cartel => Color::RED,
                Faction::Military => Color::GREEN,
                _ => Color::WHITE,
            };

            commands.entity(minimap_entity).with_children(|parent| {
                parent.spawn((
                    NodeBundle {
                        style: Style {
                            position_type: PositionType::Absolute,
                            left: Val::Px(minimap_x),
                            top: Val::Px(minimap_y),
                            width: Val::Px(4.0),
                            height: Val::Px(4.0),
                            ..default()
                        },
                        background_color: BackgroundColor(icon_color),
                        ..default()
                    },
                    MiniMapIcon {
                        unit_type: unit.unit_type.clone(),
                        faction: unit.faction.clone(),
                    },
                ));
            });
        }
    }
}

// ==================== FORMATION MOVEMENT SYSTEM ====================

fn assign_formation_positions(
    selected_units: &[Entity],
    target_center: Vec3,
    formation_type: FormationType,
    movement_query: &mut Query<&mut Movement>,
) {
    if selected_units.is_empty() {
        return;
    }

    let unit_count = selected_units.len();
    let spacing = 60.0; // Distance between units in formation

    for (i, &unit_entity) in selected_units.iter().enumerate() {
        if let Ok(mut movement) = movement_query.get_mut(unit_entity) {
            let formation_offset = match formation_type {
                FormationType::Line => {
                    // Horizontal line formation
                    let x_offset = (i as f32 - (unit_count as f32 - 1.0) / 2.0) * spacing;
                    Vec3::new(x_offset, 0.0, 0.0)
                },
                FormationType::Circle => {
                    // Circular formation
                    let angle = (i as f32 / unit_count as f32) * 2.0 * std::f32::consts::PI;
                    let radius = spacing * (unit_count as f32 / (2.0 * std::f32::consts::PI)).max(1.0);
                    Vec3::new(angle.cos() * radius, angle.sin() * radius, 0.0)
                },
                FormationType::Wedge => {
                    // V-shaped wedge formation
                    if i == 0 {
                        Vec3::ZERO // Leader at front
                    } else {
                        let side = if i % 2 == 1 { -1.0 } else { 1.0 };
                        let row = (i + 1) / 2;
                        Vec3::new(side * spacing * 0.7, -(row as f32) * spacing * 0.5, 0.0)
                    }
                },
                FormationType::Flanking => {
                    // Split formation for flanking
                    let side = if i < unit_count / 2 { -1.0 } else { 1.0 };
                    let pos_in_side = if i < unit_count / 2 { i } else { i - unit_count / 2 };
                    Vec3::new(side * spacing * 1.5, (pos_in_side as f32) * spacing * 0.5, 0.0)
                },
                FormationType::Overwatch => {
                    // Supporting positions with good fields of fire
                    let x_offset = (i as f32 - (unit_count as f32 - 1.0) / 2.0) * spacing * 1.2;
                    Vec3::new(x_offset, spacing * 0.8, 0.0)
                },
                FormationType::Retreat => {
                    // Staggered withdrawal formation
                    let x_offset = (i as f32 - (unit_count as f32 - 1.0) / 2.0) * spacing * 0.8;
                    Vec3::new(x_offset, -(i as f32 * spacing * 0.3), 0.0)
                },
            };

            movement.target_position = Some(target_center + formation_offset);
        }
    }
}

// ==================== ATTACK TARGETING SYSTEM ====================

fn find_enemy_at_position(
    position: Vec3,
    unit_query: &Query<(Entity, &Transform, &Unit, Option<&Selected>)>
) -> Option<Entity> {
    let click_radius = 50.0; // Detection radius for clicking on units

    let mut closest_enemy = None;
    let mut closest_distance = f32::INFINITY;

    for (entity, transform, unit, _) in unit_query.iter() {
        // Only target living military units (enemies of the player-controlled cartel)
        if unit.faction != Faction::Military || unit.health <= 0.0 {
            continue;
        }

        let distance = transform.translation.distance(position);
        if distance < click_radius && distance < closest_distance {
            closest_distance = distance;
            closest_enemy = Some(entity);
        }
    }

    closest_enemy
}

fn assign_attack_targets(
    selected_units: &[Entity],
    target_enemy: Entity,
    unit_query: &mut Query<&mut Unit>,
) {
    for &unit_entity in selected_units {
        if let Ok(mut unit) = unit_query.get_mut(unit_entity) {
            unit.target = Some(target_enemy);
        }
    }
}

// ==================== ANIMATION SYSTEMS ====================

pub fn sprite_animation_system(
    mut animated_query: Query<(&mut Transform, &mut AnimatedSprite)>,
    time: Res<Time>,
) {
    for (mut transform, mut animated_sprite) in animated_query.iter_mut() {
        animated_sprite.animation_timer.tick(time.delta());

        // Pulsing scale animation
        let time_ratio = animated_sprite.animation_timer.elapsed_secs() / animated_sprite.animation_timer.duration().as_secs_f32();
        let pulse = (time_ratio * std::f32::consts::PI * 2.0).sin();
        let scale_modifier = 1.0 + pulse * animated_sprite.scale_amplitude;

        transform.scale = animated_sprite.base_scale * scale_modifier;

        // Gentle rotation
        transform.rotation = Quat::from_rotation_z(
            animated_sprite.rotation_speed * time.delta_seconds()
        ) * transform.rotation;

        // Reset timer when finished
        if animated_sprite.animation_timer.finished() {
            animated_sprite.animation_timer.reset();
        }
    }
}

pub fn movement_animation_system(
    mut movement_anim_query: Query<(&mut Transform, &mut MovementAnimation, &Movement)>,
    time: Res<Time>,
) {
    for (mut transform, mut movement_anim, movement) in movement_anim_query.iter_mut() {
        movement_anim.bob_timer.tick(time.delta());

        // Only animate when moving
        if movement.target_position.is_some() {
            let bob_time = movement_anim.bob_timer.elapsed_secs();
            let bob_offset = (bob_time * 8.0).sin() * movement_anim.bob_amplitude;
            transform.translation.y = movement_anim.base_y + bob_offset;
        } else {
            // Return to base position when not moving
            transform.translation.y = movement_anim.base_y;
        }

        // Reset timer periodically
        if movement_anim.bob_timer.finished() {
            movement_anim.bob_timer.reset();
        }
    }
}