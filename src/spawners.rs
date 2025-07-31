use crate::components::*;
use crate::resources::*;
use crate::unit_systems::{
    apply_weapon_upgrades, configure_unit_stats, get_unit_abilities, get_unit_color, get_unit_emoji,
};
use crate::utils::world_to_iso;
use bevy::log::info;
use bevy::prelude::*;

// ==================== UNIT SPAWNING FUNCTIONS ====================

pub fn spawn_unit(
    commands: &mut Commands,
    unit_type: UnitType,
    faction: Faction,
    position: Vec3,
    game_assets: &Res<GameAssets>,
) {
    // Create base unit with default stats
    let mut unit = Unit {
        health: 100.0,
        max_health: 100.0,
        faction: faction.clone(),
        unit_type: unit_type.clone(),
        damage: 30.0,
        range: 100.0,
        movement_speed: 40.0,
        target: None,
        attack_cooldown: Timer::from_seconds(1.0, TimerMode::Once),
        experience: 0,
        kills: 0,
        veterancy_level: VeterancyLevel::Recruit,
        equipment: Equipment {
            weapon: WeaponType::BasicRifle,
            armor: ArmorType::None,
            upgrades: vec![],
        },
    };

    // Configure unit stats based on type and faction
    configure_unit_stats(&mut unit, &unit_type, &faction);

    // Apply weapon upgrades
    apply_weapon_upgrades(&mut unit);

    // Get visual properties
    let sprite_handle = get_sprite_handle(&unit_type, game_assets);
    let unit_color = get_unit_color(&unit_type, &faction);
    let emoji = get_unit_emoji(&unit_type);

    let iso_position = world_to_iso(position);

    // Main unit sprite (diamond shape)
    let entity = commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: unit_color,
                custom_size: Some(Vec2::new(32.0, 32.0)),
                ..default()
            },
            texture: sprite_handle.clone(),
            transform: Transform::from_translation(iso_position),
            ..default()
        },
        unit.clone(),
        Movement {
            target_position: None,
            speed: unit.movement_speed,
        },
        AnimatedSprite {
            animation_timer: Timer::from_seconds(2.0, TimerMode::Repeating),
            scale_amplitude: 0.05, // Gentle pulsing
            rotation_speed: 0.1,   // Slow rotation
            base_scale: Vec3::new(1.0, 1.0, 1.0),
        },
        MovementAnimation {
            bob_timer: Timer::from_seconds(0.5, TimerMode::Repeating),
            bob_amplitude: 2.0, // Small vertical movement
            base_y: iso_position.y,
        },
        PathfindingAgent {
            path: Vec::new(),
            current_waypoint: 0,
            avoidance_radius: 40.0,
            max_speed: unit.movement_speed,
            stuck_timer: 0.0,
        },
    ));

    let entity = entity.id();

    // Add obstacle component for roadblocks
    if unit_type == UnitType::Roadblock {
        commands.entity(entity).insert(Obstacle { radius: 50.0 });
    }

    // Add unit abilities based on type
    let abilities = get_unit_abilities(&unit_type);
    for ability in abilities {
        commands.entity(entity).insert(ability);
    }

    // Emoji overlay for clear unit identification
    commands.spawn((Text2dBundle {
        text: Text::from_section(
            emoji,
            TextStyle {
                font_size: 24.0,
                color: Color::WHITE,
                ..default()
            },
        ),
        transform: Transform::from_translation(iso_position + Vec3::new(0.0, 0.0, 1.0)),
        ..default()
    },));

    // Add health bar
    spawn_health_bar(commands, entity, iso_position);
}

fn get_sprite_handle(unit_type: &UnitType, game_assets: &Res<GameAssets>) -> Handle<Image> {
    match unit_type {
        UnitType::Sicario => game_assets.sicario_sprite.clone(),
        UnitType::Enforcer => game_assets.enforcer_sprite.clone(),
        UnitType::Sniper => game_assets.sicario_sprite.clone(), // Reuse for now
        UnitType::HeavyGunner => game_assets.enforcer_sprite.clone(), // Reuse for now
        UnitType::Medic => game_assets.sicario_sprite.clone(),  // Reuse for now
        UnitType::Ovidio => game_assets.ovidio_sprite.clone(),
        UnitType::Roadblock => game_assets.roadblock_sprite.clone(),
        UnitType::Soldier => game_assets.soldier_sprite.clone(),
        UnitType::SpecialForces => game_assets.special_forces_sprite.clone(),
        UnitType::Tank => game_assets.vehicle_sprite.clone(), // Reuse for now
        UnitType::Helicopter => game_assets.vehicle_sprite.clone(), // Reuse for now
        UnitType::Engineer => game_assets.soldier_sprite.clone(), // Reuse for now
        UnitType::Vehicle => game_assets.vehicle_sprite.clone(),
    }
}

pub fn spawn_health_bar(commands: &mut Commands, owner: Entity, position: Vec3) {
    // Background bar (red)
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(0.8, 0.2, 0.2), // Red background
                custom_size: Some(Vec2::new(32.0, 4.0)),
                ..default()
            },
            transform: Transform::from_translation(position + Vec3::new(0.0, 20.0, 0.5)),
            ..default()
        },
        HealthBar {
            owner,
            offset: Vec3::new(0.0, 20.0, 0.5),
        },
    ));

    // Foreground bar (green)
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(0.2, 0.8, 0.2), // Green foreground
                custom_size: Some(Vec2::new(32.0, 4.0)),
                ..default()
            },
            transform: Transform::from_translation(position + Vec3::new(0.0, 20.0, 0.6)),
            ..default()
        },
        HealthBar {
            owner,
            offset: Vec3::new(0.0, 20.0, 0.6),
        },
    ));
}

// ==================== INTEL OPERATOR SPAWNING ====================

pub fn spawn_intel_operator(
    commands: &mut Commands,
    intel_type: IntelType,
    position: Vec3,
    game_assets: &Res<GameAssets>,
) -> Entity {
    let (detection_range, stealth_level, cooldown_duration) = match intel_type {
        IntelType::Reconnaissance => (200.0, 0.8, 15.0), // High stealth, long range
        IntelType::RadioIntercept => (100.0, 0.6, 8.0),  // Medium stealth, faster intercepts
        IntelType::Informant => (50.0, 0.9, 30.0),       // Very stealthy, slow reports
        IntelType::CounterIntel => (150.0, 0.4, 20.0),   // Low stealth, detection focus
    };

    let (color, emoji) = match intel_type {
        IntelType::Reconnaissance => (Color::rgb(0.4, 0.6, 0.8), "👁️"), // Blue
        IntelType::RadioIntercept => (Color::rgb(0.8, 0.6, 0.2), "📡"), // Orange
        IntelType::Informant => (Color::rgb(0.6, 0.8, 0.4), "👤"),      // Green
        IntelType::CounterIntel => (Color::rgb(0.8, 0.4, 0.6), "🕵️"),   // Purple
    };

    let iso_position = world_to_iso(position);

    // Spawn the intel operator
    let entity = commands
        .spawn((
            SpriteBundle {
                sprite: Sprite {
                    color,
                    custom_size: Some(Vec2::new(24.0, 24.0)), // Smaller than combat units
                    ..default()
                },
                texture: game_assets.sicario_sprite.clone(), // Reuse existing sprite
                transform: Transform::from_translation(iso_position),
                ..default()
            },
            IntelOperator {
                intel_type: intel_type.clone(),
                detection_range,
                stealth_level,
                intel_cooldown: Timer::from_seconds(cooldown_duration, TimerMode::Once),
                last_intel_time: 0.0,
            },
            Movement {
                target_position: None,
                speed: 30.0, // Slower movement (stealth focused)
            },
            PathfindingAgent {
                path: Vec::new(),
                current_waypoint: 0,
                avoidance_radius: 20.0,
                max_speed: 30.0,
                stuck_timer: 0.0,
            },
        ))
        .id();

    // Add emoji overlay
    commands.spawn((
        Text2dBundle {
            text: Text::from_section(
                emoji,
                TextStyle {
                    font_size: 16.0,
                    color: Color::WHITE,
                    ..default()
                },
            ),
            transform: Transform::from_translation(iso_position + Vec3::new(0.0, 0.0, 0.1)),
            ..default()
        },
        HealthBar {
            owner: entity,
            offset: Vec3::new(0.0, 0.0, 0.1),
        },
    ));

    entity
}

pub fn spawn_cartel_intel_network(commands: &mut Commands, game_assets: &Res<GameAssets>) {
    // Spawn a basic intel network for the cartel

    // Radio intercept operator (hidden in safehouse area)
    spawn_intel_operator(
        commands,
        IntelType::RadioIntercept,
        Vec3::new(-50.0, 0.0, -30.0),
        game_assets,
    );

    // Reconnaissance scout (mobile, high stealth)
    spawn_intel_operator(
        commands,
        IntelType::Reconnaissance,
        Vec3::new(0.0, 0.0, 50.0),
        game_assets,
    );

    // Informant network (civilian contact)
    spawn_intel_operator(
        commands,
        IntelType::Informant,
        Vec3::new(80.0, 0.0, 20.0),
        game_assets,
    );

    info!(
        "🕵️ Intel Network deployed: Radio intercept, Reconnaissance, and Informant assets active"
    );
}
