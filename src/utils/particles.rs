use bevy::prelude::*;
use crate::components::ParticleEffect;
use crate::utils::world_to_iso;

// ==================== PARTICLE EFFECT UTILITIES ====================

/// Spawn muzzle flash effect at position
pub fn spawn_muzzle_flash(commands: &mut Commands, position: Vec3) {
    let iso_position = world_to_iso(position);
    
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(1.0, 0.8, 0.2), // Bright yellow-orange
                custom_size: Some(Vec2::new(12.0, 12.0)),
                ..default()
            },
            transform: Transform::from_translation(iso_position + Vec3::new(0.0, 0.0, 2.0)),
            ..default()
        },
        ParticleEffect {
            lifetime: Timer::from_seconds(0.1, TimerMode::Once),
            velocity: Vec3::ZERO,
        },
    ));
}

/// Spawn explosion particles at position with intensity
pub fn spawn_explosion_particles(commands: &mut Commands, position: Vec3, intensity: f32) {
    let iso_position = world_to_iso(position);
    let particle_count = (intensity * 0.1) as i32;
    
    for i in 0..particle_count.max(3).min(12) {
        let angle = (i as f32 / particle_count as f32) * std::f32::consts::PI * 2.0;
        let speed = intensity * 0.5;
        let offset = Vec3::new(angle.cos() * 20.0, angle.sin() * 20.0, 0.0);
        
        commands.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: Color::rgb(1.0, 0.4, 0.1), // Orange-red explosion
                    custom_size: Some(Vec2::new(8.0, 8.0)),
                    ..default()
                },
                transform: Transform::from_translation(iso_position + offset + Vec3::new(0.0, 0.0, 1.5)),
                ..default()
            },
            ParticleEffect {
                lifetime: Timer::from_seconds(0.5, TimerMode::Once),
                velocity: Vec3::new(angle.cos() * speed, angle.sin() * speed, 0.0),
            },
        ));
    }
    
    // Central bright flash
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(1.0, 1.0, 0.8), // Bright white-yellow
                custom_size: Some(Vec2::new(24.0, 24.0)),
                ..default()
            },
            transform: Transform::from_translation(iso_position + Vec3::new(0.0, 0.0, 2.0)),
            ..default()
        },
        ParticleEffect {
            lifetime: Timer::from_seconds(0.2, TimerMode::Once),
            velocity: Vec3::ZERO,
        },
    ));
}

/// Spawn damage number indicator
pub fn spawn_damage_numbers(commands: &mut Commands, position: Vec3, damage: f32, is_critical: bool) {
    let iso_position = world_to_iso(position);
    
    let color = if is_critical {
        Color::rgb(1.0, 0.2, 0.2) // Bright red for critical hits
    } else {
        Color::rgb(0.9, 0.9, 0.2) // Yellow for normal damage
    };
    
    let font_size = if is_critical { 20.0 } else { 16.0 };
    
    commands.spawn((
        Text2dBundle {
            text: Text::from_section(
                format!("{:.0}", damage),
                TextStyle {
                    font_size,
                    color,
                    ..default()
                },
            ),
            transform: Transform::from_translation(iso_position + Vec3::new(0.0, 15.0, 3.0)),
            ..default()
        },
        ParticleEffect {
            lifetime: Timer::from_seconds(1.5, TimerMode::Once),
            velocity: Vec3::new(0.0, 30.0, 0.0), // Float upward
        },
    ));
}

/// Spawn healing indicator
pub fn spawn_healing_indicator(commands: &mut Commands, position: Vec3, healing: f32) {
    let iso_position = world_to_iso(position);
    
    commands.spawn((
        Text2dBundle {
            text: Text::from_section(
                format!("+{:.0}", healing),
                TextStyle {
                    font_size: 18.0,
                    color: Color::rgb(0.2, 1.0, 0.2), // Bright green
                    ..default()
                },
            ),
            transform: Transform::from_translation(iso_position + Vec3::new(0.0, 20.0, 3.0)),
            ..default()
        },
        ParticleEffect {
            lifetime: Timer::from_seconds(1.2, TimerMode::Once),
            velocity: Vec3::new(0.0, 25.0, 0.0),
        },
    ));
    
    // Add small healing sparkles
    for i in 0..4 {
        let angle = (i as f32 / 4.0) * std::f32::consts::PI * 2.0;
        let offset = Vec3::new(angle.cos() * 15.0, angle.sin() * 15.0, 0.0);
        
        commands.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: Color::rgb(0.4, 1.0, 0.4), // Light green sparkles
                    custom_size: Some(Vec2::new(4.0, 4.0)),
                    ..default()
                },
                transform: Transform::from_translation(iso_position + offset + Vec3::new(0.0, 0.0, 1.0)),
                ..default()
            },
            ParticleEffect {
                lifetime: Timer::from_seconds(0.8, TimerMode::Once),
                velocity: Vec3::new(angle.cos() * 10.0, angle.sin() * 10.0 + 20.0, 0.0),
            },
        ));
    }
}

/// Spawn bullet trail effect
pub fn spawn_bullet_trail(commands: &mut Commands, start_pos: Vec3, end_pos: Vec3) {
    let iso_start = world_to_iso(start_pos);
    let iso_end = world_to_iso(end_pos);
    
    let direction = (iso_end - iso_start).normalize();
    let distance = iso_start.distance(iso_end);
    let trail_count = (distance / 20.0) as i32;
    
    for i in 0..trail_count.max(2).min(8) {
        let progress = i as f32 / trail_count as f32;
        let position = iso_start + direction * distance * progress;
        
        commands.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: Color::rgb(1.0, 0.9, 0.3), // Yellow bullet trail
                    custom_size: Some(Vec2::new(3.0, 8.0)),
                    ..default()
                },
                transform: Transform::from_translation(position + Vec3::new(0.0, 0.0, 1.0)),
                ..default()
            },
            ParticleEffect {
                lifetime: Timer::from_seconds(0.15, TimerMode::Once),
                velocity: Vec3::ZERO,
            },
        ));
    }
}

/// Spawn ability effect particles
pub fn spawn_ability_effect(commands: &mut Commands, position: Vec3, effect_type: &str, radius: f32) {
    let iso_position = world_to_iso(position);
    
    match effect_type {
        "precision_shot" => {
            // Bright focused beam effect
            commands.spawn((
                SpriteBundle {
                    sprite: Sprite {
                        color: Color::rgb(0.2, 0.8, 1.0), // Bright blue
                        custom_size: Some(Vec2::new(4.0, 40.0)),
                        ..default()
                    },
                    transform: Transform::from_translation(iso_position + Vec3::new(0.0, 0.0, 2.5)),
                    ..default()
                },
                ParticleEffect {
                    lifetime: Timer::from_seconds(0.3, TimerMode::Once),
                    velocity: Vec3::ZERO,
                },
            ));
        },
        "suppressive_fire" => {
            // Multiple rapid fire effects in area
            let particle_count = (radius / 20.0) as i32;
            for i in 0..particle_count.max(4).min(12) {
                let angle = (i as f32 / particle_count as f32) * std::f32::consts::PI * 2.0;
                let offset = Vec3::new(angle.cos() * radius * 0.7, angle.sin() * radius * 0.7, 0.0);
                
                commands.spawn((
                    SpriteBundle {
                        sprite: Sprite {
                            color: Color::rgb(1.0, 0.6, 0.1), // Orange suppression
                            custom_size: Some(Vec2::new(6.0, 6.0)),
                            ..default()
                        },
                        transform: Transform::from_translation(iso_position + offset + Vec3::new(0.0, 0.0, 1.5)),
                        ..default()
                    },
                    ParticleEffect {
                        lifetime: Timer::from_seconds(0.4, TimerMode::Once),
                        velocity: Vec3::ZERO,
                    },
                ));
            }
        },
        "field_medic" => {
            // Green healing wave
            for i in 0..6 {
                let angle = (i as f32 / 6.0) * std::f32::consts::PI * 2.0;
                let offset = Vec3::new(angle.cos() * radius * 0.5, angle.sin() * radius * 0.5, 0.0);
                
                commands.spawn((
                    SpriteBundle {
                        sprite: Sprite {
                            color: Color::rgb(0.2, 1.0, 0.3), // Bright green
                            custom_size: Some(Vec2::new(8.0, 8.0)),
                            ..default()
                        },
                        transform: Transform::from_translation(iso_position + offset + Vec3::new(0.0, 0.0, 1.0)),
                        ..default()
                    },
                    ParticleEffect {
                        lifetime: Timer::from_seconds(1.0, TimerMode::Once),
                        velocity: Vec3::new(angle.cos() * 20.0, angle.sin() * 20.0, 0.0),
                    },
                ));
            }
        },
        "tank_shell" => {
            // Massive explosion with shockwave
            spawn_explosion_particles(commands, position, 100.0);
            
            // Add shockwave ring
            commands.spawn((
                SpriteBundle {
                    sprite: Sprite {
                        color: Color::rgb(0.8, 0.8, 0.8), // Gray shockwave
                        custom_size: Some(Vec2::new(radius * 2.0, 4.0)),
                        ..default()
                    },
                    transform: Transform::from_translation(iso_position + Vec3::new(0.0, 0.0, 0.5)),
                    ..default()
                },
                ParticleEffect {
                    lifetime: Timer::from_seconds(0.6, TimerMode::Once),
                    velocity: Vec3::ZERO,
                },
            ));
        },
        _ => {
            // Default effect
            commands.spawn((
                SpriteBundle {
                    sprite: Sprite {
                        color: Color::rgb(1.0, 1.0, 1.0), // White flash
                        custom_size: Some(Vec2::new(16.0, 16.0)),
                        ..default()
                    },
                    transform: Transform::from_translation(iso_position + Vec3::new(0.0, 0.0, 2.0)),
                    ..default()
                },
                ParticleEffect {
                    lifetime: Timer::from_seconds(0.3, TimerMode::Once),
                    velocity: Vec3::ZERO,
                },
            ));
        }
    }
}

/// Spawn unit spawn indicator
pub fn spawn_unit_spawn_effect(commands: &mut Commands, position: Vec3) {
    let iso_position = world_to_iso(position);
    
    // Spawn indicator ring
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(0.3, 0.9, 1.0), // Cyan spawn effect
                custom_size: Some(Vec2::new(40.0, 4.0)),
                ..default()
            },
            transform: Transform::from_translation(iso_position + Vec3::new(0.0, 0.0, 0.5)),
            ..default()
        },
        ParticleEffect {
            lifetime: Timer::from_seconds(0.8, TimerMode::Once),
            velocity: Vec3::ZERO,
        },
    ));
    
    // Upward sparkles
    for i in 0..6 {
        let angle = (i as f32 / 6.0) * std::f32::consts::PI * 2.0;
        let offset = Vec3::new(angle.cos() * 15.0, angle.sin() * 15.0, 0.0);
        
        commands.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: Color::rgb(0.5, 1.0, 1.0), // Light cyan
                    custom_size: Some(Vec2::new(3.0, 3.0)),
                    ..default()
                },
                transform: Transform::from_translation(iso_position + offset),
                ..default()
            },
            ParticleEffect {
                lifetime: Timer::from_seconds(1.2, TimerMode::Once),
                velocity: Vec3::new(0.0, 40.0, 0.0),
            },
        ));
    }
}