// ==================== BATTLE OF CULIACÁN RTS GAME ====================
// Historical RTS simulation based on the events of October 17, 2019
// Built with Rust and Bevy Engine
// 
// This game simulates the urban warfare that unfolded during the failed 
// attempt to capture Ovidio Guzmán López in Culiacán, Mexico.
// =====================================================================

use bevy::prelude::*;
use bevy_kira_audio::prelude::{Audio as KiraAudio, AudioSource as KiraAudioSource, AudioPlugin as KiraAudioPlugin};
use rand::{thread_rng, Rng};
use std::time::Duration;

// ==================== AUDIO SYSTEM ====================

// For now using procedural audio through console logging
// Future: Real audio files with Handle<KiraAudioSource>

// ==================== ISOMETRIC SYSTEM ====================

#[derive(Component)]
struct IsometricCamera {
    pan_speed: f32,
    zoom_speed: f32,
    min_zoom: f32,
    max_zoom: f32,
}

// Isometric transformation helper function
fn world_to_iso(world_pos: Vec3) -> Vec3 {
    let x = (world_pos.x - world_pos.y) * 0.5; // Moderate isometric angle
    let y = (world_pos.x + world_pos.y) * 0.25; // Better perspective depth
    Vec3::new(x, y, world_pos.z)
}

// ==================== COMPONENTS ====================

#[derive(Component, Clone)]
struct Unit {
    health: f32,
    max_health: f32,
    faction: Faction,
    unit_type: UnitType,
    damage: f32,
    range: f32,
    movement_speed: f32,
    target: Option<Entity>,
    attack_cooldown: Timer,
}

#[derive(Component)]
struct Movement {
    target_position: Option<Vec3>,
    speed: f32,
}

#[derive(Component)]
struct HealthBar {
    owner: Entity,
    offset: Vec3,
}

#[derive(Component)]
struct DamageIndicator {
    lifetime: Timer,
}

#[derive(Component)]
struct Selected {
    selection_color: Color,
}

#[derive(Component)]
struct ParticleEffect {
    lifetime: Timer,
    velocity: Vec3,
}

#[derive(Component)]
struct WaveSpawner {
    next_wave_timer: Timer,
    wave_number: u32,
    units_in_wave: u32,
}

#[derive(Component)]
struct Objective {
    objective_type: ObjectiveType,
    _position: Vec3,
    _radius: f32,
    _health: f32,
}

#[derive(Component)]
struct UIElement;

#[derive(Component)]
struct StatusText;

#[derive(Component)]
struct WaveText;

#[derive(Component)]
struct ScoreText;

// ==================== ENUMS & TYPES ====================

#[derive(Clone, PartialEq, Debug)]
#[allow(dead_code)]
enum Faction {
    Cartel,
    Military,
    Civilian,
}

#[derive(Clone, PartialEq, Debug)]
enum UnitType {
    // Cartel units
    Sicario,
    Enforcer,
    Roadblock,
    // Military units  
    Soldier,
    SpecialForces,
    Vehicle,
    // Special
    Ovidio, // High value target
}

#[derive(Clone, PartialEq, Debug)]
#[allow(dead_code)]
enum ObjectiveType {
    Safehouse,      // Cartel must defend
    ExtractionPoint, // Military tries to reach
    Checkpoint,     // Control points
}

// ==================== RESOURCES ====================

#[derive(Resource)]
struct GameSetupComplete;

#[derive(Resource)]
struct GameAssets {
    // Individual sprite handles
    sicario_sprite: Handle<Image>,
    enforcer_sprite: Handle<Image>,
    ovidio_sprite: Handle<Image>,
    soldier_sprite: Handle<Image>,
    special_forces_sprite: Handle<Image>,
    vehicle_sprite: Handle<Image>,
    roadblock_sprite: Handle<Image>,
    safehouse_sprite: Handle<Image>,
    
    // Future expansion assets
    _health_bar_bg: Handle<Image>,
    _health_bar_fill: Handle<Image>,
    _main_font: Handle<Font>,
    _gunshot_sound: Handle<KiraAudioSource>,
    _explosion_sound: Handle<KiraAudioSource>,
    _radio_chatter: Handle<KiraAudioSource>,
}

#[derive(Resource)]
struct GameState {
    mission_timer: f32,
    current_wave: u32,
    cartel_score: u32,
    military_score: u32,
    game_phase: GamePhase,
    ovidio_captured: bool,
}

#[derive(PartialEq, Debug, Clone)]
enum GamePhase {
    Preparation,    // Initial setup
    InitialRaid,   // Mission 1: Defend safehouse
    BlockConvoy,   // Mission 2: Block extraction
    ApplyPressure, // Mission 3: Escalate pressure
    HoldTheLine,   // Mission 4: Final showdown
    GameOver,
}

impl Default for GameState {
    fn default() -> Self {
        Self {
            mission_timer: 0.0,
            current_wave: 0,
            cartel_score: 0,
            military_score: 0,
            game_phase: GamePhase::Preparation,
            ovidio_captured: false,
        }
    }
}

// ==================== MAIN FUNCTION ====================

// ==================== MAIN FUNCTION ====================

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Battle of Culiacán - El Culiacanazo RTS".into(),
                resolution: (1400.0, 900.0).into(),
                resizable: true,
                present_mode: bevy::window::PresentMode::AutoVsync,
                mode: bevy::window::WindowMode::Windowed,
                visible: true,
                ..default()
            }),
            ..default()
        }))
        .add_plugins(KiraAudioPlugin)
        .init_resource::<GameState>()
        .add_systems(Startup, (setup_assets, setup_ui))
        .add_systems(Update, setup_game.run_if(resource_exists::<GameAssets>()).run_if(not(resource_exists::<GameSetupComplete>())))
        .add_systems(Update, (
            camera_control_system,
            unit_selection_system,
            selection_indicator_system,
            wave_spawner_system,
            unit_ai_system,
            movement_system,
            combat_system,
            health_bar_system,
            particle_system,
            damage_indicator_system,
            game_phase_system,
            handle_input,
            ui_update_system,
        ))
        .run();
}

// ==================== SETUP SYSTEMS ====================

fn setup_assets(
    mut commands: Commands, 
    asset_server: Res<AssetServer>
) {
    info!("🔧 Loading sprite assets...");
    // Load individual sprite files
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

fn setup_ui(mut commands: Commands, _asset_server: Res<AssetServer>) {
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
    
    // Main UI Container
    commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                ..default()
            },
            UIElement,
        ))
        .with_children(|parent| {
            // Top HUD Bar
            parent
                .spawn(NodeBundle {
                    style: Style {
                        width: Val::Percent(100.0),
                        height: Val::Px(80.0),
                        padding: UiRect::all(Val::Px(15.0)),
                        justify_content: JustifyContent::SpaceBetween,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    background_color: Color::rgba(0.1, 0.1, 0.1, 0.9).into(),
                    ..default()
                })
                .with_children(|parent| {
                    // Mission Title
                    parent.spawn((
                        TextBundle::from_section(
                            "⚔️ Battle of Culiacán - October 17, 2019",
                            TextStyle {
                                font_size: 28.0,
                                color: Color::rgb(1.0, 0.9, 0.6),
                                ..default()
                            },
                        ),
                    ));
                    
                    // Wave Counter
                    parent.spawn((
                        TextBundle::from_section(
                            "Wave: 0",
                            TextStyle {
                                font_size: 22.0,
                                color: Color::rgb(1.0, 0.3, 0.3),
                                ..default()
                            },
                        ),
                        WaveText,
                    ));
                    
                    // Score Display
                    parent.spawn((
                        TextBundle::from_section(
                            "Cartel: 0 | Military: 0",
                            TextStyle {
                                font_size: 18.0,
                                color: Color::WHITE,
                                ..default()
                            },
                        ),
                        ScoreText,
                    ));
                });
            
            // Mission Status (Center)
            parent
                .spawn(NodeBundle {
                    style: Style {
                        width: Val::Percent(100.0),
                        height: Val::Px(50.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    background_color: Color::rgba(0.8, 0.2, 0.2, 0.8).into(),
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn((
                        TextBundle::from_section(
                            "🎯 MISSION: Defend Ovidio Guzmán López - Government forces incoming!",
                            TextStyle {
                                font_size: 20.0,
                                color: Color::WHITE,
                                ..default()
                            },
                        ),
                        StatusText,
                    ));
                });
            
            // Bottom Control Bar
            parent
                .spawn(NodeBundle {
                    style: Style {
                        position_type: PositionType::Absolute,
                        bottom: Val::Px(0.0),
                        width: Val::Percent(100.0),
                        height: Val::Px(60.0),
                        padding: UiRect::all(Val::Px(15.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    background_color: Color::rgba(0.0, 0.0, 0.0, 0.8).into(),
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_section(
                        "🎮 SPACE: Deploy Roadblock | R: Call Reinforcements | ESC: Exit | F1: Help",
                        TextStyle {
                            font_size: 16.0,
                            color: Color::rgb(0.8, 0.8, 0.8),
                            ..default()
                        },
                    ));
                });
        });
}

fn setup_game(
    mut commands: Commands, 
    game_assets: Res<GameAssets>
) {
    info!("🚀 Starting game setup...");
    info!("🎮 Battle of Culiacán - October 17, 2019");
    info!("🏛️  Government forces attempt to capture Ovidio Guzmán López");
    info!("⚔️  Sinaloa Cartel prepares defensive operations");
    
    // Audio atmosphere setup
    info!("📻 *RADIO STATIC* 'This is Command... Operation Black Thursday is a go...'");
    info!("🌅 *MORNING SOUNDS* Culiacán awakens to the sound of helicopters...");
    info!("🚁 *DISTANT ROTOR BLADES* Military forces approaching coordinates...");
    
    // Create visible ground plane
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(0.3, 0.4, 0.2), // Darker ground for contrast
                custom_size: Some(Vec2::new(800.0, 600.0)),
                ..default()
            },
            transform: Transform::from_xyz(0.0, 0.0, -10.0), // No rotation for now
            ..default()
        },
        Name::new("Ground"),
    ));
    
    // Simple grid lines for reference
    for i in -5..=5 {
        // Vertical lines
        commands.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: Color::rgba(0.5, 0.5, 0.5, 0.3),
                    custom_size: Some(Vec2::new(1.0, 400.0)),
                    ..default()
                },
                transform: Transform::from_xyz(i as f32 * 80.0, 0.0, -5.0),
                ..default()
            },
            Name::new("GridLine"),
        ));
        
        // Horizontal lines  
        commands.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: Color::rgba(0.5, 0.5, 0.5, 0.3),
                    custom_size: Some(Vec2::new(400.0, 1.0)),
                    ..default()
                },
                transform: Transform::from_xyz(0.0, i as f32 * 60.0, -5.0),
                ..default()
            },
            Name::new("GridLine"),
        ));
    }
    
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
                custom_size: Some(Vec2::new(120.0, 80.0)),
                ..default()
            },
            transform: Transform::from_translation(world_to_iso(safehouse_pos) + Vec3::new(0.0, 0.0, -1.0)),
            // Remove rotation for visibility
            ..default()
        },
        Objective {
            objective_type: ObjectiveType::Safehouse,
            _position: safehouse_pos,
            _radius: 100.0,
            _health: 200.0,
        },
    ));
    
    // Add safehouse label
    commands.spawn((
        Text2dBundle {
            text: Text::from_section(
                "🏠 SAFEHOUSE",
                TextStyle {
                    font_size: 16.0,
                    color: Color::WHITE,
                    ..default()
                },
            ),
            transform: Transform::from_translation(world_to_iso(safehouse_pos) + Vec3::new(0.0, 40.0, 1.0)),
            ..default()
        },
    ));
    
    // Spawn wave spawner
    commands.spawn(WaveSpawner {
        next_wave_timer: Timer::new(Duration::from_secs(10), TimerMode::Repeating),
        wave_number: 0,
        units_in_wave: 2,
    });
    
    info!("🎯 Mission: Defend Ovidio and prevent extraction!");
    info!("📱 Controls: SPACE=Roadblock, R=Reinforcements, ESC=Exit");
    
    // Mark game setup as complete
    commands.insert_resource(GameSetupComplete);
}

fn spawn_ovidio(
    commands: &mut Commands, 
    position: Vec3, 
    game_assets: &Res<GameAssets>
) {
    let entity = commands.spawn((
        SpriteBundle {
            texture: game_assets.ovidio_sprite.clone(),
            sprite: Sprite {
                color: Color::rgb(1.0, 0.8, 0.0), // Golden tint for VIP
                custom_size: Some(Vec2::new(64.0, 64.0)), // Larger for visibility
                ..default()
            },
            transform: Transform::from_translation(world_to_iso(position)),
            ..default()
        },
        Unit {
            health: 50.0,
            max_health: 50.0,
            faction: Faction::Cartel,
            unit_type: UnitType::Ovidio,
            damage: 0.0,
            range: 0.0,
            movement_speed: 60.0,
            target: None,
            attack_cooldown: Timer::new(Duration::from_secs(1), TimerMode::Repeating),
        },
        Movement {
            target_position: None,
            speed: 60.0,
        },
    )).id();
    
    // Add crown emoji label for Ovidio
    commands.spawn(Text2dBundle {
        text: Text::from_section(
            "👑",
            TextStyle {
                font_size: 20.0,
                color: Color::YELLOW,
                ..default()
            },
        ),
        transform: Transform::from_translation(world_to_iso(position) + Vec3::new(0.0, 25.0, 1.0)),
        ..default()
    });
    
    // Add health bar for Ovidio
    spawn_health_bar(commands, entity, position);
}

fn spawn_unit(
    commands: &mut Commands, 
    unit_type: UnitType, 
    faction: Faction, 
    position: Vec3,
    game_assets: &Res<GameAssets>
) {
    // Get unit health, damage, range, speed based on type and faction
    let (health, damage, range, speed) = match (&unit_type, &faction) {
        (UnitType::Sicario, Faction::Cartel) => (80.0, 25.0, 120.0, 100.0),
        (UnitType::Enforcer, Faction::Cartel) => (120.0, 40.0, 150.0, 80.0),
        (UnitType::Soldier, Faction::Military) => (100.0, 30.0, 140.0, 90.0),
        (UnitType::SpecialForces, Faction::Military) => (140.0, 50.0, 180.0, 110.0),
        (UnitType::Vehicle, Faction::Military) => (200.0, 60.0, 200.0, 70.0),
        (UnitType::Roadblock, Faction::Cartel) => (75.0, 0.0, 0.0, 0.0),
        _ => (100.0, 20.0, 100.0, 80.0),
    };
    
    // Get sprite handle for unit type
    let sprite_handle = match unit_type {
        UnitType::Sicario => game_assets.sicario_sprite.clone(),
        UnitType::Enforcer => game_assets.enforcer_sprite.clone(),
        UnitType::Soldier => game_assets.soldier_sprite.clone(),
        UnitType::SpecialForces => game_assets.special_forces_sprite.clone(),
        UnitType::Vehicle => game_assets.vehicle_sprite.clone(),
        UnitType::Roadblock => game_assets.roadblock_sprite.clone(),
        _ => game_assets.sicario_sprite.clone(), // fallback
    };
    
    let entity = commands.spawn((
        SpriteBundle {
            texture: sprite_handle,
            sprite: Sprite {
                custom_size: Some(Vec2::new(64.0, 64.0)), // Larger sprite size for visibility
                ..default()
            },
            transform: Transform::from_translation(world_to_iso(position)),
            ..default()
        },
        Unit {
            health,
            max_health: health,
            faction: faction.clone(),
            unit_type: unit_type.clone(),
            damage,
            range,
            movement_speed: speed,
            target: None,
            attack_cooldown: Timer::new(Duration::from_millis(800), TimerMode::Repeating),
        },
        Movement {
            target_position: None,
            speed,
        },
    )).id();
    
    // Add health bar for all units except roadblocks
    if unit_type != UnitType::Roadblock {
        spawn_health_bar(commands, entity, position);
    }
    
    // Add emoji indicator above the unit for better identification
    let unit_label = match unit_type {
        UnitType::Sicario => "🔫",
        UnitType::Enforcer => "💪", 
        UnitType::Soldier => "🪖",
        UnitType::SpecialForces => "⭐",
        UnitType::Vehicle => "🚗",
        UnitType::Roadblock => "🚧",
        _ => "❓",
    };
    
    commands.spawn((
        Text2dBundle {
            text: Text::from_section(
                unit_label,
                TextStyle {
                    font_size: 18.0,
                    color: Color::WHITE,
                    ..default()
                },
            ),
            transform: Transform::from_translation(world_to_iso(position) + Vec3::new(0.0, 20.0, 2.0)),
            ..default()
        },
        Name::new("UnitEmoji"),
    ));
    
    commands.spawn(Text2dBundle {
        text: Text::from_section(
            "📍", // Generic marker
            TextStyle {
                font_size: 10.0,
                color: Color::WHITE,
                ..default()
            },
        ),
        transform: Transform::from_translation(position + Vec3::new(0.0, 20.0, 1.0)),
        ..default()
    });
}

fn spawn_health_bar(commands: &mut Commands, owner: Entity, position: Vec3) {
    let _health_bg = commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(0.2, 0.2, 0.2),
                custom_size: Some(Vec2::new(30.0, 4.0)),
                ..default()
            },
            transform: Transform::from_translation(position + Vec3::new(0.0, -25.0, 1.0)),
            ..default()
        },
        HealthBar {
            owner,
            offset: Vec3::new(0.0, -25.0, 1.0),
        },
    )).id();
    
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(0.2, 0.8, 0.2),
                custom_size: Some(Vec2::new(30.0, 4.0)),
                ..default()
            },
            transform: Transform::from_translation(position + Vec3::new(0.0, -25.0, 2.0)),
            ..default()
        },
        HealthBar {
            owner,
            offset: Vec3::new(0.0, -25.0, 2.0),
        },
    ));
}

// ==================== GAME SYSTEMS ====================

fn wave_spawner_system(
    time: Res<Time>,
    mut commands: Commands,
    mut wave_query: Query<&mut WaveSpawner>,
    mut game_state: ResMut<GameState>,
    game_assets: Res<GameAssets>,
) {
    for mut spawner in wave_query.iter_mut() {
        spawner.next_wave_timer.tick(time.delta());
        
        if spawner.next_wave_timer.just_finished() {
            spawner.wave_number += 1;
            game_state.current_wave = spawner.wave_number;
            
            let wave_difficulty = (spawner.wave_number as f32 * 0.5 + 1.0).min(4.0);
            let units_to_spawn = (spawner.units_in_wave as f32 * wave_difficulty) as u32;
            
            info!("🚁 *HELICOPTER ROTORS* 🌊 WAVE {} INCOMING! {} military units deployed 📻 *RADIO STATIC*", spawner.wave_number, units_to_spawn);
            
            // Atmospheric audio cues
            match spawner.wave_number {
                1 => info!("📻 'Alpha team, move in! Target: Ovidio Guzmán!'"),
                2 => info!("📻 'Bravo team, reinforce Alpha! Heavy resistance!'"), 
                3 => info!("📻 'Charlie team, we need immediate backup!'"),
                4 => info!("📻 'All units! Full assault! Take the safehouse!'"),
                _ => info!("📻 'Command, we're escalating operations!'"),
            }
            
            // Spawn military units from different entry points (closer to center)
            let entry_points = vec![
                Vec3::new(200.0, 100.0, 0.0),   // North entry
                Vec3::new(200.0, 0.0, 0.0),     // East entry  
                Vec3::new(200.0, -100.0, 0.0),  // South entry
            ];
            
            for i in 0..units_to_spawn {
                let entry_point = entry_points[i as usize % entry_points.len()];
                let offset = Vec3::new(
                    thread_rng().gen_range(-50.0..50.0),
                    thread_rng().gen_range(-50.0..50.0), 
                    0.0
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
        }
    }
}

fn unit_ai_system(
    mut unit_query: Query<(&mut Unit, &Transform, &mut Movement), Without<Objective>>,
    objective_query: Query<(&Objective, &Transform), (With<Objective>, Without<Unit>)>,
    enemy_units: Query<(Entity, &Unit, &Transform), (With<Unit>, Without<Movement>)>,
) {
    for (mut unit, transform, mut movement) in unit_query.iter_mut() {
        match unit.faction {
            Faction::Military => {
                // Military AI: Find and attack cartel units or move to objectives
                if unit.target.is_none() {
                    // Find nearest enemy or objective
                    let mut nearest_target = None;
                    let mut nearest_distance = f32::MAX;
                    
                    // Check for enemy units in range
                    for (entity, other_unit, other_transform) in enemy_units.iter() {
                        if other_unit.faction == Faction::Cartel {
                            let distance = transform.translation.distance(other_transform.translation);
                            if distance < nearest_distance && distance <= unit.range {
                                nearest_distance = distance;
                                nearest_target = Some(entity);
                            }
                        }
                    }
                    
                    unit.target = nearest_target;
                    
                    // If no target, move toward safehouse
                    if unit.target.is_none() {
                        for (objective, obj_transform) in objective_query.iter() {
                            if objective.objective_type == ObjectiveType::Safehouse {
                                movement.target_position = Some(obj_transform.translation);
                                break;
                            }
                        }
                    }
                }
            },
            Faction::Cartel => {
                // Cartel AI: Defend position and attack nearby military units
                if unit.target.is_none() {
                    let mut nearest_enemy = None;
                    let mut nearest_distance = f32::MAX;
                    
                    for (entity, other_unit, other_transform) in enemy_units.iter() {
                        if other_unit.faction == Faction::Military {
                            let distance = transform.translation.distance(other_transform.translation);
                            if distance < nearest_distance && distance <= unit.range {
                                nearest_distance = distance;
                                nearest_enemy = Some(entity);
                            }
                        }
                    }
                    
                    unit.target = nearest_enemy;
                }
            },
            _ => {}
        }
    }
}

fn movement_system(
    time: Res<Time>,
    mut unit_query: Query<(&mut Transform, &Movement, &Unit)>,
    target_query: Query<&Transform, (With<Unit>, Without<Movement>)>,
) {
    for (mut transform, movement, unit) in unit_query.iter_mut() {
        if let Some(target_pos) = movement.target_position {
            let direction = (target_pos - transform.translation).normalize();
            let movement_delta = direction * movement.speed * time.delta_seconds();
            
            // Stop when close enough to target
            if transform.translation.distance(target_pos) > 20.0 {
                transform.translation += movement_delta;
            }
        }
        
        // Move toward target unit if attacking
        if let Some(target_entity) = unit.target {
            if let Ok(target_transform) = target_query.get(target_entity) {
                let direction = (target_transform.translation - transform.translation).normalize();
                let distance = transform.translation.distance(target_transform.translation);
                
                // Move closer if out of range
                if distance > unit.range * 0.8 {
                    let movement_delta = direction * unit.movement_speed * time.delta_seconds();
                    transform.translation += movement_delta;
                }
            }
        }
    }
}

fn combat_system(
    time: Res<Time>,
    mut commands: Commands,
    mut unit_query: Query<(Entity, &mut Unit, &Transform)>,
    mut game_state: ResMut<GameState>,
    _audio: Res<KiraAudio>, // Add audio system
) {
    let mut combat_events = Vec::new();
    
    // Collect combat events first
    let units: Vec<(Entity, Unit, Transform)> = unit_query.iter()
        .map(|(e, u, t)| (e, u.clone(), *t))
        .collect();
    
    for (_entity, mut unit, _transform) in unit_query.iter_mut() {
        unit.attack_cooldown.tick(time.delta());
        
        if let Some(target_entity) = unit.target {
            if unit.attack_cooldown.just_finished() {
                // Check if target still exists in our collected units
                if let Some((_, _, target_transform)) = units.iter()
                    .find(|(e, _, _)| *e == target_entity) {
                    let distance = _transform.translation.distance(target_transform.translation);
                    if distance <= unit.range {
                        combat_events.push((target_entity, unit.damage, unit.faction.clone(), _transform.translation, target_transform.translation));
                    } else {
                        unit.target = None; // Target out of range
                    }
                } else {
                    unit.target = None; // Target no longer exists
                }
            }
        }
    }
    
    // Apply damage and visual/audio effects
    for (target_entity, damage, attacker_faction, attacker_pos, target_pos) in combat_events {
        if let Ok((entity, mut unit, transform)) = unit_query.get_mut(target_entity) {
            unit.health -= damage;
            
            // Audio feedback via console - simulating gunshot sounds
            let sound_effect = match attacker_faction {
                Faction::Military => "🔫 *POP-POP-POP*", // Military rifle burst
                Faction::Cartel => "💥 *BANG-BANG*",    // Cartel pistol shots
                _ => "💢 *CRACK*",
            };
            
            info!("{} {} fires! {} takes {} damage (HP: {:.1})", 
                  sound_effect,
                  format!("{:?}", attacker_faction),
                  format!("{:?}", unit.faction),
                  damage,
                  unit.health);
            
            // TODO: Replace with actual audio when files are available
            // audio.play(assets.gunshot_sound.clone());
            
            // Spawn muzzle flash particles with color effects
            for _ in 0..3 {
                let velocity = Vec3::new(
                    thread_rng().gen_range(-100.0..100.0),
                    thread_rng().gen_range(-100.0..100.0),
                    0.0,
                );
                
                commands.spawn((
                    SpriteBundle {
                        sprite: Sprite {
                            color: Color::rgb(1.0, 1.0, 0.6),
                            custom_size: Some(Vec2::new(5.0, 5.0)),
                            ..default()
                        },
                        transform: Transform::from_translation(world_to_iso(attacker_pos) + Vec3::new(0.0, 0.0, 3.0)),
                        ..default()
                    },
                    ParticleEffect {
                        lifetime: Timer::new(Duration::from_millis(200), TimerMode::Once),
                        velocity,
                    },
                ));
            }
            
            // Spawn damage indicator
            let indicator_color = match attacker_faction {
                Faction::Military => Color::rgb(0.3, 1.0, 0.3),
                Faction::Cartel => Color::rgb(1.0, 0.3, 0.3),
                _ => Color::WHITE,
            };
            
            commands.spawn((
                Text2dBundle {
                    text: Text::from_section(
                        format!("-{}", damage as u32),
                        TextStyle {
                            font_size: 14.0,
                            color: indicator_color,
                            ..default()
                        },
                    ),
                    transform: Transform::from_translation(world_to_iso(transform.translation + Vec3::new(0.0, 10.0, 4.0))),
                    ..default()
                },
                DamageIndicator {
                    lifetime: Timer::new(Duration::from_secs(1), TimerMode::Once),
                },
            ));
            
            // Impact particles at target
            for _ in 0..2 {
                let velocity = Vec3::new(
                    thread_rng().gen_range(-50.0..50.0),
                    thread_rng().gen_range(-50.0..50.0),
                    0.0,
                );
                
                commands.spawn((
                    SpriteBundle {
                        sprite: Sprite {
                            color: Color::rgb(0.8, 0.3, 0.3),
                            custom_size: Some(Vec2::new(2.0, 2.0)),
                            ..default()
                        },
                        transform: Transform::from_translation(target_pos + Vec3::new(0.0, 0.0, 3.0)),
                        ..default()
                    },
                    ParticleEffect {
                        lifetime: Timer::new(Duration::from_millis(300), TimerMode::Once),
                        velocity,
                    },
                ));
            }
            
            if unit.health <= 0.0 {
                // Death audio feedback
                let death_sound = match unit.faction {
                    Faction::Cartel => "💀 *CARTEL DOWN*",
                    Faction::Military => "⚰️ *MILITARY KIA*", 
                    _ => "💥 *ELIMINATED*",
                };
                
                let unit_name = match unit.unit_type {
                    UnitType::Ovidio => "👑 OVIDIO GUZMÁN LÓPEZ",
                    UnitType::Sicario => "🔫 Sicario",
                    UnitType::Enforcer => "⚔️ Enforcer", 
                    UnitType::Soldier => "🪖 Soldier",
                    UnitType::SpecialForces => "🎯 Special Forces",
                    UnitType::Vehicle => "🚗 Vehicle",
                    UnitType::Roadblock => "🚧 Roadblock",
                };
                
                info!("{} {} eliminated! 💥💥💥", death_sound, unit_name);
                
                match unit.faction {
                    Faction::Cartel => {
                        game_state.military_score += 10;
                        if unit.unit_type == UnitType::Ovidio {
                            game_state.ovidio_captured = true;
                            info!("🎯 CRITICAL: Ovidio Guzmán López captured!");
                        }
                    },
                    Faction::Military => {
                        game_state.cartel_score += 15;
                    },
                    _ => {}
                }
                
                // Spawn death explosion effect
                for _ in 0..8 {
                    let velocity = Vec3::new(
                        thread_rng().gen_range(-150.0..150.0),
                        thread_rng().gen_range(-150.0..150.0),
                        0.0,
                    );
                    
                    commands.spawn((
                        SpriteBundle {
                            sprite: Sprite {
                                color: Color::rgb(1.0, 0.6, 0.2),
                                custom_size: Some(Vec2::new(4.0, 4.0)),
                                ..default()
                            },
                            transform: Transform::from_translation(transform.translation + Vec3::new(0.0, 0.0, 3.0)),
                            ..default()
                        },
                        ParticleEffect {
                            lifetime: Timer::new(Duration::from_millis(500), TimerMode::Once),
                            velocity,
                        },
                    ));
                }
                
                commands.entity(entity).despawn_recursive();
            }
        }
    }
}

fn ui_update_system(
    game_state: Res<GameState>,
    unit_query: Query<&Unit>,
    mut wave_text: Query<&mut Text, (With<WaveText>, Without<StatusText>, Without<ScoreText>)>,
    mut status_text: Query<&mut Text, (With<StatusText>, Without<WaveText>, Without<ScoreText>)>,
    mut score_text: Query<&mut Text, (With<ScoreText>, Without<WaveText>, Without<StatusText>)>,
) {
    // Update wave counter
    if let Ok(mut text) = wave_text.get_single_mut() {
        text.sections[0].value = format!("Wave: {}", game_state.current_wave);
    }
    
    // Update score display
    if let Ok(mut text) = score_text.get_single_mut() {
        text.sections[0].value = format!("Cartel: {} | Military: {}", 
                                        game_state.cartel_score, game_state.military_score);
    }
    
    // Update mission status
    if let Ok(mut text) = status_text.get_single_mut() {
        let cartel_count = unit_query.iter()
            .filter(|u| u.faction == Faction::Cartel && u.unit_type != UnitType::Roadblock)
            .count();
        let military_count = unit_query.iter()
            .filter(|u| u.faction == Faction::Military)
            .count();
        let ovidio_alive = unit_query.iter()
            .any(|u| u.unit_type == UnitType::Ovidio);
        
        let status_msg = match game_state.game_phase {
            GamePhase::Preparation => "🎯 PREPARING: Government forces mobilizing...",
            GamePhase::InitialRaid => "🚁 PHASE 1: Initial raid in progress!",
            GamePhase::BlockConvoy => "🛑 PHASE 2: Block all escape routes!",
            GamePhase::ApplyPressure => "👥 PHASE 3: Pressure tactics engaged!",
            GamePhase::HoldTheLine => "⏰ PHASE 4: Final showdown - Hold the line!",
            GamePhase::GameOver => if game_state.ovidio_captured { "💀 DEFEAT: Ovidio captured" } else { "🏆 VICTORY: Historical outcome achieved" },
        };
        
        text.sections[0].value = format!("{} | Cartel: {} | Military: {} | Ovidio: {} | Time: {:.0}s",
                                        status_msg, cartel_count, military_count,
                                        if ovidio_alive { "SAFE" } else { "CAPTURED" },
                                        game_state.mission_timer);
    }
}

fn health_bar_system(
    unit_query: Query<(Entity, &Unit, &Transform), Changed<Unit>>,
    mut health_bar_query: Query<(&mut Transform, &mut Sprite, &HealthBar), Without<Unit>>,
) {
    for (unit_entity, unit, unit_transform) in unit_query.iter() {
        // Update health bars for this unit
        for (mut bar_transform, mut bar_sprite, health_bar) in health_bar_query.iter_mut() {
            if health_bar.owner == unit_entity {
                // Update position with isometric offset
                let iso_pos = unit_transform.translation + health_bar.offset;
                bar_transform.translation = Vec3::new(iso_pos.x, iso_pos.y + 15.0, iso_pos.z); // Higher up in isometric view
                
                // Update health bar fill (green bar on top)
                if health_bar.offset.z > 1.5 { // This is the fill bar
                    let health_percentage = (unit.health / unit.max_health).max(0.0);
                    bar_sprite.custom_size = Some(Vec2::new(30.0 * health_percentage, 4.0));
                    
                    // Change color based on health
                    bar_sprite.color = if health_percentage > 0.6 {
                        Color::rgb(0.2, 0.8, 0.2) // Green
                    } else if health_percentage > 0.3 {
                        Color::rgb(0.8, 0.8, 0.2) // Yellow
                    } else {
                        Color::rgb(0.8, 0.2, 0.2) // Red
                    };
                }
            }
        }
    }
}

fn particle_system(
    time: Res<Time>,
    mut commands: Commands,
    mut particle_query: Query<(Entity, &mut Transform, &mut ParticleEffect)>,
) {
    for (entity, mut transform, mut particle) in particle_query.iter_mut() {
        particle.lifetime.tick(time.delta());
        
        if particle.lifetime.finished() {
            commands.entity(entity).despawn();
        } else {
            // Move particle
            transform.translation += particle.velocity * time.delta_seconds();
            
            // Fade out
            let _alpha = 1.0 - particle.lifetime.percent();
            // Update sprite color alpha here if needed
        }
    }
}

fn damage_indicator_system(
    time: Res<Time>,
    mut commands: Commands,
    mut indicator_query: Query<(Entity, &mut Transform, &mut DamageIndicator)>,
) {
    for (entity, mut transform, mut indicator) in indicator_query.iter_mut() {
        indicator.lifetime.tick(time.delta());
        
        if indicator.lifetime.finished() {
            commands.entity(entity).despawn();
        } else {
            // Float upward
            transform.translation.y += 50.0 * time.delta_seconds();
        }
    }
}

fn game_phase_system(
    time: Res<Time>,
    mut game_state: ResMut<GameState>,
    unit_query: Query<&Unit>,
) {
    game_state.mission_timer += time.delta_seconds();
    
    let cartel_alive = unit_query.iter().any(|u| u.faction == Faction::Cartel && u.unit_type != UnitType::Roadblock);
    let _military_alive = unit_query.iter().any(|u| u.faction == Faction::Military);
    let ovidio_alive = unit_query.iter().any(|u| u.unit_type == UnitType::Ovidio && u.health > 0.0);
    
    // Phase transitions based on time and events
    let new_phase = match game_state.game_phase {
        GamePhase::Preparation if game_state.mission_timer > 5.0 => {
            info!("🚁 *HELICOPTER SOUNDS* 📻 'ATENCIÓN! OPERATION BLACK THURSDAY INITIATED!' 🚁 Phase 1: INITIAL RAID - Government forces storm the safehouse! 🔊 *SIRENS WAILING*");
            GamePhase::InitialRaid
        },
        GamePhase::InitialRaid if game_state.mission_timer > 120.0 => {
            info!("🛑 Phase 2: BLOCK CONVOY - Cut off all escape routes!");
            GamePhase::BlockConvoy  
        },
        GamePhase::BlockConvoy if game_state.mission_timer > 300.0 => {
            info!("👨‍👩‍👧‍👦 Phase 3: APPLY PRESSURE - Target military families!");
            GamePhase::ApplyPressure
        },
        GamePhase::ApplyPressure if game_state.mission_timer > 480.0 => {
            info!("⏰ Phase 4: HOLD THE LINE - Final showdown!");
            GamePhase::HoldTheLine
        },
        _ => game_state.game_phase.clone(),
    };
    
    if new_phase != game_state.game_phase {
        game_state.game_phase = new_phase;
    }
    
    // Victory/Defeat conditions (only check after game has started properly)
    if game_state.mission_timer > 5.0 {
        if !ovidio_alive && !game_state.ovidio_captured {
            info!("🔍 DEBUG: Ovidio death detected - ovidio_alive: {}, captured: {}, timer: {:.1}s", ovidio_alive, game_state.ovidio_captured, game_state.mission_timer);
            info!("💀 DEFEAT: Ovidio Guzmán López was killed in the operation");
            game_state.game_phase = GamePhase::GameOver;
        }
    }
    
    if game_state.ovidio_captured && !cartel_alive {
        info!("🎖️ MILITARY VICTORY: Target captured, cartel eliminated");  
        game_state.game_phase = GamePhase::GameOver;
    } else if game_state.mission_timer > 600.0 && cartel_alive {
        info!("🏆 HISTORICAL OUTCOME: Government releases Ovidio to prevent casualties");
        info!("📰 'El Culiacanazo' - Cartel demonstrates power over the state");
        game_state.game_phase = GamePhase::GameOver;
    }
}

// ==================== CAMERA CONTROL SYSTEM ====================

fn camera_control_system(
    input: Res<Input<KeyCode>>,
    mut mouse_wheel_events: EventReader<bevy::input::mouse::MouseWheel>,
    mut camera_query: Query<(&mut Transform, &mut IsometricCamera), With<Camera>>,
    time: Res<Time>,
) {
    for (mut transform, camera) in camera_query.iter_mut() {
        let mut movement = Vec3::ZERO;
        
        // WASD camera movement
        if input.pressed(KeyCode::W) || input.pressed(KeyCode::Up) {
            movement.y += camera.pan_speed * time.delta_seconds();
        }
        if input.pressed(KeyCode::S) || input.pressed(KeyCode::Down) {
            movement.y -= camera.pan_speed * time.delta_seconds();
        }
        if input.pressed(KeyCode::A) || input.pressed(KeyCode::Left) {
            movement.x -= camera.pan_speed * time.delta_seconds();
        }
        if input.pressed(KeyCode::D) || input.pressed(KeyCode::Right) {
            movement.x += camera.pan_speed * time.delta_seconds();
        }
        
        transform.translation += movement;
        
        // Mouse wheel zoom
        for wheel_event in mouse_wheel_events.iter() {
            let zoom_delta = wheel_event.y * camera.zoom_speed;
            let new_scale = (transform.scale.x - zoom_delta).clamp(camera.min_zoom, camera.max_zoom);
            transform.scale = Vec3::splat(new_scale);
        }
    }
}

// ==================== UNIT SELECTION SYSTEM ====================

fn unit_selection_system(
    mut commands: Commands,
    mouse_button_input: Res<Input<MouseButton>>,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform), With<IsometricCamera>>,
    mut unit_query: Query<(Entity, &Transform, &Unit, Option<&Selected>)>,
) {
    if !mouse_button_input.just_pressed(MouseButton::Left) {
        return;
    }
    
    let window = windows.single();
    let (camera, camera_transform) = camera_query.single();
    
    if let Some(cursor_pos) = window.cursor_position() {
        // Convert screen position to world position
        if let Some(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_pos) {
            let mut closest_unit = None;
            let mut closest_distance = f32::INFINITY;
            
            // Find closest cartel unit to cursor
            for (entity, transform, unit, selected) in unit_query.iter() {
                if unit.faction == Faction::Cartel {
                    let distance = transform.translation.truncate().distance(world_pos);
                    if distance < 50.0 && distance < closest_distance {
                        closest_distance = distance;
                        closest_unit = Some((entity, selected.is_some()));
                    }
                }
            }
            
            // Clear all selections first
            for (entity, _, _, selected) in unit_query.iter() {
                if selected.is_some() {
                    commands.entity(entity).remove::<Selected>();
                }
            }
            
            // Select the closest unit
            if let Some((entity, _was_selected)) = closest_unit {
                commands.entity(entity).insert(Selected {
                    selection_color: Color::rgb(1.0, 1.0, 0.0), // Yellow selection
                });
                info!("🎯 Unit selected! Right-click to move.");
            }
        }
    }
}

// ==================== SELECTION INDICATOR SYSTEM ====================

fn selection_indicator_system(
    mut commands: Commands,
    selected_query: Query<(Entity, &Transform, &Selected), With<Unit>>,
    indicator_query: Query<Entity, (With<SelectionIndicator>, Without<Unit>)>,
) {
    // Remove old indicators
    for entity in indicator_query.iter() {
        commands.entity(entity).despawn();
    }
    
    // Add new indicators for selected units
    for (_, transform, selected) in selected_query.iter() {
        commands.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: selected.selection_color,
                    custom_size: Some(Vec2::new(80.0, 80.0)), // Circle around unit
                    ..default()
                },
                transform: Transform::from_translation(transform.translation + Vec3::new(0.0, 0.0, -0.5)),
                ..default()
            },
            SelectionIndicator,
        ));
    }
}

#[derive(Component)]
struct SelectionIndicator;

fn handle_input(
    input: Res<Input<KeyCode>>,
    mouse_button_input: Res<Input<MouseButton>>,
    mut commands: Commands,
    mut game_state: ResMut<GameState>,
    game_assets: Res<GameAssets>,
    mut app_exit_events: EventWriter<bevy::app::AppExit>,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform), With<IsometricCamera>>,
    mut selected_units: Query<&mut Movement, (With<Selected>, With<Unit>)>,
) {
    // Right-click to move selected units
    if mouse_button_input.just_pressed(MouseButton::Right) {
        let window = windows.single();
        if let Ok((camera, camera_transform)) = camera_query.get_single() {
            if let Some(cursor_pos) = window.cursor_position() {
                if let Some(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_pos) {
                    let target_pos = Vec3::new(world_pos.x, world_pos.y, 0.0);
                    
                    // Move all selected units to the target position
                    for mut movement in selected_units.iter_mut() {
                        movement.target_position = Some(target_pos);
                        info!("🚶 Unit moving to position: ({:.0}, {:.0})", target_pos.x, target_pos.y);
                    }
                }
            }
        }
    }
    
    if input.just_pressed(KeyCode::Space) {
        // Deploy roadblock with enhanced visuals
        let position = Vec3::new(
            thread_rng().gen_range(-400.0..400.0),
            thread_rng().gen_range(-300.0..300.0),
            0.0
        );
        
        let _entity = commands.spawn((
            SpriteBundle {
                texture: game_assets.roadblock_sprite.clone(),
                sprite: Sprite {
                    color: Color::rgb(0.8, 0.6, 0.3), // Slightly brighter tint for visibility
                    custom_size: Some(Vec2::new(64.0, 64.0)), // Larger for visibility
                    ..default()
                },
                transform: Transform::from_translation(world_to_iso(position)),
                ..default()
            },
            Unit {
                health: 75.0,
                max_health: 75.0,
                faction: Faction::Cartel,
                unit_type: UnitType::Roadblock,
                damage: 0.0,
                range: 0.0,
                movement_speed: 0.0,
                target: None,
                attack_cooldown: Timer::new(Duration::from_secs(1), TimerMode::Repeating),
            },
        )).id();
        
        // Add roadblock label
        commands.spawn(Text2dBundle {
            text: Text::from_section(
                "🚧 ROADBLOCK",
                TextStyle {
                    font_size: 10.0,
                    color: Color::ORANGE,
                    ..default()
                },
            ),
            transform: Transform::from_translation(position + Vec3::new(0.0, 20.0, 1.0)),
            ..default()
        });
        
        // Spawn construction particles
        for _ in 0..5 {
            let velocity = Vec3::new(
                thread_rng().gen_range(-80.0..80.0),
                thread_rng().gen_range(-80.0..80.0),
                0.0,
            );
            
            commands.spawn((
                SpriteBundle {
                    sprite: Sprite {
                        color: Color::rgb(0.8, 0.6, 0.2),
                        custom_size: Some(Vec2::new(3.0, 3.0)),
                        ..default()
                    },
                    transform: Transform::from_translation(position + Vec3::new(0.0, 0.0, 2.0)),
                    ..default()
                },
                ParticleEffect {
                    lifetime: Timer::new(Duration::from_millis(400), TimerMode::Once),
                    velocity,
                },
            ));
        }
        
        info!("� *CONSTRUCTION SOUNDS* �🛑 ROADBLOCK deployed! Military convoy movement disrupted 📻 'Cartel blocking the roads!'");
        game_state.cartel_score += 5;
    }
    
    if input.just_pressed(KeyCode::R) {
        // Call reinforcements with enhanced spawning (closer positions)
        let spawn_positions = vec![
            Vec3::new(-150.0, 80.0, 0.0),
            Vec3::new(-120.0, 60.0, 0.0),
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
                            color: Color::rgb(0.9, 0.2, 0.2),
                            custom_size: Some(Vec2::new(4.0, 4.0)),
                            ..default()
                        },
                        transform: Transform::from_translation(*position + Vec3::new(0.0, 0.0, 2.0)),
                        ..default()
                    },
                    ParticleEffect {
                        lifetime: Timer::new(Duration::from_millis(600), TimerMode::Once),
                        velocity,
                    },
                ));
            }
        }
        
        info!("� *ENGINE REVVING* �📱 REINFORCEMENTS arriving! Cartel sends backup to the safehouse 📻 '¡Necesitamos más hombres!'");
        game_state.cartel_score += 10;
    }
    
    if input.just_pressed(KeyCode::Escape) {
        info!("🔍 DEBUG: ESC key pressed manually by user");
        info!("🏛️ SIMULATION ENDED");
        info!("📚 Historical Note: The real Battle of Culiacán ended with the government releasing Ovidio Guzmán López");
        info!("⚖️ This demonstrated the complex balance of power between organized crime and the state in Mexico");
        app_exit_events.send(bevy::app::AppExit);
    }
    
    // Debug keys
    if input.just_pressed(KeyCode::F1) {
        info!("🎮 ENHANCED CONTROLS:");
        info!("SPACE - Deploy roadblock with construction effects");
        info!("R - Call reinforcements with arrival particles");  
        info!("ESC - End simulation");
        info!("F1 - Show this help");
        info!("📊 Graphics: Health bars, damage indicators, particle effects");
        info!("🎨 Visual: Unit icons, labels, explosion effects");
    }
}
