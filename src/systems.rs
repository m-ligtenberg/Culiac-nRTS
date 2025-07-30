use bevy::prelude::*;
use crate::components::*;
use crate::resources::*;
use crate::utils::{play_tactical_sound, world_to_iso, find_combat_pairs, apply_combat_damage, clear_invalid_targets, get_default_ability, get_ability_cooldown, get_ability_range, execute_ability_simple};
use crate::spawners::{spawn_unit, spawn_health_bar};

// ==================== SETUP SYSTEMS ====================

pub fn setup_assets(
    mut commands: Commands, 
    asset_server: Res<AssetServer>
) {
    info!("🔧 Loading sprite assets...");
    // Load individual sprite files with proper error handling
    let assets = GameAssets {
        sicario_sprite: asset_server.load("sprites/units/sicario.png"),
        enforcer_sprite: asset_server.load("sprites/units/enforcer.png"),
        ovidio_sprite: asset_server.load("sprites/units/ovidio.png"),
        soldier_sprite: asset_server.load("sprites/units/soldier.png"),
        special_forces_sprite: asset_server.load("sprites/units/special_forces.png"),
        vehicle_sprite: asset_server.load("sprites/units/vehicle.png"),
        roadblock_sprite: asset_server.load("sprites/units/roadblock.png"),
        safehouse_sprite: asset_server.load("sprites/units/safehouse.png"),
        _health_bar_bg: Handle::default(),
        _health_bar_fill: Handle::default(),
        _main_font: Handle::default(),
        _gunshot_sound: Handle::default(),
        _explosion_sound: Handle::default(),
        _radio_chatter: Handle::default(),
    };
    
    commands.insert_resource(assets);
    info!("✅ Assets loaded successfully!");
}

pub fn setup_ui(mut commands: Commands, _asset_server: Res<AssetServer>) {
    // Add a background to make sprites visible
    commands.spawn(SpriteBundle {
        sprite: Sprite {
            color: Color::rgb(0.2, 0.3, 0.2), // Dark green background
            custom_size: Some(Vec2::new(2000.0, 1500.0)),
            ..default()
        },
        transform: Transform::from_translation(Vec3::new(0.0, 0.0, -10.0)),
        ..default()
    });
    
    // Camera setup with better positioning for isometric view
    commands.spawn((
        Camera2dBundle {
            transform: Transform::from_xyz(0.0, 0.0, 999.9)
                .with_scale(Vec3::splat(1.5)), // Zoom out more to see the battlefield
            ..default()
        },
        IsometricCamera {
            pan_speed: 300.0,
            zoom_speed: 0.1,
            min_zoom: 0.5,
            max_zoom: 3.0,
        },
    ));
    
    // Minimap
    commands.spawn((
        NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                right: Val::Px(20.0),
                top: Val::Px(20.0),
                width: Val::Px(200.0),
                height: Val::Px(150.0),
                border: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            background_color: BackgroundColor(Color::rgba(0.0, 0.0, 0.0, 0.7)),
            border_color: BorderColor(Color::WHITE),
            ..default()
        },
        MiniMap,
    ));
    
    // Main UI Container
    commands.spawn((
        NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                left: Val::Px(20.0),
                top: Val::Px(20.0),
                width: Val::Px(300.0),
                height: Val::Px(200.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(10.0)),
                ..default()
            },
            background_color: BackgroundColor(Color::rgba(0.0, 0.0, 0.0, 0.7)),
            ..default()
        },
        UIElement,
    )).with_children(|parent| {
        // Status text
        parent.spawn((
            TextBundle::from_section(
                "Mission Status: Initializing...",
                TextStyle {
                    font_size: 18.0,
                    color: Color::WHITE,
                    ..default()
                },
            ),
            StatusText,
        ));
        
        // Wave information
        parent.spawn((
            TextBundle::from_section(
                "Wave: 0 - Preparing...",
                TextStyle {
                    font_size: 16.0,
                    color: Color::YELLOW,
                    ..default()
                },
            ),
            WaveText,
        ));
        
        // Score display
        parent.spawn((
            TextBundle::from_section(
                "Score: Cartel 0 - Military 0",
                TextStyle {
                    font_size: 16.0,
                    color: Color::CYAN,
                    ..default()
                },
            ),
            ScoreText,
        ));
        
        // Difficulty display
        parent.spawn((
            TextBundle::from_section(
                "Difficulty: 1.0 (AUTO) | Performance: 50%\nD=Toggle | F1-F4=Set Level",
                TextStyle {
                    font_size: 14.0,
                    color: Color::ORANGE,
                    ..default()
                },
            ),
            DifficultyDisplay,
        ));
    });
    
    info!("✅ UI elements created successfully!");
}

pub fn setup_game(
    mut commands: Commands,
    game_assets: Res<GameAssets>,
) {
    info!("🎮 Initializing Battle of Culiacán simulation...");
    
    // Spawn Ovidio (High Value Target) at center for visibility
    spawn_ovidio(&mut commands, Vec3::new(0.0, 0.0, 0.0), &game_assets);
    
    // Spawn initial cartel defenders around the center
    for i in 0..3 {
        spawn_unit(&mut commands, UnitType::Sicario, Faction::Cartel, 
                   Vec3::new(-100.0 + i as f32 * 100.0, -50.0, 0.0), 
                   &game_assets);
    }
    
    // Spawn safehouse objective with enhanced graphics
    let safehouse_pos = Vec3::new(0.0, 100.0, 0.0);
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(0.6, 0.4, 0.2),
                custom_size: Some(Vec2::new(64.0, 64.0)),
                ..default()
            },
            texture: game_assets.safehouse_sprite.clone(),
            transform: Transform::from_translation(safehouse_pos),
            ..default()
        },
        Objective {
            objective_type: ObjectiveType::Safehouse,
            _position: safehouse_pos,
            _radius: 50.0,
            _health: 100.0,
        },
    ));
    
    // Wave spawner
    commands.spawn(WaveSpawner {
        next_wave_timer: Timer::from_seconds(10.0, TimerMode::Repeating),
        wave_number: 0,
        units_in_wave: 2,
    });
    
    // Mark setup as complete
    commands.insert_resource(GameSetupComplete);
    
    play_tactical_sound("radio", "Command: Operation initiated. Ovidio's location confirmed. All units, hold your positions!");
    info!("✅ Game setup completed! Press SPACE for roadblocks, R for reinforcements, ESC to end.");
}

fn spawn_ovidio(commands: &mut Commands, position: Vec3, game_assets: &Res<GameAssets>) {
    let entity = commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                custom_size: Some(Vec2::new(48.0, 48.0)),
                ..default()
            },
            texture: game_assets.ovidio_sprite.clone(),
            transform: Transform::from_translation(position),
            ..default()
        },
        Unit {
            health: 200.0,
            max_health: 200.0,
            faction: Faction::Cartel,
            unit_type: UnitType::Ovidio,
            damage: 35.0,
            range: 160.0,
            movement_speed: 60.0,
            target: None,
            attack_cooldown: Timer::from_seconds(0.8, TimerMode::Once),
            experience: 0,
            kills: 0,
            veterancy_level: VeterancyLevel::Elite,
            equipment: Equipment {
                weapon: WeaponType::AssaultRifle,
                armor: ArmorType::HeavyArmor,
                upgrades: vec![UpgradeType::ScopedSight, UpgradeType::ReinforcedArmor],
            },
        },
        Movement {
            target_position: None,
            speed: 60.0,
        },
    )).id();
    
    // Add health bar for Ovidio
    spawn_health_bar(commands, entity, position);
}

// ==================== PATHFINDING SYSTEM ====================

pub fn pathfinding_system(
    mut unit_query: Query<(&mut Transform, &mut Movement, &mut PathfindingAgent, &Unit)>,
    obstacle_query: Query<&Transform, (With<Obstacle>, Without<Unit>)>,
    other_units_query: Query<&Transform, (With<Unit>, Without<PathfindingAgent>)>,
    time: Res<Time>,
) {
    for (mut transform, mut movement, mut pathfinding, unit) in unit_query.iter_mut() {
        pathfinding.stuck_timer += time.delta_seconds();
        
        if let Some(target_pos) = movement.target_position {
            let current_pos = transform.translation;
            
            // Generate simple path if needed
            if pathfinding.path.is_empty() || pathfinding.current_waypoint >= pathfinding.path.len() {
                pathfinding.path = generate_simple_path(current_pos, target_pos, &obstacle_query);
                pathfinding.current_waypoint = 0;
                pathfinding.stuck_timer = 0.0;
            }
            
            // Follow current waypoint
            if pathfinding.current_waypoint < pathfinding.path.len() {
                let waypoint = pathfinding.path[pathfinding.current_waypoint];
                let direction = (waypoint - current_pos).normalize_or_zero();
                
                // Apply obstacle avoidance
                let avoidance_force = calculate_avoidance_force(
                    current_pos,
                    direction,
                    pathfinding.avoidance_radius,
                    &obstacle_query,
                    &other_units_query,
                );
                
                let final_direction = (direction + avoidance_force * 0.5).normalize_or_zero();
                let move_delta = final_direction * unit.movement_speed * time.delta_seconds();
                
                // Check if reached waypoint
                if current_pos.distance(waypoint) < 10.0 {
                    pathfinding.current_waypoint += 1;
                    pathfinding.stuck_timer = 0.0;
                }
                
                // Check if stuck
                if pathfinding.stuck_timer > 2.0 {
                    // Regenerate path or find alternate route
                    pathfinding.path.clear();
                    pathfinding.stuck_timer = 0.0;
                }
                
                transform.translation += move_delta;
            } else {
                // Reached final destination
                movement.target_position = None;
                pathfinding.path.clear();
                pathfinding.current_waypoint = 0;
            }
        }
    }
}

fn generate_simple_path(start: Vec3, end: Vec3, obstacle_query: &Query<&Transform, (With<Obstacle>, Without<Unit>)>) -> Vec<Vec3> {
    let mut path = Vec::new();
    
    // Simple straight-line path with basic obstacle checking
    let direction = (end - start).normalize_or_zero();
    let distance = start.distance(end);
    let step_size = 50.0;
    let steps = (distance / step_size).ceil() as usize;
    
    for i in 1..=steps {
        let t = i as f32 / steps as f32;
        let mut point = start.lerp(end, t);
        
        // Basic obstacle avoidance - offset points near obstacles
        for obstacle_transform in obstacle_query.iter() {
            let obstacle_pos = obstacle_transform.translation;
            let dist_to_obstacle = point.distance(obstacle_pos);
            
            if dist_to_obstacle < 60.0 {
                // Offset perpendicular to line
                let perpendicular = Vec3::new(-direction.y, direction.x, 0.0);
                let offset_direction = if obstacle_pos.dot(perpendicular) > 0.0 { -1.0 } else { 1.0 };
                point += perpendicular * offset_direction * 40.0;
            }
        }
        
        path.push(point);
    }
    
    if path.is_empty() {
        path.push(end);
    }
    
    path
}

fn calculate_avoidance_force(
    position: Vec3,
    _desired_direction: Vec3,
    avoidance_radius: f32,
    obstacle_query: &Query<&Transform, (With<Obstacle>, Without<Unit>)>,
    other_units_query: &Query<&Transform, (With<Unit>, Without<PathfindingAgent>)>,
) -> Vec3 {
    let mut avoidance_force = Vec3::ZERO;
    
    // Avoid obstacles
    for obstacle_transform in obstacle_query.iter() {
        let obstacle_pos = obstacle_transform.translation;
        let distance = position.distance(obstacle_pos);
        
        if distance < avoidance_radius && distance > 0.0 {
            let away_direction = (position - obstacle_pos).normalize_or_zero();
            let strength = (avoidance_radius - distance) / avoidance_radius;
            avoidance_force += away_direction * strength * 2.0;
        }
    }
    
    // Avoid other units
    for other_transform in other_units_query.iter() {
        let other_pos = other_transform.translation;
        let distance = position.distance(other_pos);
        
        if distance < avoidance_radius * 0.7 && distance > 0.0 {
            let away_direction = (position - other_pos).normalize_or_zero();
            let strength = (avoidance_radius * 0.7 - distance) / (avoidance_radius * 0.7);
            avoidance_force += away_direction * strength;
        }
    }
    
    avoidance_force
}

// ==================== CORE GAME SYSTEMS ====================

pub fn movement_system(
    time: Res<Time>,
    mut unit_query: Query<(&mut Transform, &Movement, &Unit)>,
) {
    for (mut transform, movement, unit) in unit_query.iter_mut() {
        if let Some(target_pos) = movement.target_position {
            let current_pos = transform.translation;
            let direction = (target_pos - current_pos).normalize();
            let move_delta = direction * unit.movement_speed * time.delta_seconds();
            
            // Check if we're close enough to the target
            if current_pos.distance(target_pos) > 5.0 {
                let new_pos = current_pos + move_delta;
                // Apply isometric transformation
                transform.translation = world_to_iso(new_pos);
            }
        }
    }
}

pub fn combat_system(
    mut commands: Commands,
    mut unit_query: Query<(Entity, &mut Unit, &Transform)>,
    effect_query: Query<&AbilityEffect>,
    time: Res<Time>,
) {
    // Find combat pairs and calculate damage - prioritize assigned targets
    let units: Vec<_> = unit_query.iter().collect();
    let combat_events = find_combat_pairs(&units);
    
    // Apply combat damage and effects
    for (attacker, target, damage) in combat_events {
        apply_combat_damage(&mut commands, attacker, target, damage, &mut unit_query, &effect_query);
    }
    
    // Clear invalid targets (dead units) and update attack cooldowns
    clear_invalid_targets(&mut unit_query);
    
    for (_, mut unit, _) in unit_query.iter_mut() {
        // Update attack cooldowns
        unit.attack_cooldown.tick(time.delta());
    }
}


// ==================== ABILITY SYSTEM ====================

pub fn ability_system(
    mut commands: Commands,
    input: Res<Input<KeyCode>>,
    mut unit_queries: ParamSet<(
        Query<(Entity, &Transform, &mut Unit, Option<&mut UnitAbility>)>,
        Query<(Entity, &Transform, &Unit), Without<Selected>>,
    )>,
    selected_query: Query<Entity, With<Selected>>,
    time: Res<Time>,
    game_assets: Res<GameAssets>,
) {
    // Update ability cooldowns
    for (_, _, _, ability) in unit_queries.p0().iter_mut() {
        if let Some(mut ability) = ability {
            ability.cooldown.tick(time.delta());
        }
    }
    
    // Handle ability activation keys
    if input.just_pressed(KeyCode::Q) {
        activate_ability_for_selected(&mut commands, &selected_query, &mut unit_queries, 0, &game_assets);
    }
    if input.just_pressed(KeyCode::E) {
        activate_ability_for_selected(&mut commands, &selected_query, &mut unit_queries, 1, &game_assets);
    }
}

fn activate_ability_for_selected(
    commands: &mut Commands,
    selected_query: &Query<Entity, With<Selected>>,
    unit_queries: &mut ParamSet<(
        Query<(Entity, &Transform, &mut Unit, Option<&mut UnitAbility>)>,
        Query<(Entity, &Transform, &Unit), Without<Selected>>,
    )>,
    ability_index: usize,
    game_assets: &Res<GameAssets>,
) {
    // Collect enemy data first
    let enemy_data: Vec<(Entity, Vec3, UnitType, f32)> = unit_queries.p1().iter()
        .map(|(entity, transform, unit)| (entity, transform.translation, unit.unit_type.clone(), unit.health))
        .collect();
    
    for selected_entity in selected_query.iter() {
        if let Ok((entity, transform, mut unit, ability)) = unit_queries.p0().get_mut(selected_entity) {
            if let Some(mut ability) = ability {
                if ability.cooldown.finished() {
                    let ability_type = ability.ability_type.clone();
                    execute_ability_simple(commands, entity, transform.translation, &mut unit, ability_type, &enemy_data, game_assets);
                    ability.cooldown.reset();
                }
            } else {
                // Give units default abilities based on faction
                let default_ability = get_default_ability(&unit.faction, ability_index);
                if let Some(ability_type) = default_ability {
                    commands.entity(entity).insert(UnitAbility {
                        ability_type: ability_type.clone(),
                        cooldown: Timer::from_seconds(get_ability_cooldown(&ability_type), TimerMode::Once),
                        range: get_ability_range(&ability_type),
                        energy_cost: 10,
                    });
                    execute_ability_simple(commands, entity, transform.translation, &mut unit, ability_type, &enemy_data, game_assets);
                }
            }
        }
    }
}



pub fn ability_effect_system(
    mut commands: Commands,
    mut effect_query: Query<(Entity, &mut Unit, &mut AbilityEffect)>,
    time: Res<Time>,
) {
    for (entity, mut unit, mut effect) in effect_query.iter_mut() {
        effect.duration.tick(time.delta());
        
        // Apply effect modifications
        match effect.effect_type {
            EffectType::DamageBoost(_multiplier) => {
                // This would be applied during combat calculations
            },
            EffectType::SpeedBoost(_multiplier) => {
                // This would be applied during movement calculations
            },
            EffectType::DamageReduction(_reduction) => {
                // This would be applied during damage calculations
            },
            EffectType::Stunned => {
                // Apply instant damage if this is damage effect
                if effect.strength > 0.0 {
                    unit.health -= effect.strength;
                    effect.strength = 0.0; // Prevent multiple applications
                }
            },
            EffectType::Intimidated => {
                // Effect applied during combat
            },
            EffectType::Healing(amount) => {
                // Apply healing over time
                let heal_amount = amount * time.delta_seconds();
                unit.health = (unit.health + heal_amount).min(unit.max_health);
            },
            EffectType::Suppressed => {
                // Reduce movement and accuracy - applied during movement/combat
            },
            EffectType::ArmorPiercing => {
                // Apply instant damage bypassing armor
                if effect.strength > 0.0 {
                    unit.health -= effect.strength;
                    effect.strength = 0.0; // Prevent multiple applications
                }
            },
            EffectType::AerialView => {
                // Enhanced detection range - applied in detection systems
            },
            EffectType::Fortified => {
                // Damage reduction bonus - applied during damage calculations
            },
        }
        
        // Remove expired effects
        if effect.duration.finished() {
            commands.entity(entity).remove::<AbilityEffect>();
        }
    }
}

// ==================== END OF SYSTEMS ====================