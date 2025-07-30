use bevy::prelude::*;
use rand::{thread_rng, Rng};
use crate::components::*;
use crate::utils::play_tactical_sound;

// ==================== COMBAT HELPER FUNCTIONS ====================

pub fn calculate_damage_modifier(weapon_type: &WeaponType) -> f32 {
    match weapon_type {
        WeaponType::HeavyMachineGun | WeaponType::RPG => 1.5,
        WeaponType::TacticalRifle | WeaponType::CartelSniperRifle | WeaponType::MilitarySniperRifle => 1.3,
        WeaponType::VehicleWeapons => 2.0,
        _ => 1.0,
    }
}

pub fn calculate_ability_damage_modifier(effect_option: Result<&AbilityEffect, bevy::ecs::query::QueryEntityError>) -> f32 {
    if let Ok(effect) = effect_option {
        match effect.effect_type {
            EffectType::DamageBoost(multiplier) => multiplier,
            _ => 1.0,
        }
    } else {
        1.0
    }
}

pub fn calculate_damage_reduction(effect_option: Result<&AbilityEffect, bevy::ecs::query::QueryEntityError>) -> f32 {
    if let Ok(effect) = effect_option {
        match effect.effect_type {
            EffectType::DamageReduction(reduction) => reduction,
            EffectType::Intimidated => 0.7, // Intimidated units take less damage
            _ => 1.0,
        }
    } else {
        1.0
    }
}

pub fn get_weapon_sound(weapon_type: &WeaponType) -> &'static str {
    match weapon_type {
        WeaponType::RPG => "explosion",
        WeaponType::VehicleWeapons => "vehicle",
        _ => "gunfire",
    }
}

pub fn update_veterancy_level(unit: &mut Unit) {
    unit.veterancy_level = match unit.kills {
        0..=2 => VeterancyLevel::Recruit,
        3..=5 => VeterancyLevel::Veteran,
        _ => VeterancyLevel::Elite,
    };
}

pub fn find_combat_pairs(units: &[(Entity, &Unit, &Transform)]) -> Vec<(Entity, Entity, f32)> {
    let mut combat_events = Vec::new();
    
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
    
    combat_events
}

pub fn apply_combat_damage(
    commands: &mut Commands,
    attacker: Entity,
    target: Entity,
    base_damage: f32,
    unit_query: &mut Query<(Entity, &mut Unit, &Transform)>,
    effect_query: &Query<&AbilityEffect>,
) -> bool {
    // Get immutable data first
    let (attacker_transform, attacker_weapon) = if let Ok((_, unit, transform)) = unit_query.get(attacker) {
        (transform.translation, unit.equipment.weapon.clone())
    } else { return false; };
    
    let target_transform = if let Ok((_, _, transform)) = unit_query.get(target) {
        transform.translation
    } else { return false; };
    
    // Calculate damage modifiers
    let damage_modifier = calculate_damage_modifier(&attacker_weapon);
    let ability_damage_modifier = calculate_ability_damage_modifier(effect_query.get(attacker));
    let final_damage = base_damage * damage_modifier * ability_damage_modifier;
    
    // Update attacker cooldown and stats
    if let Ok((_, mut attacker_unit, _)) = unit_query.get_mut(attacker) {
        attacker_unit.attack_cooldown.reset();
    }
    
    // Apply damage to target (accounting for damage reduction effects)
    let target_died = if let Ok((_, mut target_unit, _)) = unit_query.get_mut(target) {
        let damage_reduction = calculate_damage_reduction(effect_query.get(target));
        let reduced_damage = final_damage * damage_reduction;
        target_unit.health -= reduced_damage;
        let died = target_unit.health <= 0.0;
        
        // Audio feedback
        let weapon_sound = get_weapon_sound(&attacker_weapon);
        play_tactical_sound(weapon_sound, &format!("Combat: {} damage dealt", reduced_damage as u32));
        
        died
    } else { false };
    
    // Update attacker if target died
    if target_died {
        if let Ok((_, mut attacker_unit, _)) = unit_query.get_mut(attacker) {
            attacker_unit.kills += 1;
            attacker_unit.experience += 10;
            update_veterancy_level(&mut attacker_unit);
            play_tactical_sound("radio", &format!("{:?} gains experience from elimination", attacker_unit.unit_type));
        }
    }
    
    // Create visual effects
    spawn_damage_indicator(commands, target_transform, final_damage);
    spawn_combat_particles(commands, attacker_transform, target_transform);
    
    target_died
}

pub fn clear_invalid_targets(
    unit_query: &mut Query<(Entity, &mut Unit, &Transform)>,
) {
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
    }
}

pub fn spawn_damage_indicator(commands: &mut Commands, position: Vec3, damage: f32) {
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

pub fn spawn_combat_particles(commands: &mut Commands, attacker_pos: Vec3, target_pos: Vec3) {
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