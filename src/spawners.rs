use bevy::prelude::*;
use crate::components::*;
use crate::resources::*;
use crate::utils::world_to_iso;

// ==================== UNIT SPAWNING FUNCTIONS ====================

pub fn spawn_unit(
    commands: &mut Commands, 
    unit_type: UnitType, 
    faction: Faction, 
    position: Vec3,
    game_assets: &Res<GameAssets>
) {
    let (health, damage, range, speed, weapon, armor) = match (&unit_type, &faction) {
        // Cartel units
        (UnitType::Sicario, Faction::Cartel) => (80.0, 25.0, 120.0, 80.0, WeaponType::BasicRifle, ArmorType::LightVest),
        (UnitType::Enforcer, Faction::Cartel) => (120.0, 35.0, 100.0, 70.0, WeaponType::AssaultRifle, ArmorType::TacticalVest),
        
        // Military units  
        (UnitType::Soldier, Faction::Military) => (100.0, 30.0, 110.0, 75.0, WeaponType::StandardIssue, ArmorType::TacticalVest),
        (UnitType::SpecialForces, Faction::Military) => (140.0, 40.0, 130.0, 85.0, WeaponType::TacticalRifle, ArmorType::HeavyArmor),
        (UnitType::Vehicle, Faction::Military) => (200.0, 50.0, 150.0, 60.0, WeaponType::VehicleWeapons, ArmorType::VehicleArmor),
        
        // Special cases
        (UnitType::Roadblock, _) => (150.0, 0.0, 0.0, 0.0, WeaponType::BasicRifle, ArmorType::None),
        _ => (100.0, 30.0, 100.0, 70.0, WeaponType::BasicRifle, ArmorType::None), // Default
    };

    let (sprite_handle, unit_color, emoji) = match unit_type {
        UnitType::Sicario => (&game_assets.sicario_sprite, Color::rgb(0.8, 0.2, 0.2), "🔫"),
        UnitType::Enforcer => (&game_assets.enforcer_sprite, Color::rgb(0.6, 0.1, 0.1), "⚔️"),
        UnitType::Ovidio => (&game_assets.ovidio_sprite, Color::rgb(1.0, 0.8, 0.0), "👑"),
        UnitType::Soldier => (&game_assets.soldier_sprite, Color::rgb(0.2, 0.6, 0.2), "🪖"),
        UnitType::SpecialForces => (&game_assets.special_forces_sprite, Color::rgb(0.1, 0.8, 0.1), "🎯"),
        UnitType::Vehicle => (&game_assets.vehicle_sprite, Color::rgb(0.1, 0.4, 0.1), "🚗"),
        UnitType::Roadblock => (&game_assets.roadblock_sprite, Color::rgb(0.8, 0.5, 0.2), "🚧"),
    };

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
        Unit {
            health,
            max_health: health,
            faction: faction.clone(),
            unit_type: unit_type.clone(),
            damage,
            range,
            movement_speed: speed,
            target: None,
            attack_cooldown: Timer::from_seconds(1.0, TimerMode::Once),
            experience: 0,
            kills: 0,
            veterancy_level: VeterancyLevel::Recruit,
            equipment: Equipment {
                weapon,
                armor,
                upgrades: vec![],
            },
        },
        Movement {
            target_position: None,
            speed,
        },
        AnimatedSprite {
            animation_timer: Timer::from_seconds(2.0, TimerMode::Repeating),
            scale_amplitude: 0.05, // Gentle pulsing
            rotation_speed: 0.1, // Slow rotation
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
            max_speed: speed,
            stuck_timer: 0.0,
        },
    ));

    let entity = entity.id();
    
    // Add obstacle component for roadblocks
    if unit_type == UnitType::Roadblock {
        commands.entity(entity).insert(Obstacle {
            radius: 50.0,
        });
    }

    // Emoji overlay for clear unit identification
    commands.spawn((
        Text2dBundle {
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
        },
    ));

    // Add health bar
    spawn_health_bar(commands, entity, iso_position);
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