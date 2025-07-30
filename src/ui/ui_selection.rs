use crate::components::*;
use crate::utils::play_tactical_sound;
use bevy::ecs::system::ParamSet;
use bevy::prelude::*;

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
                if !keyboard_input.pressed(KeyCode::ShiftLeft)
                    && !keyboard_input.pressed(KeyCode::ShiftRight)
                {
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
                            assign_attack_targets(
                                &selected_units,
                                enemy_entity,
                                &mut unit_queries.p1(),
                            );
                            play_tactical_sound(
                                "radio",
                                &format!("{} units ordered to attack target", selected_units.len()),
                            );
                        } else {
                            // Movement command: formation movement
                            let formation_type = if keyboard_input.pressed(KeyCode::ControlLeft) {
                                FormationType::Wedge
                            } else if keyboard_input.pressed(KeyCode::AltLeft) {
                                FormationType::Circle
                            } else {
                                FormationType::Line
                            };

                            assign_formation_positions(
                                &selected_units,
                                target_pos,
                                formation_type.clone(),
                                &mut movement_query,
                            );
                            play_tactical_sound(
                                "movement",
                                &format!(
                                    "{} units moving in {:?} formation",
                                    selected_units.len(),
                                    formation_type
                                ),
                            );
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
                    color: Color::rgba(
                        selected.selection_color.r(),
                        selected.selection_color.g(),
                        selected.selection_color.b(),
                        0.6,
                    ),
                    custom_size: Some(Vec2::new(48.0, 48.0)),
                    ..default()
                },
                transform: Transform::from_translation(
                    transform.translation + Vec3::new(0.0, 0.0, -0.1),
                ),
                ..default()
            },
            SelectionIndicator,
        ));

        // Inner selection ring (solid)
        commands.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: Color::rgba(
                        selected.selection_color.r(),
                        selected.selection_color.g(),
                        selected.selection_color.b(),
                        0.8,
                    ),
                    custom_size: Some(Vec2::new(40.0, 2.0)), // Thin ring
                    ..default()
                },
                transform: Transform::from_translation(
                    transform.translation + Vec3::new(0.0, -25.0, 0.1),
                ),
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
                    transform: Transform::from_translation(
                        transform.translation + Vec3::new(x, y, 0.2),
                    ),
                    ..default()
                },
                SelectionIndicator,
            ));
        }
    }
}

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
                        transform: Transform::from_translation(
                            target_transform.translation + Vec3::new(0.0, 0.0, 0.3),
                        ),
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
                        transform: Transform::from_translation(
                            target_transform.translation + Vec3::new(0.0, 0.0, 0.3),
                        ),
                        ..default()
                    },
                    TargetIndicator,
                ));
            }
        }
    }
}

// ==================== HELPER FUNCTIONS ====================

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
                }
                FormationType::Circle => {
                    // Circular formation
                    let angle = (i as f32 / unit_count as f32) * 2.0 * std::f32::consts::PI;
                    let radius =
                        spacing * (unit_count as f32 / (2.0 * std::f32::consts::PI)).max(1.0);
                    Vec3::new(angle.cos() * radius, angle.sin() * radius, 0.0)
                }
                FormationType::Wedge => {
                    // V-shaped wedge formation
                    if i == 0 {
                        Vec3::ZERO // Leader at front
                    } else {
                        let side = if i % 2 == 1 { -1.0 } else { 1.0 };
                        let row = (i + 1) / 2;
                        Vec3::new(side * spacing * 0.7, -(row as f32) * spacing * 0.5, 0.0)
                    }
                }
                FormationType::Flanking => {
                    // Split formation for flanking
                    let side = if i < unit_count / 2 { -1.0 } else { 1.0 };
                    let pos_in_side = if i < unit_count / 2 {
                        i
                    } else {
                        i - unit_count / 2
                    };
                    Vec3::new(
                        side * spacing * 1.5,
                        (pos_in_side as f32) * spacing * 0.5,
                        0.0,
                    )
                }
                FormationType::Overwatch => {
                    // Supporting positions with good fields of fire
                    let x_offset = (i as f32 - (unit_count as f32 - 1.0) / 2.0) * spacing * 1.2;
                    Vec3::new(x_offset, spacing * 0.8, 0.0)
                }
                FormationType::Retreat => {
                    // Staggered withdrawal formation
                    let x_offset = (i as f32 - (unit_count as f32 - 1.0) / 2.0) * spacing * 0.8;
                    Vec3::new(x_offset, -(i as f32 * spacing * 0.3), 0.0)
                }
            };

            movement.target_position = Some(target_center + formation_offset);
        }
    }
}

fn find_enemy_at_position(
    position: Vec3,
    unit_query: &Query<(Entity, &Transform, &Unit, Option<&Selected>)>,
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
