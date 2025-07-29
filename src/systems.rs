use bevy::prelude::*;
use rand::{thread_rng, Rng};
use crate::components::*;
use crate::resources::*;
use crate::utils::{play_tactical_sound, world_to_iso};
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
    let mut combat_events = Vec::new();
    
    // Find combat pairs and calculate damage - prioritize assigned targets
    let units: Vec<_> = unit_query.iter().collect();
    for (i, (entity_a, unit_a, transform_a)) in units.iter().enumerate() {
        if unit_a.health <= 0.0 || !unit_a.attack_cooldown.finished() {
            continue;
        }
        
        // Try to attack assigned target first
        if let Some(target_entity) = unit_a.target {
            if let Some((_, target_unit, target_transform)) = units.iter()
                .find(|(entity, _, _)| *entity == target_entity) {
                
                // Check if target is valid (alive, enemy faction, in range)
                if target_unit.health > 0.0 
                    && target_unit.faction != unit_a.faction 
                    && transform_a.translation.distance(target_transform.translation) <= unit_a.range {
                    combat_events.push((*entity_a, target_entity, unit_a.damage));
                    continue; // Skip general combat for this unit
                }
            }
        }
        
        // General combat - attack nearest enemy if no specific target
        for (entity_b, unit_b, transform_b) in units.iter().skip(i + 1) {
            // Only enemies can fight
            if unit_a.faction == unit_b.faction || unit_b.health <= 0.0 {
                continue;
            }
            
            let distance = transform_a.translation.distance(transform_b.translation);
            
            // Check if units are in range to attack each other
            if distance <= unit_a.range {
                combat_events.push((*entity_a, *entity_b, unit_a.damage));
            }
            if distance <= unit_b.range && unit_b.attack_cooldown.finished() {
                combat_events.push((*entity_b, *entity_a, unit_b.damage));
            }
        }
    }
    
    // Apply combat damage and effects
    for (attacker, target, damage) in combat_events {
        // Get immutable data first
        let (attacker_transform, attacker_weapon) = if let Ok((_, unit, transform)) = unit_query.get(attacker) {
            (transform.translation, unit.equipment.weapon.clone())
        } else { continue; };
        
        let target_transform = if let Ok((_, _, transform)) = unit_query.get(target) {
            transform.translation
        } else { continue; };
        
        // Calculate damage modifier
        let damage_modifier = match attacker_weapon {
            WeaponType::HeavyMachineGun | WeaponType::RPG => 1.5,
            WeaponType::TacticalRifle | WeaponType::CartelSniperRifle | WeaponType::MilitarySniperRifle => 1.3,
            WeaponType::VehicleWeapons => 2.0,
            _ => 1.0,
        };
        
        // Apply ability effects to damage
        let mut ability_damage_modifier = 1.0;
        if let Ok(effect) = effect_query.get(attacker) {
            match effect.effect_type {
                EffectType::DamageBoost(multiplier) => {
                    ability_damage_modifier *= multiplier;
                },
                _ => {}
            }
        }
        
        let final_damage = damage * damage_modifier * ability_damage_modifier;
        
        // Update attacker cooldown and stats
        if let Ok((_, mut attacker_unit, _)) = unit_query.get_mut(attacker) {
            attacker_unit.attack_cooldown.reset();
        }
        
        // Apply damage to target (accounting for damage reduction effects)
        let target_died = if let Ok((_, mut target_unit, _)) = unit_query.get_mut(target) {
            let mut damage_reduction = 1.0;
            if let Ok(effect) = effect_query.get(target) {
                match effect.effect_type {
                    EffectType::DamageReduction(reduction) => {
                        damage_reduction *= reduction;
                    },
                    EffectType::Intimidated => {
                        damage_reduction *= 0.7; // Intimidated units take less damage
                    },
                    _ => {}
                }
            }
            
            let reduced_damage = final_damage * damage_reduction;
            target_unit.health -= reduced_damage;
            let died = target_unit.health <= 0.0;
            
            // Audio feedback
            let weapon_sound = match attacker_weapon {
                WeaponType::RPG => "explosion",
                WeaponType::VehicleWeapons => "vehicle",
                _ => "gunfire",
            };
            play_tactical_sound(weapon_sound, &format!("Combat: {} damage dealt", reduced_damage as u32));
            
            died
        } else { false };
        
        // Update attacker if target died
        if target_died {
            if let Ok((_, mut attacker_unit, _)) = unit_query.get_mut(attacker) {
                attacker_unit.kills += 1;
                attacker_unit.experience += 10;
                
                // Update veterancy level
                attacker_unit.veterancy_level = match attacker_unit.kills {
                    0..=2 => VeterancyLevel::Recruit,
                    3..=5 => VeterancyLevel::Veteran,
                    _ => VeterancyLevel::Elite,
                };
                
                play_tactical_sound("radio", &format!("{:?} gains experience from elimination", attacker_unit.unit_type));
            }
        }
        
        // Create visual effects
        spawn_damage_indicator(&mut commands, target_transform, final_damage);
        spawn_combat_particles(&mut commands, attacker_transform, target_transform);
    }
    
    // Clear invalid targets (dead units) and update attack cooldowns
    let living_entities: std::collections::HashSet<Entity> = unit_query.iter()
        .filter(|(_, unit, _)| unit.health > 0.0)
        .map(|(entity, _, _)| entity)
        .collect();
    
    for (_, mut unit, _) in unit_query.iter_mut() {
        if let Some(target) = unit.target {
            if !living_entities.contains(&target) {
                unit.target = None; // Clear dead target
            }
        }
        // Update attack cooldowns
        unit.attack_cooldown.tick(time.delta());
    }
}

fn spawn_damage_indicator(commands: &mut Commands, position: Vec3, damage: f32) {
    // Determine color and size based on damage amount
    let (color, font_size) = if damage >= 50.0 {
        (Color::rgb(1.0, 0.2, 0.2), 28.0) // High damage - large red
    } else if damage >= 25.0 {
        (Color::rgb(1.0, 0.5, 0.2), 24.0) // Medium damage - orange
    } else {
        (Color::rgb(0.9, 0.9, 0.3), 20.0) // Low damage - yellow
    };
    
    // Random offset for visual variety
    let offset_x = thread_rng().gen_range(-10.0..10.0);
    let start_pos = position + Vec3::new(offset_x, 35.0, 1.0);
    
    commands.spawn((
        Text2dBundle {
            text: Text::from_section(
                format!("-{}", damage as u32),
                TextStyle {
                    font_size,
                    color,
                    ..default()
                },
            ),
            transform: Transform::from_translation(start_pos),
            ..default()
        },
        DamageIndicator {
            lifetime: Timer::from_seconds(2.0, TimerMode::Once),
        },
        // Add velocity for floating upward animation
        ParticleEffect {
            lifetime: Timer::from_seconds(2.0, TimerMode::Once),
            velocity: Vec3::new(0.0, 30.0, 0.0), // Float upward
        },
    ));
}

fn spawn_combat_particles(commands: &mut Commands, attacker_pos: Vec3, target_pos: Vec3) {
    let direction = (target_pos - attacker_pos).normalize();
    let distance = attacker_pos.distance(target_pos);
    
    // Muzzle flash at attacker position
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(1.0, 0.9, 0.3),
                custom_size: Some(Vec2::new(12.0, 12.0)),
                ..default()
            },
            transform: Transform::from_translation(attacker_pos + Vec3::new(0.0, 0.0, 0.8)),
            ..default()
        },
        ParticleEffect {
            lifetime: Timer::from_seconds(0.15, TimerMode::Once),
            velocity: Vec3::ZERO,
        },
    ));
    
    // Bullet trail particles
    let num_particles = (distance / 20.0).clamp(3.0, 8.0) as usize;
    for i in 0..num_particles {
        let progress = i as f32 / num_particles as f32;
        let particle_pos = attacker_pos.lerp(target_pos, progress);
        
        let velocity = direction * thread_rng().gen_range(50.0..150.0) + Vec3::new(
            thread_rng().gen_range(-30.0..30.0),
            thread_rng().gen_range(-30.0..30.0),
            0.0,
        );
        
        // Vary particle colors for more visual interest
        let color = match thread_rng().gen_range(0..3) {
            0 => Color::rgb(1.0, 0.7, 0.3), // Orange
            1 => Color::rgb(0.9, 0.9, 0.3), // Yellow
            _ => Color::rgb(0.7, 0.7, 0.7), // Gray smoke
        };
        
        commands.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color,
                    custom_size: Some(Vec2::new(4.0, 4.0)),
                    ..default()
                },
                transform: Transform::from_translation(particle_pos + Vec3::new(0.0, 0.0, 0.6)),
                ..default()
            },
            ParticleEffect {
                lifetime: Timer::from_seconds(thread_rng().gen_range(0.2..0.5), TimerMode::Once),
                velocity,
            },
        ));
    }
    
    // Impact spark at target position
    for _ in 0..3 {
        let spark_velocity = Vec3::new(
            thread_rng().gen_range(-100.0..100.0),
            thread_rng().gen_range(-100.0..100.0),
            0.0,
        );
        
        commands.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: Color::rgb(1.0, 0.3, 0.1),
                    custom_size: Some(Vec2::new(6.0, 6.0)),
                    ..default()
                },
                transform: Transform::from_translation(target_pos + Vec3::new(0.0, 0.0, 0.7)),
                ..default()
            },
            ParticleEffect {
                lifetime: Timer::from_seconds(0.4, TimerMode::Once),
                velocity: spark_velocity,
            },
        ));
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

fn execute_ability_simple(
    commands: &mut Commands,
    caster_entity: Entity,
    caster_position: Vec3,
    _caster_unit: &mut Unit,
    ability_type: AbilityType,
    enemy_data: &[(Entity, Vec3, UnitType, f32)],
    _game_assets: &Res<GameAssets>,
) {
    match ability_type {
        AbilityType::BurstFire => {
            // Temporary damage boost
            commands.entity(caster_entity).insert(AbilityEffect {
                effect_type: EffectType::DamageBoost(1.5),
                duration: Timer::from_seconds(3.0, TimerMode::Once),
                strength: 1.5,
            });
            play_tactical_sound("ability", "Burst fire activated! Increased damage for 3 seconds");
        },
        AbilityType::Intimidate => {
            // Find nearby enemies and apply intimidation
            let intimidation_range = 80.0;
            for (enemy_entity, enemy_position, _, enemy_health) in enemy_data.iter() {
                let distance = caster_position.distance(*enemy_position);
                if distance <= intimidation_range && *enemy_health > 0.0 {
                    commands.entity(*enemy_entity).insert(AbilityEffect {
                        effect_type: EffectType::Intimidated,
                        duration: Timer::from_seconds(5.0, TimerMode::Once),
                        strength: 0.7, // 30% damage reduction
                    });
                }
            }
            play_tactical_sound("ability", "Intimidation used! Nearby enemies are demoralized");
        },
        AbilityType::CallBackup => {
            // Spawn a reinforcement unit near the caster
            let backup_pos = caster_position + Vec3::new(30.0, 30.0, 0.0);
            spawn_unit(commands, UnitType::Sicario, Faction::Cartel, backup_pos, _game_assets);
            play_tactical_sound("ability", "Backup called! Reinforcement unit arriving");
        },
        AbilityType::FragGrenade => {
            // Create area damage around target location
            create_explosion_effect_simple(commands, caster_position, 60.0, 40.0, enemy_data);
            play_tactical_sound("ability", "Frag grenade thrown! Area damage inflicted");
        },
        AbilityType::AirStrike => {
            // Delayed area bombardment
            for (enemy_entity, enemy_position, _, enemy_health) in enemy_data.iter() {
                let distance = caster_position.distance(*enemy_position);
                if distance <= 100.0 && *enemy_health > 0.0 {
                    // Apply delayed damage
                    commands.entity(*enemy_entity).insert(AbilityEffect {
                        effect_type: EffectType::Stunned,
                        duration: Timer::from_seconds(1.0, TimerMode::Once),
                        strength: 50.0, // Damage amount
                    });
                }
            }
            play_tactical_sound("ability", "Air strike called in! Incoming bombardment");
        },
        AbilityType::TacticalRetreat => {
            // Speed boost and damage reduction
            commands.entity(caster_entity).insert(AbilityEffect {
                effect_type: EffectType::SpeedBoost(1.8),
                duration: Timer::from_seconds(4.0, TimerMode::Once),
                strength: 1.8,
            });
            commands.entity(caster_entity).insert(AbilityEffect {
                effect_type: EffectType::DamageReduction(0.5),
                duration: Timer::from_seconds(4.0, TimerMode::Once),
                strength: 0.5,
            });
            play_tactical_sound("ability", "Tactical retreat! Speed boost and damage reduction active");
        },
        AbilityType::PrecisionShot => {
            // High-damage single shot with armor piercing
            if let Some((target_entity, _, _, _)) = enemy_data.iter()
                .find(|(_, pos, _, health)| caster_position.distance(*pos) <= 250.0 && *health > 0.0) {
                commands.entity(*target_entity).insert(AbilityEffect {
                    effect_type: EffectType::ArmorPiercing,
                    duration: Timer::from_seconds(0.1, TimerMode::Once),
                    strength: 120.0, // High damage
                });
            }
            play_tactical_sound("ability", "Precision shot! High-damage armor-piercing round fired");
        },
        AbilityType::SuppressiveFire => {
            // Area suppression effect
            let suppression_range = 120.0;
            for (enemy_entity, enemy_position, _, enemy_health) in enemy_data.iter() {
                let distance = caster_position.distance(*enemy_position);
                if distance <= suppression_range && *enemy_health > 0.0 {
                    commands.entity(*enemy_entity).insert(AbilityEffect {
                        effect_type: EffectType::Suppressed,
                        duration: Timer::from_seconds(6.0, TimerMode::Once),
                        strength: 0.6, // 40% accuracy reduction
                    });
                }
            }
            play_tactical_sound("ability", "Suppressive fire! Enemy accuracy and movement reduced");
        },
        AbilityType::FieldMedic => {
            // Heal nearby allies
            // Note: Would need ally query to implement properly, using caster for now
            commands.entity(caster_entity).insert(AbilityEffect {
                effect_type: EffectType::Healing(25.0),
                duration: Timer::from_seconds(5.0, TimerMode::Once),
                strength: 25.0,
            });
            play_tactical_sound("ability", "Field medic! Healing allies in the area");
        },
        AbilityType::TankShell => {
            // Massive area damage
            create_explosion_effect_simple(commands, caster_position, 100.0, 80.0, enemy_data);
            play_tactical_sound("ability", "Tank shell fired! Devastating area damage");
        },
        AbilityType::StrafeRun => {
            // Linear area attack
            for (enemy_entity, enemy_position, _, enemy_health) in enemy_data.iter() {
                let distance = caster_position.distance(*enemy_position);
                if distance <= 150.0 && *enemy_health > 0.0 {
                    commands.entity(*enemy_entity).insert(AbilityEffect {
                        effect_type: EffectType::ArmorPiercing,
                        duration: Timer::from_seconds(0.1, TimerMode::Once),
                        strength: 60.0,
                    });
                }
            }
            play_tactical_sound("ability", "Helicopter strafe run! Multiple targets engaged");
        },
        AbilityType::DeployBarricade => {
            // Create defensive cover
            let barricade_pos = caster_position + Vec3::new(40.0, 0.0, 0.0);
            spawn_unit(commands, UnitType::Roadblock, Faction::Military, barricade_pos, _game_assets);
            play_tactical_sound("ability", "Barricade deployed! Defensive cover established");
        },
        AbilityType::RepairVehicle => {
            // Heal nearby vehicles/allies
            commands.entity(caster_entity).insert(AbilityEffect {
                effect_type: EffectType::Healing(40.0),
                duration: Timer::from_seconds(3.0, TimerMode::Once),
                strength: 40.0,
            });
            play_tactical_sound("ability", "Repair tools active! Vehicle health restored");
        },
    }
}

fn create_explosion_effect(
    commands: &mut Commands,
    position: Vec3,
    radius: f32,
    damage: f32,
    enemy_query: &Query<(Entity, &Transform, &Unit), Without<Selected>>,
) {
    // Visual explosion effect
    for i in 0..8 {
        let angle = (i as f32 / 8.0) * std::f32::consts::PI * 2.0;
        let offset = Vec3::new(angle.cos() * 20.0, angle.sin() * 20.0, 0.0);
        
        commands.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: Color::rgb(1.0, 0.5, 0.1),
                    custom_size: Some(Vec2::new(16.0, 16.0)),
                    ..default()
                },
                transform: Transform::from_translation(position + offset + Vec3::new(0.0, 0.0, 0.8)),
                ..default()
            },
            ParticleEffect {
                lifetime: Timer::from_seconds(0.8, TimerMode::Once),
                velocity: offset.normalize() * 80.0,
            },
        ));
    }
    
    // Apply damage to enemies in range
    for (enemy_entity, enemy_transform, _) in enemy_query.iter() {
        let distance = position.distance(enemy_transform.translation);
        if distance <= radius {
            let damage_multiplier = 1.0 - (distance / radius);
            let final_damage = damage * damage_multiplier;
            
            commands.entity(enemy_entity).insert(AbilityEffect {
                effect_type: EffectType::Stunned,
                duration: Timer::from_seconds(0.1, TimerMode::Once),
                strength: final_damage,
            });
        }
    }
}

fn create_explosion_effect_simple(
    commands: &mut Commands,
    position: Vec3,
    radius: f32,
    damage: f32,
    enemy_data: &[(Entity, Vec3, UnitType, f32)],
) {
    // Visual explosion effect
    for i in 0..8 {
        let angle = (i as f32 / 8.0) * std::f32::consts::PI * 2.0;
        let offset = Vec3::new(angle.cos() * 20.0, angle.sin() * 20.0, 0.0);
        
        commands.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: Color::rgb(1.0, 0.5, 0.1),
                    custom_size: Some(Vec2::new(16.0, 16.0)),
                    ..default()
                },
                transform: Transform::from_translation(position + offset + Vec3::new(0.0, 0.0, 0.8)),
                ..default()
            },
            ParticleEffect {
                lifetime: Timer::from_seconds(0.8, TimerMode::Once),
                velocity: offset.normalize() * 80.0,
            },
        ));
    }
    
    // Apply damage to enemies in range
    for (enemy_entity, enemy_position, _, enemy_health) in enemy_data.iter() {
        let distance = position.distance(*enemy_position);
        if distance <= radius && *enemy_health > 0.0 {
            let damage_multiplier = 1.0 - (distance / radius);
            let final_damage = damage * damage_multiplier;
            
            commands.entity(*enemy_entity).insert(AbilityEffect {
                effect_type: EffectType::Stunned,
                duration: Timer::from_seconds(0.1, TimerMode::Once),
                strength: final_damage,
            });
        }
    }
}

fn get_default_ability(faction: &Faction, ability_index: usize) -> Option<AbilityType> {
    match (faction, ability_index) {
        (Faction::Cartel, 0) => Some(AbilityType::BurstFire),
        (Faction::Cartel, 1) => Some(AbilityType::Intimidate),
        (Faction::Military, 0) => Some(AbilityType::FragGrenade),
        (Faction::Military, 1) => Some(AbilityType::TacticalRetreat),
        _ => None,
    }
}

fn get_ability_cooldown(ability_type: &AbilityType) -> f32 {
    match ability_type {
        AbilityType::BurstFire => 8.0,
        AbilityType::Intimidate => 12.0,
        AbilityType::CallBackup => 20.0,
        AbilityType::FragGrenade => 10.0,
        AbilityType::AirStrike => 15.0,
        AbilityType::TacticalRetreat => 18.0,
        AbilityType::PrecisionShot => 8.0,
        AbilityType::SuppressiveFire => 12.0,
        AbilityType::FieldMedic => 6.0,
        AbilityType::TankShell => 15.0,
        AbilityType::StrafeRun => 20.0,
        AbilityType::DeployBarricade => 25.0,
        AbilityType::RepairVehicle => 10.0,
    }
}

fn get_ability_range(ability_type: &AbilityType) -> f32 {
    match ability_type {
        AbilityType::BurstFire => 0.0, // Self-target
        AbilityType::Intimidate => 80.0,
        AbilityType::CallBackup => 0.0, // Self-target
        AbilityType::FragGrenade => 120.0,
        AbilityType::AirStrike => 150.0,
        AbilityType::TacticalRetreat => 0.0, // Self-target
        AbilityType::PrecisionShot => 300.0,
        AbilityType::SuppressiveFire => 160.0,
        AbilityType::FieldMedic => 100.0,
        AbilityType::TankShell => 250.0,
        AbilityType::StrafeRun => 200.0,
        AbilityType::DeployBarricade => 50.0,
        AbilityType::RepairVehicle => 80.0,
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