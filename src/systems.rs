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
    time: Res<Time>,
) {
    let mut combat_events = Vec::new();
    
    // Find combat pairs and calculate damage
    let units: Vec<_> = unit_query.iter().collect();
    for (i, (entity_a, unit_a, transform_a)) in units.iter().enumerate() {
        for (entity_b, unit_b, transform_b) in units.iter().skip(i + 1) {
            // Only enemies can fight
            if unit_a.faction == unit_b.faction || unit_a.health <= 0.0 || unit_b.health <= 0.0 {
                continue;
            }
            
            let distance = transform_a.translation.distance(transform_b.translation);
            
            // Check if units are in range to attack each other
            if distance <= unit_a.range && unit_a.attack_cooldown.finished() {
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
            WeaponType::TacticalRifle | WeaponType::SniperRifle => 1.3,
            WeaponType::VehicleWeapons => 2.0,
            _ => 1.0,
        };
        
        let final_damage = damage * damage_modifier;
        
        // Update attacker cooldown and stats
        if let Ok((_, mut attacker_unit, _)) = unit_query.get_mut(attacker) {
            attacker_unit.attack_cooldown.reset();
        }
        
        // Apply damage to target
        let target_died = if let Ok((_, mut target_unit, _)) = unit_query.get_mut(target) {
            target_unit.health -= final_damage;
            let died = target_unit.health <= 0.0;
            
            // Audio feedback
            let weapon_sound = match attacker_weapon {
                WeaponType::RPG => "explosion",
                WeaponType::VehicleWeapons => "vehicle",
                _ => "gunfire",
            };
            play_tactical_sound(weapon_sound, &format!("Combat: {} damage dealt", final_damage as u32));
            
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
    
    // Update attack cooldowns
    for (_, mut unit, _) in unit_query.iter_mut() {
        unit.attack_cooldown.tick(time.delta());
    }
}

fn spawn_damage_indicator(commands: &mut Commands, position: Vec3, damage: f32) {
    commands.spawn((
        Text2dBundle {
            text: Text::from_section(
                format!("-{}", damage as u32),
                TextStyle {
                    font_size: 20.0,
                    color: Color::RED,
                    ..default()
                },
            ),
            transform: Transform::from_translation(position + Vec3::new(0.0, 40.0, 1.0)),
            ..default()
        },
        DamageIndicator {
            lifetime: Timer::from_seconds(1.5, TimerMode::Once),
        },
    ));
}

fn spawn_combat_particles(commands: &mut Commands, attacker_pos: Vec3, target_pos: Vec3) {
    let direction = (target_pos - attacker_pos).normalize();
    
    for _ in 0..5 {
        let velocity = direction * 200.0 + Vec3::new(
            thread_rng().gen_range(-50.0..50.0),
            thread_rng().gen_range(-50.0..50.0),
            0.0,
        );
        
        commands.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: Color::ORANGE,
                    custom_size: Some(Vec2::new(3.0, 3.0)),
                    ..default()
                },
                transform: Transform::from_translation(attacker_pos + Vec3::new(0.0, 0.0, 0.5)),
                ..default()
            },
            ParticleEffect {
                lifetime: Timer::from_seconds(0.3, TimerMode::Once),
                velocity,
            },
        ));
    }
}

// ==================== END OF SYSTEMS ====================