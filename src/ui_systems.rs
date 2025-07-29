use bevy::prelude::*;
use crate::components::*;
use crate::resources::*;
use crate::utils::play_tactical_sound;

// ==================== UI UPDATE SYSTEMS ====================

pub fn ui_update_system(
    game_state: Res<GameState>,
    unit_query: Query<&Unit>,
    mut status_query: Query<&mut Text, (With<StatusText>, Without<WaveText>, Without<ScoreText>)>,
    mut wave_query: Query<&mut Text, (With<WaveText>, Without<StatusText>, Without<ScoreText>)>,
    mut score_query: Query<&mut Text, (With<ScoreText>, Without<StatusText>, Without<WaveText>)>,
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
                GamePhase::Preparation => "🔄 Phase: Preparation",
                GamePhase::InitialRaid => "⚔️ Phase: Initial Raid",
                GamePhase::BlockConvoy => "🚧 Phase: Block Convoy",
                GamePhase::ApplyPressure => "🔥 Phase: Apply Pressure",
                GamePhase::HoldTheLine => "🛡️ Phase: Hold The Line",
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
    mut damage_query: Query<(Entity, &mut Transform, &mut DamageIndicator)>,
    time: Res<Time>,
) {
    for (entity, mut transform, mut indicator) in damage_query.iter_mut() {
        indicator.lifetime.tick(time.delta());
        
        // Move damage indicator upward
        transform.translation.y += 30.0 * time.delta_seconds();
        
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
    unit_query: Query<(Entity, &Transform, &Unit, Option<&Selected>)>,
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
                
                for (entity, transform, unit, selected) in unit_query.iter() {
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
    
    // Handle right-click movement commands
    if mouse_button_input.just_pressed(MouseButton::Right) {
        if let Ok((camera, camera_transform)) = camera_query.get_single() {
            if let Some(cursor_pos) = window.cursor_position() {
            if let Some(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_pos) {
                    let target_pos = Vec3::new(world_pos.x, world_pos.y, 0.0);
                    
                    // Move all selected units to target position
                    for selected_entity in selected_query.iter() {
                        if let Ok(mut movement) = movement_query.get_mut(selected_entity) {
                            movement.target_position = Some(target_pos);
                            
                            // Audio feedback
                            play_tactical_sound("movement", "Units moving to new position");
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
    
    // Create new indicators for selected units
    for (_, transform, selected) in selected_query.iter() {
        commands.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: selected.selection_color,
                    custom_size: Some(Vec2::new(60.0, 60.0)),
                    ..default()
                },
                transform: Transform::from_translation(transform.translation + Vec3::new(0.0, 0.0, -0.1)),
                ..default()
            },
            SelectionIndicator,
        ));
    }
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